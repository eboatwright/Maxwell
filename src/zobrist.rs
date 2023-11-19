use crate::piece::*;
use crate::Board;
use crate::PIECES_IN_ORDER;
use macroquad::rand::{srand, gen_range};

pub const RAND_SEED: u64 = 3141592653589793238;

#[derive(Clone)]
pub struct Zobrist {
	pub pieces: [[[u64; 64]; 6]; 2],
	pub castling_rights: [u64; 16],
	pub en_passant: [u64; 9],
	pub side_to_move: u64,
}

impl Zobrist {
	pub fn generate() -> Self {
		srand(RAND_SEED);


		let mut pieces = [[[0; 64]; 6]; 2];
		for color in 0..2 {
			for piece in 0..6 {
				for square in 0..64 {
					pieces[color][piece][square] = gen_range(0, u64::MAX);
				}
			}
		}

		let mut castling_rights = [0; 16];
		for i in 0..16 {
			castling_rights[i] = gen_range(0, u64::MAX);
		}

		let mut en_passant = [0; 9];
		for i in 1..9 {
			en_passant[i] = gen_range(0, u64::MAX);
		}


		Self {
			pieces,
			castling_rights,
			en_passant,
			side_to_move: gen_range(0, u64::MAX),
		}
	}
}