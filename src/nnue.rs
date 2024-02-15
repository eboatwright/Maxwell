// 768 inputs -> 196,608 weights -> 256 nodes -> 256 weights -> 1 output

use crate::move_data::{SHORT_CASTLE_FLAG, LONG_CASTLE_FLAG, EN_PASSANT_FLAG, MoveData};
use crate::pieces::{WHITE_ROOK, BLACK_ROOK, NO_PIECE, PROMOTABLE, build_piece, is_piece_white, char_to_piece};
use crate::Board;
use rand::Rng;

const LEARN_RATE: f32 = 0.008;

pub fn generate_random_weights(length: usize) -> Vec<f32> {
	let mut rng = rand::thread_rng();
	let mut result = vec![];

	for _ in 0..length {
		result.push(rng.gen_range(-2.0..2.0));
	}

	result
}

#[derive(Clone)]
pub struct DataPoint {
	fen: String,
	evaluation: i32,
}

impl DataPoint {
	pub fn new(fen: String, evaluation: i32) -> Self {
		Self {
			fen,
			evaluation,
		}
	}

	pub fn evaluation_as_pawns(&self) -> f32 { self.evaluation as f32 * 0.01 }
}

pub struct NNUE {
	// TODO: in the final version, these should be &'static [f32]
	pub input_layer: Vec<f32>,

	pub input_layer_weights: Vec<f32>,
	pub input_layer_biases: Vec<f32>,

	pub hidden_layer_weights: Vec<f32>,
	pub hidden_layer_biases: Vec<f32>,
}

impl NNUE {
	pub fn from(
		input_layer_weights: Vec<f32>,
		input_layer_biases: Vec<f32>,

		hidden_layer_weights: Vec<f32>,
		hidden_layer_biases: Vec<f32>,
	) -> Self {
		Self {
			input_layer: input_layer_biases.clone(),

			input_layer_weights,
			input_layer_biases,

			hidden_layer_weights,
			hidden_layer_biases,
		}
	}

	pub fn initialize(
		board: &Board,

		input_layer_weights: Vec<f32>,
		input_layer_biases: Vec<f32>,

		hidden_layer_weights: Vec<f32>,
		hidden_layer_biases: Vec<f32>,
	) -> Self {
		let mut nnue = NNUE::from(
			input_layer_weights,
			input_layer_biases,

			hidden_layer_weights,
			hidden_layer_biases,
		);

		for i in 0..64 {
			let piece = board.get_piece(i);
			if piece != NO_PIECE {
				nnue.activate(i, piece as u8);
			}
		}

		nnue
	}

	pub fn setup_fen(&mut self, fen: &String) {
		self.input_layer = self.input_layer_biases.clone();

		let fen_split = fen.split(' ').collect::<Vec<&str>>();

		let piece_rows = fen_split[0].split('/').collect::<Vec<&str>>();
		let mut i = 0;

		for row in piece_rows {
			for piece in row.chars() {
				if let Ok(empty_squares) = piece.to_string().parse::<usize>() {
					i += empty_squares;
				} else {
					let piece = char_to_piece(piece);
					self.activate(i as u8, piece as u8);
				}
			}
		}
	}

	fn get_index(square: u8, piece: u8) -> usize {
		(square * 12 + piece) as usize
	}

	pub fn activate(&mut self, square: u8, piece: u8) {
		let length = self.input_layer.len();
		let index = Self::get_index(square, piece);

		for i in 0..length {
			self.input_layer[i] += self.input_layer_weights[index * length + i];
		}
	}

	pub fn deactivate(&mut self, square: u8, piece: u8) {
		let length = self.input_layer.len();
		let index = Self::get_index(square, piece);

		for i in 0..length {
			self.input_layer[i] -= self.input_layer_weights[index * length + i];
		}
	}

