use crate::magic_numbers::MagicNumbers;
use std::ops::Range;

pub const A_FILE: u64 = 0b_10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
pub const B_FILE: u64 = A_FILE >> 1;
pub const C_FILE: u64 = A_FILE >> 2;
pub const D_FILE: u64 = A_FILE >> 3;
pub const E_FILE: u64 = A_FILE >> 4;
pub const F_FILE: u64 = A_FILE >> 5;
pub const G_FILE: u64 = A_FILE >> 6;
pub const H_FILE: u64 = A_FILE >> 7;

pub const NOT_A_FILE: u64 = B_FILE | C_FILE | D_FILE | E_FILE | F_FILE | G_FILE | H_FILE;
pub const NOT_AB_FILES: u64 = C_FILE | D_FILE | E_FILE | F_FILE | G_FILE | H_FILE;

pub const NOT_H_FILE: u64 = A_FILE | B_FILE | C_FILE | D_FILE | E_FILE | F_FILE | G_FILE;
pub const NOT_GH_FILES: u64 = A_FILE | B_FILE | C_FILE | D_FILE | E_FILE | F_FILE;

pub const FIRST_RANK: u64 = 0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;

pub const A1H8_DIAGONAL: u64 = 0b_00000001_00000010_00000100_00001000_00010000_00100000_01000000_10000000;
pub const A8H1_DIAGONAL: u64 = 0b_10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001;

pub const CENTER_SQUARES_BITBOARD: u64 = u64::MAX & !A_FILE & !H_FILE & !FIRST_RANK & !(FIRST_RANK << 56);

pub const NO:   i8 = -8;
pub const EA:   i8 =  1;
pub const SO:   i8 =  8;
pub const WE:   i8 = -1;
pub const NOEA: i8 = -7;
pub const SOWE: i8 =  7;
pub const SOEA: i8 =  9;
pub const NOWE: i8 = -9;

pub const ROOK_DIRECTIONS: Range<usize>   = 0..4;
pub const BISHOP_DIRECTIONS: Range<usize> = 4..8;
pub const QUEEN_DIRECTIONS: Range<usize>  = 0..8;

pub const DIRECTION_OFFSETS: [i8; 8] = [NO, EA, SO, WE, NOEA, SOEA, SOWE, NOWE];

pub struct PrecalculatedMoveData {
	pub squares_to_edge: [[usize; 8]; 64],

	pub file_of_square: [u64; 64],
	pub file_in_front_of_pawn: [[u64; 64]; 2],
	pub files_beside_square: [u64; 64],
	pub squares_ahead_of_pawn: [[u64; 64]; 2],

	pub pawn_attacks: [[u64; 64]; 2],
	pub knight_attacks: [u64; 64],
	pub king_attacks: [u64; 64],

	pub bishop_relevant_occupancy_masks: [u64; 64],
	pub bishop_attacks: [Vec<u64>; 64],

	pub rook_relevant_occupancy_masks: [u64; 64],
	pub rook_attacks: [Vec<u64>; 64],

	pub magic_numbers: MagicNumbers,
}

