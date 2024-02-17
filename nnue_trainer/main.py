'''
Network structure
   768 nodes (Input layer)
-> 256 nodes (Hidden layer, Clipped ReLU activation)
-> 1 node (Output layer)

and for the hidden layer -> the output layer, there's 8 "buckets"
or in other words, 8 different sets of weights and biases that are selected
depending on the number of pieces currently on the board


Notes:
For the buckets, do I simulate all the weights and biases with 8 output nodes and just pick the one I need,
or only use the weights and biases for the ones I'm going to use?


Plan:

0) Initialize with random weights, or start with already / partially tuned weights

--- Everything after this loops ---

1) Clear the old testing data (?)

2) Loop for X amount of games
	2a) Play a move
		10% chance for a random move
		90% chance for the best move from the engine

	2b) Save the position in a temporary list

	2c) When the game ends, label each position in the temporary list with the outcome of the game, and put them into the testing data

3) Train weights to predict outcomes of the testing data

4) Save the resulting weights
'''


import math
import random
import itertools
import subprocess

import chess
import chess.engine


GAMES_PER_MATCH = 250
EPOCHS_PER_TRAIN = 2 # ?
MINIBATCH_SIZE = 200
LEARNING_RATE = 0.05

DEPTH_PER_MOVE = 10
PERC_CHANCE_FOR_RANDOM_MOVE = 5

INPUT_NODES = 768
HIDDEN_NODES = 256
OUTPUT_NODES = 1

BUCKETS = 8


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

def error(output, target):
	return Matrix.pow(Matrix.subtract(output, target), 2.0)

def error_derivative(output, target):
	return 2.0 * (output - target)


class TrainingResults:
	def __init__(self):
		self.games = 0
		self.positions = 0


class DataPoint:
	def __init__(self, fen, outcome):
		self.fen = fen
		self.outcome = outcome


