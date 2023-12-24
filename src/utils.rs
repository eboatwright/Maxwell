use colored::{Colorize, ColoredString};

pub fn pop_lsb(bitboard: &mut u64) -> u8 {
	let i = bitboard.trailing_zeros();
	*bitboard &= *bitboard - 1;
	i as u8
}

pub fn index_to_coordinate(index: u8) -> String {
	format!("{}{}",
		match index % 8 {
			0 => 'a',
			1 => 'b',
			2 => 'c',
			3 => 'd',
			4 => 'e',
			5 => 'f',
			6 => 'g',
			7 => 'h',
			_ => '?',
		},
		8 - (index / 8),
	)
}

pub fn coordinate_to_index(coordinate: &str) -> u8 {
	// .next() effectively gives me the first character
	(match coordinate.chars().next().expect("Invalid coordinate") {
		'a' => 0,
		'b' => 1,
		'c' => 2,
		'd' => 3,
		'e' => 4,
		'f' => 5,
		'g' => 6,
		'h' => 7,
		_ => 0,
	}) + (8 - coordinate.chars().nth(1).expect("Invalid coordinate").to_string().parse::<u8>().expect("Invalid coordinate")) * 8
}


pub const CHECKMATE_EVAL: i32 = 100000;

pub fn evaluation_is_mate(evaluation: i32) -> bool {
	evaluation.abs() > CHECKMATE_EVAL - 100
}

pub fn moves_ply_from_mate(evaluation: i32) -> u8 {
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