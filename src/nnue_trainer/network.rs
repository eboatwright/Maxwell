use crate::nnue_weights::*;
use crate::char_to_piece;
use std::io::{stdout, Write};
use std::fs::File;
use super::{
	config,
	math_funcs::*,
	nnue_trainer::DataPoint,
	matrix::Matrix,
};

type ActivationFn = fn(f32) -> f32;

#[derive(Clone)]
pub struct Layer {
	pub weights: Matrix,
	pub biases: Matrix,
	activation_fn: ActivationFn,

	outputs: Matrix,
}

impl Layer {
	fn new(weights: Matrix, biases: Matrix, activation_fn: ActivationFn) -> Self {
		let node_count = biases.data.len();

		Self {
			weights,
			biases,
			activation_fn,

			outputs: Matrix::empty(1, node_count)
		}
	}

	fn forward_pass(&mut self, inputs: &Matrix) {
		self.outputs = Matrix::map(
			&Matrix::add(
				&Matrix::dot(inputs, &self.weights),
				&self.biases,
			),
			self.activation_fn,
		);
	}
}

#[derive(Clone)]
pub struct Network {
	pub inputs: Matrix,
	pub hidden_layer: Layer,
	pub output_layer: Layer,
}

impl Network {
	pub fn new() -> Self {
		Self {
			inputs: Matrix::empty(1, config::INPUT_NODES),
			hidden_layer: Layer::new(
				Matrix::random(config::INPUT_NODES, config::HIDDEN_NODES),
				Matrix::random(1, config::HIDDEN_NODES),
				clipped_relu,
			),
			output_layer: Layer::new(
				Matrix::random(config::HIDDEN_NODES, config::OUTPUT_NODES),
				Matrix::random(1, config::OUTPUT_NODES),
				sigmoid,
			),
		}
	}

	pub fn setup(&mut self, fen: &String) {
		self.inputs.fill_zeros();

		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let row_split = fen_split[0].split('/').collect::<Vec<&str>>();
		let mut i = 0;

		for row in row_split {
			for piece_char in row.chars() {
				if let Ok(empty_squares) = piece_char.to_string().parse::<usize>() {
					i += empty_squares;
				} else {
					let piece = char_to_piece(piece_char);
					self.inputs.data[i * 12 + piece] = 1.0;

					i += 1;
				}
			}
		}
	}

	pub fn forward_pass(&mut self) -> &Matrix {
		self.hidden_layer.forward_pass(&self.inputs);
		self.output_layer.forward_pass(&self.hidden_layer.outputs);

		&self.output_layer.outputs
	}

	pub fn back_prop(&mut self, data_index: usize, data_batch: &[DataPoint]) {
		let mut total_error = 0.0;

		let mut new_output_layer_biases = self.output_layer.biases.clone();
		let mut new_output_layer_weights = self.output_layer.weights.clone();

		let mut new_hidden_layer_biases = self.hidden_layer.biases.clone();
		let mut new_hidden_layer_weights = self.hidden_layer.weights.clone();

		for data_point in data_batch.iter() {
			self.setup(&data_point.fen);
			let output = self.forward_pass().data[0];
			let error = data_point.outcome - output;

			total_error += error.abs().powf(2.0);

			let output_layer_error = Matrix { rows: 1, cols: 1, data: vec![error] };

			let mut output_layer_gradients = Matrix::map(&self.output_layer.outputs, sigmoid_derivative);
			output_layer_gradients.multiply_mut(&output_layer_error);
			output_layer_gradients.multiply_by_num_mut(config::LEARNING_RATE);

			new_output_layer_biases.add_mut(&output_layer_gradients);
			new_output_layer_weights.add_mut(&Matrix::dot(&self.hidden_layer.outputs.transposed(), &output_layer_gradients));


			let hidden_layer_error = Matrix::dot(&output_layer_error, &self.output_layer.weights.transposed());

			let mut hidden_layer_gradients = Matrix::map(&self.hidden_layer.outputs, clipped_relu_derivative);
			hidden_layer_gradients.multiply_mut(&hidden_layer_error);
			hidden_layer_gradients.multiply_by_num_mut(config::LEARNING_RATE);

			new_hidden_layer_biases.add_mut(&hidden_layer_gradients);
			new_hidden_layer_weights.add_mut(&Matrix::dot(&self.inputs.transposed(), &hidden_layer_gradients));
		}

		self.output_layer.biases = new_output_layer_biases;
		self.output_layer.weights = new_output_layer_weights;

		self.hidden_layer.biases = new_hidden_layer_biases;
		self.hidden_layer.weights = new_hidden_layer_weights;

		println!("Batch {}~{} error: {}", data_index, data_index + data_batch.len(), total_error / data_batch.len() as f32);
	}

	pub fn save_weights(&self) {
		print!("Saving weights... ");
		stdout().flush().expect("Failed to flush stdout");

		let mut output_file = File::create("./src/nnue_trainer/output_weights.rs").expect("Failed to create weight output file");

		writeln!(output_file, "pub const HIDDEN_LAYER_WEIGHTS: [f32; {}] = {:?};",
			self.hidden_layer.weights.data.len(),
			self.hidden_layer.weights.data
		).expect("Failed to write HIDDEN_LAYER_WEIGHTS");

		writeln!(output_file, "pub const HIDDEN_LAYER_BIASES: [f32; {}] = {:?};",
			self.hidden_layer.biases.data.len(),
			self.hidden_layer.biases.data
		).expect("Failed to write HIDDEN_LAYER_BIASES");

		writeln!(output_file, "pub const OUTPUT_LAYER_WEIGHTS: [f32; {}] = {:?};",
			self.output_layer.weights.data.len(),
			self.output_layer.weights.data
		).expect("Failed to write OUTPUT_LAYER_WEIGHTS");

		writeln!(output_file, "pub const OUTPUT_LAYER_BIASES: [f32; {}] = {:?};",
			self.output_layer.biases.data.len(),
			self.output_layer.biases.data
		).expect("Failed to write OUTPUT_LAYER_BIASES");

		println!("Done!\n");
	}
}