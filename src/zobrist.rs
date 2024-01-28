use crate::value_holder::ValueHolder;
use crate::pieces::*;
use crate::Board;
use crate::move_data::*;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

const SEED: u64 = 19274892; // old seed: 3141592653589793238

pub struct Zobrist {
	pub key: ValueHolder<u64>,

	pieces: [[u64; 64]; PIECE_COUNT],
	castling_rights: [u64; 16],
	en_passant: [u64; 9],
	en_passant_file: usize, // Used for make_null_move
	side_to_move: u64,
}

impl Default for Zobrist {
	fn default() -> Self {
		Self {
			key: ValueHolder::new(0),

			pieces: [[0; 64]; PIECE_COUNT],
			castling_rights: [0; 16],
			en_passant: [0; 9],
			en_passant_file: 0,
			side_to_move: 0,
		}
	}
}

impl Zobrist {
	pub fn generate(board: &mut Board) -> Self {
		let mut zobrist = Self::default();
		let mut rng = Pcg64::seed_from_u64(SEED);

		for piece in 0..PIECE_COUNT {
			for square in 0..64 {
				zobrist.pieces[piece][square] = rng.gen::<u64>();
			}
		}

		for i in 0..16 {
			zobrist.castling_rights[i] = rng.gen::<u64>();
		}

		for i in 0..9 {
			zobrist.en_passant[i] = rng.gen::<u64>();
		}

		zobrist.side_to_move = rng.gen::<u64>();


		let mut initial_key = 0;

		for i in 0..64 {
			let piece = board.get_piece(i);
			if piece != NO_PIECE {
				initial_key ^= zobrist.pieces[piece][i as usize];
			}
		}

		initial_key ^= zobrist.castling_rights[board.castling_rights.current as usize];

		initial_key ^= zobrist.en_passant[board.en_passant_file];

		if !board.white_to_move {
			initial_key ^= zobrist.side_to_move;
		}

		zobrist.key = ValueHolder::new(initial_key);


		zobrist
	}

	pub fn is_repetition(&self, fifty_move_counter: usize) -> bool {
		// let mut count = 0;
		for i in ((self.key.index - fifty_move_counter)..self.key.index).step_by(2) {
			if self.key.history[i] == self.key.current {
				return true;
			}
		}
		false
	}

	// There's probably still bugs here; I'm very tired :`D
	pub fn make_move(
		&mut self,
		data: MoveData,
		last_move: MoveData,
		castling_rights: u8,
		last_castling_rights: u8,
	) {
		let to = data.to as usize;

		self.key.current ^= self.pieces[data.piece as usize][data.from as usize];

		if !PROMOTABLE.contains(&data.flag) {
			self.key.current ^= self.pieces[data.piece as usize][to];
		} else {
			self.key.current ^= self.pieces[build_piece(is_piece_white(data.piece as usize), data.flag as usize)][to];
		}

		if data.flag == SHORT_CASTLE_FLAG {
			let rook = build_piece(is_piece_white(data.piece as usize), ROOK);
			self.key.current ^= self.pieces[rook][to + 1];
			self.key.current ^= self.pieces[rook][to - 1];
		} else if data.flag == LONG_CASTLE_FLAG {
			let rook = build_piece(is_piece_white(data.piece as usize), ROOK);
			self.key.current ^= self.pieces[rook][to - 2];
			self.key.current ^= self.pieces[rook][to + 1];
		} else if data.capture != NO_PIECE as u8 {
			if data.flag == EN_PASSANT_FLAG {
				let pawn_to_en_passant = if is_piece_white(data.piece as usize) {
					to + 8
				} else {
					to - 8
				};

				self.key.current ^= self.pieces[data.capture as usize][pawn_to_en_passant];
			} else {
				self.key.current ^= self.pieces[data.capture as usize][to];
			}
		}

		self.key.current ^= self.castling_rights[last_castling_rights as usize];
		self.key.current ^= self.castling_rights[castling_rights as usize];

		if last_move.flag == DOUBLE_PAWN_PUSH_FLAG {
			let file = (last_move.to as usize % 8) + 1;
			self.key.current ^= self.en_passant[file];
		}

		if data.flag == DOUBLE_PAWN_PUSH_FLAG {
			let file = (to % 8) + 1;
			self.key.current ^= self.en_passant[file];
			self.en_passant_file = file;
		}

		self.key.current ^= self.side_to_move;

		self.key.push();
	}

	pub fn make_null_move(&mut self) {
		self.key.current ^= self.side_to_move;
		self.key.current ^= self.en_passant[self.en_passant_file];

		self.key.push();
	}
}