	pub fn make_move(&mut self, data: &MoveData) {
		self.deactivate(data.from, data.piece);

		if PROMOTABLE.contains(&data.flag) {
			let promotion = build_piece(is_piece_white(data.piece as usize), data.flag as usize);
			self.activate(data.to, promotion as u8);
		} else {
			self.activate(data.to, data.piece);

			if data.flag == SHORT_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.deactivate(63, WHITE_ROOK as u8);
					self.activate(61, WHITE_ROOK as u8);
				} else {
					self.deactivate(7, BLACK_ROOK as u8);
					self.activate(5, BLACK_ROOK as u8);
				}
			} else if data.flag == LONG_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.deactivate(56, WHITE_ROOK as u8);
					self.activate(59, WHITE_ROOK as u8);
				} else {
					self.deactivate(0, BLACK_ROOK as u8);
					self.activate(3, BLACK_ROOK as u8);
				}
			}
		}

		if data.capture != NO_PIECE as u8 {
			if data.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(data.piece as usize) {
						data.to + 8
					} else {
						data.to - 8
					};

				self.deactivate(en_passant_square, data.capture);
			} else {
				self.deactivate(data.to, data.capture);
			}
		}
	}

	pub fn undo_move(&mut self, data: &MoveData) {
		self.activate(data.from, data.piece);

		if PROMOTABLE.contains(&data.flag) {
			let promotion = build_piece(is_piece_white(data.piece as usize), data.flag as usize);
			self.deactivate(data.to, promotion as u8);
		} else {
			self.deactivate(data.to, data.piece);

			if data.flag == SHORT_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.activate(63, WHITE_ROOK as u8);
					self.deactivate(61, WHITE_ROOK as u8);
				} else {
					self.activate(7, BLACK_ROOK as u8);
					self.deactivate(5, BLACK_ROOK as u8);
				}
			} else if data.flag == LONG_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.activate(56, WHITE_ROOK as u8);
					self.deactivate(59, WHITE_ROOK as u8);
				} else {
					self.activate(0, BLACK_ROOK as u8);
					self.deactivate(3, BLACK_ROOK as u8);
				}
			}
		}

		if data.capture != NO_PIECE as u8 {
			if data.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(data.piece as usize) {
						data.to + 8
					} else {
						data.to - 8
					};

				self.activate(en_passant_square, data.capture);
			} else {
				self.activate(data.to, data.capture);
			}
		}
	}

	pub fn evaluate(&self) -> f32 {
		let mut output = self.hidden_layer_biases[0];

		for i in 0..self.input_layer.len() {
			output += Self::clipped_relu(self.input_layer[i]) * self.hidden_layer_weights[i];
		}

		output
	}

	fn clipped_relu(x: f32) -> f32 {
		x.clamp(0.0, 1.0)
	}

	fn clipped_relu_derivative(x: f32) -> f32 {
		if 0.0 < x && x < 1.0 {
			1.0
		} else {
			0.0
		}
	}

	fn get_error_of_data_point(&mut self, data_point: &DataPoint) -> f32 {
		self.setup_fen(&data_point.fen);
		let output = self.evaluate();

		(output - data_point.evaluation_as_pawns()).powf(2.0)
	}

	pub fn get_total_error_of_data_set(&mut self, training_data: &Vec<DataPoint>) -> f32 {
		let mut total_error = 0.0;

		for data_point in training_data.iter() {
			total_error += self.get_error_of_data_point(data_point);
		}

		total_error / training_data.len() as f32
	}

	pub fn train_by_gradient_descent(&mut self, training_data: &Vec<DataPoint>) {
		const H: f32 = 0.00001;
		let original_error = self.get_total_error_of_data_set(training_data);



		// Calculate input layer gradients

		let mut input_layer_weight_error_gradient = vec![];
		let mut input_layer_bias_error_gradient = vec![];

		for i in 0..self.input_layer_weights.len() {
			self.input_layer_weights[i] += H;
			input_layer_weight_error_gradient.push((self.get_total_error_of_data_set(training_data) - original_error) / H);
			self.input_layer_weights[i] -= H;
		}

		for i in 0..self.input_layer_biases.len() {
			self.input_layer_biases[i] += H;
			input_layer_bias_error_gradient.push((self.get_total_error_of_data_set(training_data) - original_error) / H);
			self.input_layer_biases[i] -= H;
		}



		// Calculate hidden layer gradients

		let mut hidden_layer_weight_error_gradient = vec![];
		let mut hidden_layer_bias_error_gradient = vec![];

		for i in 0..self.hidden_layer_weights.len() {
			self.hidden_layer_weights[i] += H;
			hidden_layer_weight_error_gradient.push((self.get_total_error_of_data_set(training_data) - original_error) / H);
			self.hidden_layer_weights[i] -= H;
		}

		for i in 0..self.hidden_layer_biases.len() {
			self.hidden_layer_biases[i] += H;
			hidden_layer_bias_error_gradient.push((self.get_total_error_of_data_set(training_data) - original_error) / H);
			self.hidden_layer_biases[i] -= H;
		}



		// Apply gradients

		for i in 0..self.input_layer_weights.len() {
			self.input_layer_weights[i] -= input_layer_weight_error_gradient[i] * LEARN_RATE;
		}

		for i in 0..self.input_layer_biases.len() {
			self.input_layer_biases[i] -= input_layer_bias_error_gradient[i] * LEARN_RATE;
		}

		for i in 0..self.hidden_layer_weights.len() {
			self.hidden_layer_weights[i] -= hidden_layer_weight_error_gradient[i] * LEARN_RATE;
		}

		for i in 0..self.hidden_layer_biases.len() {
			self.hidden_layer_biases[i] -= hidden_layer_bias_error_gradient[i] * LEARN_RATE;
		}
	}
}