import chess
import math
import subprocess

import config
from matrix import Matrix


def linear(inputs):
	return inputs

def clipped_relu(value):
	return max(0.0, min(1.0, value))

def clipped_relu_derivative(value):
	if 0.0 < value and value < 1.0:
		return 1.0

	return 0.0

def sigmoid(value):
	return 2 / (1 + math.exp(-value)) - 1

def sigmoid_derivative(value):
	return (2 * math.exp(-value)) / ((math.exp(-value) + 1) ** 2.0)


class Layer:
	def __init__(self, weights, biases, activation):
		self.weights = weights
		self.biases = biases
		self.activation = activation

		self.outputs = Matrix(1, len(biases))

	def forward_pass(self, inputs, bucket=0):
		self.outputs = Matrix.map(
			Matrix.add(
				Matrix.dot(inputs, self.weights[bucket]),
				self.biases[bucket],
			),
			self.activation,
		)


class NeuralNetwork:
	def __init__(self):
		self.inputs = Matrix(1, config.INPUT_NODES)
		self.bucket = 0

		self.hidden_layer = Layer(
			[Matrix.random(config.INPUT_NODES, config.HIDDEN_NODES)], # Weights
			[Matrix.random(1, config.HIDDEN_NODES)], # Biases
			clipped_relu,
		)

		# random_output_layer_weights = []
		# random_output_layer_biases = []
		# for i in range(BUCKETS):
		# 	random_output_layer_weights.append(Matrix.random(config.HIDDEN_NODES, config.OUTPUT_NODES))
		# 	random_output_layer_biases.append(Matrix.random(1, config.OUTPUT_NODES))

		self.output_layer = Layer(
			[Matrix.random(config.HIDDEN_NODES, config.OUTPUT_NODES)],
			[Matrix.random(1, config.OUTPUT_NODES)],
			sigmoid,
		)

	def setup(self, fen):
		self.inputs.fill_zeros()

		fen_rows = fen.split(' ')[0].split('/')
		i = 0
		piece_count = 0

		for row in fen_rows:
			for piece_symbol in row:
				if piece_symbol.isdigit():
					i += int(piece_symbol)
				else:
					piece = chess.Piece.from_symbol(piece_symbol)
					piece_index = (piece.piece_type - 1) + piece.color * 6

					self.inputs.data[0][i * 12 + piece_index] = 1.0

					i += 1
					piece_count += 1

		# self.bucket = math.floor((piece_count - 1) / 4)

	def forward_pass(self):
		self.hidden_layer.forward_pass(self.inputs)
		self.output_layer.forward_pass(self.hidden_layer.outputs, self.bucket)

		return self.output_layer.outputs

	def get_total_error(self, data_batch):
		total_error = 0.0

		for data_point in data_batch:
			self.setup(data_point.fen)
			output = self.forward_pass().data[0][0]
			total_error += data_point.outcome - output # ** 2.0?

		return total_error / len(data_batch)

	def back_prop(self, batch_index_from, batch_index_to, data_batch):
		# total_error = self.get_total_error(data_batch)
		# print(f"Batch {batch_index_from}~{batch_index_to} error: {total_error}")

		errors = []
		for data_point in data_batch:
			self.setup(data_point.fen)
			output = self.forward_pass().data[0][0]
			errors.append(data_point.outcome - output)

		total_errors = 0.0
		for error in errors:
			total_errors += abs(error)

			output_layer_error = Matrix.from_2d_list([[error]])

			output_layer_gradients = Matrix.map(self.output_layer.outputs, sigmoid_derivative)
			output_layer_gradients = Matrix.multiply(output_layer_gradients, output_layer_error)
			output_layer_gradients = Matrix.scale(output_layer_gradients, config.LEARNING_RATE)

			self.output_layer.biases[self.bucket] = Matrix.add(self.output_layer.biases[self.bucket], output_layer_gradients)

			output_layer_delta = Matrix.dot(Matrix.transpose(self.hidden_layer.outputs), output_layer_gradients)

			self.output_layer.weights[self.bucket] = Matrix.add(self.output_layer.weights[self.bucket], output_layer_delta)


			hidden_layer_error = Matrix.dot(output_layer_error, Matrix.transpose(self.output_layer.weights[self.bucket]))

			hidden_layer_gradients = Matrix.map(self.hidden_layer.outputs, clipped_relu_derivative)
			hidden_layer_gradients = Matrix.multiply(hidden_layer_gradients, hidden_layer_error)
			hidden_layer_gradients = Matrix.scale(hidden_layer_gradients, config.LEARNING_RATE)

			self.hidden_layer.biases[0] = Matrix.add(self.hidden_layer.biases[0], hidden_layer_gradients)

			hidden_layer_delta = Matrix.dot(Matrix.transpose(self.inputs), hidden_layer_gradients)

			self.hidden_layer.weights[0] = Matrix.add(self.hidden_layer.weights[0], hidden_layer_delta)

		print(f"Batch {batch_index_from}~{batch_index_to} error: {total_errors / len(errors)}")

	def save_weights(self):
		print("Saving weights...")

		weight_output_file = open("../src/nnue_weights.rs", "w")

		flattened_input_layer_weights = self.hidden_layer.weights[0].flatten()
		weight_output_file.write(f"pub const INPUT_LAYER_WEIGHTS: [f32; {len(flattened_input_layer_weights)}] = {flattened_input_layer_weights};\n")

		flattened_input_layer_biases = self.hidden_layer.biases[0].flatten()
		weight_output_file.write(f"pub const INPUT_LAYER_BIASES: [f32; {len(flattened_input_layer_biases)}] = {flattened_input_layer_biases};\n")

		flattened_output_layer_weights = []

		for m in self.output_layer.weights:
			flattened_output_layer_weights.extend(m.flatten())

		weight_output_file.write(f"pub const HIDDEN_LAYER_WEIGHTS: [f32; {len(flattened_output_layer_weights)}] = {flattened_output_layer_weights};\n")

		flattened_output_layer_biases = []

		for m in self.output_layer.biases:
			flattened_output_layer_biases.extend(m.flatten())

		weight_output_file.write(f"pub const HIDDEN_LAYER_BIASES: [f32; {len(flattened_output_layer_biases)}] = {flattened_output_layer_biases};\n")

		weight_output_file.close()

		cargobuild = subprocess.Popen(["cargo", "build", "--release"])
		cargobuild.communicate()

		print("\nDone!\n")