class Matrix:
	def __init__(self, rows, cols):
		self.rows = rows
		self.cols = cols

		self.data = []
		for row in range(rows):
			new_row = []
			for col in range(cols):
				new_row.append(0.0)
			self.data.append(new_row)

	def from_2d_list(input_2d_list):
		result = Matrix(len(input_2d_list), len(input_2d_list[0]))
		result.data = input_2d_list
		return result

	def fill_zeros(self):
		for row in range(self.rows):
			for col in range(self.cols):
				self.data[row][col] = 0.0

	def flatten(self):
		result = []

		for row in range(self.rows):
			for col in range(self.cols):
				result.append(self.data[row][col])

		return result

	def transpose(a):
		result = Matrix(a.cols, a.rows)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[col][row] = a.data[row][col]

		return result

	def random(rows, cols):
		result = Matrix(rows, cols)

		for row in range(rows):
			for col in range(cols):
				result.data[row][col] = random.uniform(-0.8, 0.8)

		return result

	def add(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] + b.data[row][col]

		return result

	def subtract(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] - b.data[row][col]

		return result

	def multiply(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] * b.data[row][col]

		return result

	def divide(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] / b.data[row][col]

		return result

	def divide_by_num(mat, num):
		result = Matrix(mat.rows, mat.cols)

		for row in range(mat.rows):
			for col in range(mat.cols):
				result.data[row][col] = mat.data[row][col] / num

		return result

	def dot(a, b):
		result = Matrix(a.rows, b.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				sum_of_column = 0

				for offset in range(a.cols):
					sum_of_column += a.data[row][offset] * b.data[offset][col]

				result.data[row][col] = sum_of_column

		return result

	def scale(m, s):
		result = Matrix(m.rows, m.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				result.data[row][col] = m.data[row][col] * s

		return result

	def pow(m, e):
		result = Matrix(m.rows, m.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				result.data[row][col] = m.data[row][col] ** e

		return result

	def map(m, fn):
		for row in range(m.rows):
			for col in range(m.cols):
				m.data[row][col] = fn(m.data[row][col])
		return m


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
		self.inputs = Matrix(1, INPUT_NODES)
		self.bucket = 0

		self.hidden_layer = Layer(
			[Matrix.random(INPUT_NODES, HIDDEN_NODES)], # Weights
			[Matrix.random(1, HIDDEN_NODES)], # Biases
			clipped_relu,
		)

		random_output_layer_weights = []
		random_output_layer_biases = []
		for i in range(BUCKETS):
			random_output_layer_weights.append(Matrix.random(HIDDEN_NODES, OUTPUT_NODES))
			random_output_layer_biases.append(Matrix.random(1, OUTPUT_NODES))

		self.output_layer = Layer(
			random_output_layer_weights,
			random_output_layer_biases,
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

		self.bucket = math.floor((piece_count - 1) / 4)

	def forward_pass(self):
		self.hidden_layer.forward_pass(self.inputs)
		self.output_layer.forward_pass(self.hidden_layer.outputs, self.bucket)

		return self.output_layer.outputs

	def get_total_error(self, data_batch):
		total_error = 0.0

		for data_point in data_batch:
			self.setup(data_point.fen)
			output = self.forward_pass().data[0][0]
			total_error += (output - data_point.outcome) ** 2.0

		return total_error / len(data_batch)

	def back_prop(self, data_batch):
		output_layer_error = Matrix.from_2d_list([[self.get_total_error(data_batch)]])

		output_layer_gradients = Matrix.map(self.output_layer.outputs, sigmoid_derivative)
		output_layer_gradients = Matrix.multiply(output_layer_gradients, output_layer_error)
		output_layer_gradients = Matrix.scale(output_layer_gradients, LEARNING_RATE)

		self.output_layer.biases[self.bucket] = Matrix.add(self.output_layer.biases[self.bucket], output_layer_gradients)

		output_layer_delta = Matrix.dot(Matrix.transpose(self.hidden_layer.outputs), output_layer_gradients)

		self.output_layer.weights[self.bucket] = Matrix.add(self.output_layer.weights[self.bucket], output_layer_delta)


		hidden_layer_error = Matrix.dot(output_layer_error, Matrix.transpose(self.output_layer.weights[self.bucket]))

		hidden_layer_gradients = Matrix.map(self.hidden_layer.outputs, clipped_relu_derivative)
		hidden_layer_gradients = Matrix.multiply(hidden_layer_gradients, hidden_layer_error)
		hidden_layer_gradients = Matrix.scale(hidden_layer_gradients, LEARNING_RATE)

		self.hidden_layer.biases[0] = Matrix.add(self.hidden_layer.biases[0], hidden_layer_gradients)

		hidden_layer_delta = Matrix.dot(Matrix.transpose(self.inputs), hidden_layer_gradients)

		self.hidden_layer.weights[0] = Matrix.add(self.hidden_layer.weights[0], hidden_layer_delta)


training_results = TrainingResults()
nn = NeuralNetwork()
data_points = []


def play_games():
	maxwell_engine = chess.engine.SimpleEngine.popen_uci(["./../target/release/maxwell", "debug_output=false"])

	for game_index in range(GAMES_PER_MATCH):
		print(f"Playing game {game_index + 1}...")

		board = chess.Board()

		# TODO: start from a random opening position

		fen_strings = []

		while not board.is_game_over():
			if random.randint(1, 100) < PERC_CHANCE_FOR_RANDOM_MOVE:
				board.push(random.choice(list(board.legal_moves)))
			else:
				result = maxwell_engine.play(board, chess.engine.Limit(depth=DEPTH_PER_MOVE))
				board.push(result.move)

			fen_strings.append(board.fen())

		game_outcome = 0.0

		match board.outcome().winner:
			case chess.WHITE:
				game_outcome = 1.0

			case chess.BLACK:
				game_outcome = -1.0

		for fen in fen_strings:
			data_points.append(DataPoint(fen, game_outcome))

	maxwell_engine.close()


if __name__ == "__main__":
	print("### MAXWELL NNUE TRAINER ###\n")

	training_cycle = 0

	while True:
		data_points = []

		print(f"Training cycle {training_cycle + 1}:")

		print(f"Starting {GAMES_PER_MATCH} self-play games...")

		play_games()

		training_results.games += GAMES_PER_MATCH;
		training_results.positions += len(data_points)

		print("Self-play done!\n")
		print("Training network...")

		for epoch in range(EPOCHS_PER_TRAIN):
			print(f"Epoch {epoch + 1}...")

			random.shuffle(data_points)

			data_point_index = 0
			while data_point_index < len(data_points):
				next_index = min(data_point_index + MINIBATCH_SIZE, len(data_points))
				minibatch = data_points[data_point_index:next_index]

				nn.back_prop(minibatch)

				data_point_index = next_index

		print("Done training!")
		print("Calculating total error...")
		print(f"Total error: {nn.get_total_error(data_points)}")
		print("Done!\n")

		training_cycle += 1

		if training_cycle % 5 == 0:
			print("Saving weights...")

			weight_output_file = open("../src/nnue_weights.rs", "w")

			flattened_input_layer_weights = nn.hidden_layer.weights[0].flatten()
			weight_output_file.write(f"pub const INPUT_LAYER_WEIGHTS: [f32; {len(flattened_input_layer_weights)}] = {flattened_input_layer_weights};\n")

			flattened_input_layer_biases = nn.hidden_layer.biases[0].flatten()
			weight_output_file.write(f"pub const INPUT_LAYER_BIASES: [f32; {len(flattened_input_layer_biases)}] = {flattened_input_layer_biases};\n")

			flattened_output_layer_weights = []

			for m in nn.output_layer.weights:
				flattened_output_layer_weights.extend(m.flatten())

			weight_output_file.write(f"pub const HIDDEN_LAYER_WEIGHTS: [f32; {len(flattened_output_layer_weights)}] = {flattened_output_layer_weights};\n")

			flattened_output_layer_biases = []

			for m in nn.output_layer.biases:
				flattened_output_layer_biases.extend(m.flatten())

			weight_output_file.write(f"pub const HIDDEN_LAYER_BIASES: [f32; {len(flattened_output_layer_biases)}] = {flattened_output_layer_biases};\n")

			weight_output_file.close()

			cargobuild = subprocess.Popen(["cargo", "build", "--release"])
			cargobuild.communicate()

			print("\nDone!\n")

		print(f"Total games played: {training_results.games}")
		print(f"Total positions trained on: {training_results.positions}\n")
