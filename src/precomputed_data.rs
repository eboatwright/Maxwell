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

pub const DIRECTION_OFFSETS: [i8; 8] = [-8, 8, -1, 1, -7, 7, -9, 9];

pub const SHORT_CASTLE_MASK: [u64; 2] = [
	(1 << 5) | (1 << 6),
	(1 << 61) | (1 << 62),
];

pub const LONG_CASTLE_MASK: [u64; 2] = [
	(1 << 2) | (1 << 3),
	(1 << 58) | (1 << 59),
];


#[derive(Clone)]
pub struct PrecomputedData {
	pub squares_to_edge: [[usize; 8]; 64],

	pub pawn_bitboards: [[u64; 64]; 2],
	pub knight_bitboards: [u64; 64],
	pub king_bitboards: [u64; 64],
}

impl PrecomputedData {
	pub fn calculate() -> Self {
		Self {
			squares_to_edge: Self::calculate_squares_to_edge(),

			pawn_bitboards: Self::calculate_pawn_bitboards(),
			knight_bitboards: Self::calculate_knight_bitboards(),
			king_bitboards: Self::calculate_king_bitboards(),
		}
	}



	fn calculate_squares_to_edge() -> [[usize; 8]; 64] {
		let mut squares_to_edge = [[0; 8]; 64];

		for y in 0..8 {
			for x in 0..8 {
				let north_to_edge = y;
				let south_to_edge = 7 - y;
				let west_to_edge = x;
				let east_to_edge = 7 - x;

				squares_to_edge[x + y * 8] = [
					north_to_edge,
					south_to_edge,
					west_to_edge,
					east_to_edge,

					usize::min(north_to_edge, east_to_edge),
					usize::min(south_to_edge, west_to_edge),
					usize::min(north_to_edge, west_to_edge),
					usize::min(south_to_edge, east_to_edge),
				];
			}
		}

		squares_to_edge
	}




	fn calculate_pawn_bitboards() -> [[u64; 64]; 2] {
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




	fn calculate_knight_bitboards() -> [u64; 64] {
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




	fn calculate_king_bitboards() -> [u64; 64] {
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