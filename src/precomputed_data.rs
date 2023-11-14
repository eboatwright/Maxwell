pub const A_FILE: u64 = 0b_10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
pub const B_FILE: u64 = 0b_01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
pub const C_FILE: u64 = 0b_00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000;
pub const D_FILE: u64 = 0b_00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
pub const E_FILE: u64 = 0b_00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
pub const F_FILE: u64 = 0b_00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
pub const G_FILE: u64 = 0b_00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
pub const H_FILE: u64 = 0b_00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

pub const NOT_A_FILE: u64 = B_FILE | C_FILE | D_FILE | E_FILE | F_FILE | G_FILE | H_FILE;
pub const NOT_AB_FILES: u64 = C_FILE | D_FILE | E_FILE | F_FILE | G_FILE | H_FILE;

pub const NOT_H_FILE: u64 = A_FILE | B_FILE | C_FILE | D_FILE | E_FILE | F_FILE | G_FILE;
pub const NOT_GH_FILES: u64 = A_FILE | B_FILE | C_FILE | D_FILE | E_FILE | F_FILE;

#[derive(Clone)]
pub struct PrecomputedData {
	pub pawn_bitboards: [[u64; 64]; 2],
	pub knight_bitboards: [u64; 64],
	pub king_bitboards: [u64; 64],
}

impl PrecomputedData {
	pub fn initialize() -> Self {
		Self {
			pawn_bitboards: Self::initialize_pawn_bitboards(),
			knight_bitboards: Self::initialize_knight_bitboards(),
			king_bitboards: Self::initialize_king_bitboards(),
		}
	}

	fn initialize_pawn_bitboards() -> [[u64; 64]; 2] {
		let mut bitboards = [[0; 64]; 2];

		for i in 8..56 {
			let mut bitboard = 0;

			bitboard |= ((1 << i) & NOT_H_FILE) << 7;
			bitboard |= ((1 << i) & NOT_A_FILE) << 9;

			bitboards[0][i] = bitboard;
		}

		for i in 8..56 {
			let mut bitboard = 0;

			bitboard |= ((1 << i) & NOT_H_FILE) >> 9;
			bitboard |= ((1 << i) & NOT_A_FILE) >> 7;

			bitboards[1][i] = bitboard;
		}

		bitboards
	}

	fn initialize_knight_bitboards() -> [u64; 64] {
		let mut bitboards = [0; 64];

		for i in 0..64 {
			let mut bitboard = 0;

			bitboard |= ((1 << i) & NOT_AB_FILES) << 10;
			bitboard |= ((1 << i) & NOT_A_FILE) << 17;
			bitboard |= ((1 << i) & NOT_H_FILE) << 15;
			bitboard |= ((1 << i) & NOT_GH_FILES) << 6;

			bitboard |= ((1 << i) & NOT_GH_FILES) >> 10;
			bitboard |= ((1 << i) & NOT_H_FILE) >> 17;
			bitboard |= ((1 << i) & NOT_A_FILE) >> 15;
			bitboard |= ((1 << i) & NOT_AB_FILES) >> 6;

			bitboards[i] = bitboard;
		}

		bitboards
	}

	fn initialize_king_bitboards() -> [u64; 64] {
		let mut bitboards = [0; 64];

		for i in 0..64 {
			let mut bitboard = 0;

			bitboard |= ((1 << i) & NOT_H_FILE) >> 9;
			bitboard |= (1 << i) >> 8;
			bitboard |= ((1 << i) & NOT_A_FILE) >> 7;
			bitboard |= ((1 << i) & NOT_H_FILE) >> 1;

			bitboard |= ((1 << i) & NOT_A_FILE) << 9;
			bitboard |= (1 << i) << 8;
			bitboard |= ((1 << i) & NOT_H_FILE) << 7;
			bitboard |= ((1 << i) & NOT_A_FILE) << 1;

			bitboards[i] = bitboard;
		}

		bitboards
	}
}