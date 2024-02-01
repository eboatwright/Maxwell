use colored::{Colorize, ColoredString};

pub const SQUARE_COORDINATES: [&str; 64] = [
	"a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
	"a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
	"a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
	"a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
	"a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
	"a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
	"a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
	"a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
];

pub fn coordinate_to_index(coordinate: &str) -> u8 {
	for i in 0..64 {
		if SQUARE_COORDINATES[i] == coordinate {
			return i as u8;
		}
	}

	0
}

pub fn move_str_is_valid(move_str: &str) -> bool {
	if move_str.len() < 4 {
		return false;
	}

	let move_str = move_str.to_lowercase();

	if !SQUARE_COORDINATES.contains(&&move_str[0..2]) {
		return false;
	}

	if !SQUARE_COORDINATES.contains(&&move_str[2..4]) {
		return false;
	}

	if let Some(promotion) = move_str.chars().nth(4) {
		if !(promotion == 'n'
		|| promotion == 'b'
		|| promotion == 'r'
		|| promotion == 'q') {
			return false;
		}
	}

	true
}

pub fn pop_lsb(bitboard: &mut u64) -> u8 {
	let i = bitboard.trailing_zeros();
	*bitboard &= *bitboard - 1;
	i as u8
}

pub fn get_lsb(bitboard: u64) -> u8 {
	bitboard.trailing_zeros() as u8
}

pub const CHECKMATE_EVAL: i32 = 100000;

pub fn evaluation_is_mate(evaluation: i32) -> bool {
	evaluation.abs() > CHECKMATE_EVAL - 100
}

pub fn ply_from_mate(evaluation: i32) -> u8 {
	(CHECKMATE_EVAL - evaluation.abs()) as u8
}

pub fn print_bitboard(label: &str, one: ColoredString, bitboard: u64) {
	println!("{}", label);
	println!("---------------------------------");
	for rank in 0..8 {
		for file in 0..8 {
			let i = file + rank * 8;

			let bit = if bitboard & (1 << i) != 0 {
				one.clone()
			} else {
				"0".normal()
			};

			print!("| {} ", bit);
		}
		println!("|\n---------------------------------");
	}
}