impl PrecalculatedMoveData {
	pub fn calculate() -> Self {
		let mut data = Self {
			squares_to_edge: [[0; 8]; 64],

			file_of_square: [0; 64],
			file_in_front_of_pawn: [[0; 64]; 2],
			files_beside_square: [0; 64],
			squares_ahead_of_pawn: [[0; 64]; 2],

			pawn_attacks: [[0; 64]; 2],
			knight_attacks: [0; 64],
			king_attacks: [0; 64],

			bishop_relevant_occupancy_masks: [0; 64],
			bishop_attacks: std::array::from_fn(|_| vec![]),

			rook_relevant_occupancy_masks: [0; 64],
			rook_attacks: std::array::from_fn(|_| vec![]),

			magic_numbers: Default::default(),
		};

		for i in 0..64 {
			let file = i % 8;
			let rank = i / 8;
			let piece = 1 << i;



			let north_to_edge = rank;
			let south_to_edge = 7 - rank;
			let west_to_edge  = file;
			let east_to_edge  = 7 - file;

			data.squares_to_edge[i] = [
				north_to_edge,
				east_to_edge,
				south_to_edge,
				west_to_edge,

				usize::min(north_to_edge, east_to_edge),
				usize::min(south_to_edge, east_to_edge),
				usize::min(south_to_edge, west_to_edge),
				usize::min(north_to_edge, west_to_edge),
			];



			data.file_of_square[i] = (A_FILE >> (8 - (i % 8) - 1)) ^ 1 << i;



			let mut file_in_front_of_black_pawn = 0;
			let mut file_in_front_of_white_pawn = 0;

			for rank in 1..data.squares_to_edge[i][2] { // South
				file_in_front_of_black_pawn |= 1 << (i + 8 * rank) as u64;
			}

			for rank in 1..data.squares_to_edge[i][0] { // North
				file_in_front_of_white_pawn |= 1 << (i - 8 * rank) as u64;
			}

			data.file_in_front_of_pawn[0][i] = file_in_front_of_black_pawn;
			data.file_in_front_of_pawn[1][i] = file_in_front_of_white_pawn;



			let mut files_beside_square = 0;

			files_beside_square |= (A_FILE >> (8 - (i % 8) - 2)) & NOT_H_FILE;
			files_beside_square |= (A_FILE >> (8 - (i % 8))) & NOT_A_FILE;

			data.files_beside_square[i] = files_beside_square;



			let mut squares_ahead_of_black_pawn = 0;
			let mut squares_ahead_of_white_pawn = 0;

			for rank in 1..data.squares_to_edge[i][2] { // South
				squares_ahead_of_black_pawn |= (1 << (i + 8 * rank - 1) as u64) & NOT_A_FILE;
				squares_ahead_of_black_pawn |= 1 << (i + 8 * rank) as u64;
				squares_ahead_of_black_pawn |= (1 << (i + 8 * rank + 1) as u64) & NOT_H_FILE;
			}

			for rank in 1..data.squares_to_edge[i][0] { // North
				squares_ahead_of_white_pawn |= (1 << (i - 8 * rank - 1) as u64) & NOT_A_FILE;
				squares_ahead_of_white_pawn |= 1 << (i - 8 * rank) as u64;
				squares_ahead_of_white_pawn |= (1 << (i - 8 * rank + 1) as u64) & NOT_H_FILE;
			}

			data.squares_ahead_of_pawn[0][i] = squares_ahead_of_black_pawn;
			data.squares_ahead_of_pawn[1][i] = squares_ahead_of_white_pawn;



			let mut black_pawn_bitboard = 0;

			black_pawn_bitboard |= (piece << 9) & NOT_H_FILE;
			black_pawn_bitboard |= (piece << 7) & NOT_A_FILE;

			data.pawn_attacks[0][i] = black_pawn_bitboard;



			let mut white_pawn_bitboard = 0;

			white_pawn_bitboard |= (piece >> 7) & NOT_H_FILE;
			white_pawn_bitboard |= (piece >> 9) & NOT_A_FILE;

			data.pawn_attacks[1][i] = white_pawn_bitboard;



			let mut knight_bitboard = 0;

			knight_bitboard |= (piece << 17) & NOT_H_FILE;
			knight_bitboard |= (piece << 10) & NOT_GH_FILES;
			knight_bitboard |= (piece << 6)  & NOT_AB_FILES;
			knight_bitboard |= (piece << 15) & NOT_A_FILE;

			knight_bitboard |= (piece >> 15) & NOT_H_FILE;
			knight_bitboard |= (piece >> 6)  & NOT_GH_FILES;
			knight_bitboard |= (piece >> 10) & NOT_AB_FILES;
			knight_bitboard |= (piece >> 17) & NOT_A_FILE;

			data.knight_attacks[i] = knight_bitboard;



			let mut king_bitboard = 0;

			king_bitboard |= (piece << 1) & NOT_H_FILE;
			king_bitboard |= (piece >> 1) & NOT_A_FILE;

			king_bitboard |= piece << 8;
			king_bitboard |= piece >> 8;

			king_bitboard |= (piece << 9) & NOT_H_FILE;
			king_bitboard |= (piece >> 9) & NOT_A_FILE;

			king_bitboard |= (piece << 7) & NOT_A_FILE;
			king_bitboard |= (piece >> 7) & NOT_H_FILE;

			data.king_attacks[i] = king_bitboard;



			// This is used in move generation so it's saved
			let mut bishop_relevant_occupancy_mask = 0;

			for direction_index in BISHOP_DIRECTIONS {
				for n in 1..data.squares_to_edge[i][direction_index] {
					let to = (i as i8 + DIRECTION_OFFSETS[direction_index] * n as i8) as usize;
					bishop_relevant_occupancy_mask |= 1 << to;
				}
			}

			data.bishop_relevant_occupancy_masks[i] = bishop_relevant_occupancy_mask;

			let mut rook_relevant_occupancy_mask = 0;

			for direction_index in ROOK_DIRECTIONS {
				for n in 1..data.squares_to_edge[i][direction_index] {
					let to = (i as i8 + DIRECTION_OFFSETS[direction_index] * n as i8) as usize;
					rook_relevant_occupancy_mask |= 1 << to;
				}
			}

			data.rook_relevant_occupancy_masks[i] = rook_relevant_occupancy_mask;



			// Generating bishop attacks
			let mut move_square_indexes = vec![];
			for j in 0..64 {
				if ((bishop_relevant_occupancy_mask >> j) & 1) == 1 {
					move_square_indexes.push(j);
				}
			}

			let number_of_patterns = 1 << move_square_indexes.len();
			let mut occupancy_bitboards = vec![0; number_of_patterns];

			for pattern in 0..number_of_patterns {
				for bit in 0..move_square_indexes.len() {
					occupancy_bitboards[pattern] |= (((pattern >> bit) & 1) as u64) << move_square_indexes[bit];
				}
			}

			data.bishop_attacks[i] = vec![0; 1 << (64 - data.magic_numbers.bishop_shift[i])];
			for occupancies in occupancy_bitboards {
				let bishop_bitboard = data.generate_sliding_moves_bitboard(i, BISHOP_DIRECTIONS, occupancies);
				let key = data.generate_bishop_key(i, occupancies);
				data.bishop_attacks[i][key] = bishop_bitboard;
			}



			// Generating rook attacks
			let mut move_square_indexes = vec![];
			for j in 0..64 {
				if ((rook_relevant_occupancy_mask >> j) & 1) == 1 {
					move_square_indexes.push(j);
				}
			}

			let number_of_patterns = 1 << move_square_indexes.len();
			let mut occupancy_bitboards = vec![0; number_of_patterns];

			for pattern in 0..number_of_patterns {
				for bit in 0..move_square_indexes.len() {
					occupancy_bitboards[pattern] |= (((pattern >> bit) & 1) as u64) << move_square_indexes[bit];
				}
			}

			data.rook_attacks[i] = vec![0; 1 << (64 - data.magic_numbers.rook_shift[i])];
			for occupancies in occupancy_bitboards {
				let rook_bitboard = data.generate_sliding_moves_bitboard(i, ROOK_DIRECTIONS, occupancies);
				let key = data.generate_rook_key(i, occupancies);
				data.rook_attacks[i][key] = rook_bitboard;
			}
		}

		data
	}

	pub fn generate_bishop_key(&self, piece_index: usize, occupancies: u64) -> usize {
		((occupancies * self.magic_numbers.bishop[piece_index]) >> self.magic_numbers.bishop_shift[piece_index]) as usize
	}

	pub fn generate_rook_key(&self, piece_index: usize, occupancies: u64) -> usize {
		((occupancies * self.magic_numbers.rook[piece_index]) >> self.magic_numbers.rook_shift[piece_index]) as usize
	}

	fn generate_sliding_moves_bitboard(
		&self,
		piece_index: usize,
		direction_range: Range<usize>,
		occupancies: u64,
	) -> u64 {
		let mut result = 0;

		for direction_index in direction_range {
			for n in 1..=self.squares_to_edge[piece_index][direction_index] {
				let to = (piece_index as i8 + DIRECTION_OFFSETS[direction_index] * n as i8) as usize;

				result |= 1 << to;

				if 1 << to & occupancies != 0 {
					break;
				}
			}
		}

		result
	}
}