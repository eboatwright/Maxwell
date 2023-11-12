use crate::heatmaps::*;
use crate::piece::*;

pub fn get_image_index_for_piece(piece: u8) -> usize {
	if piece == 0 {
		return 0;
	}

	let base = match piece & 0b_0111 {
		PAWN => 1,
		KNIGHT => 2,
		BISHOP => 3,
		ROOK => 4,
		QUEEN => 5,
		KING => 6,

		_ => 0,
	};

	if is_white(piece) {
		return base;
	} else {
		return base + 6;
	}
}

pub fn get_worth_for_piece(piece: u8, mut i: usize) -> i32 {
	if !is_white(piece) {
		// let mut p = Point::from_index(i);
		// p.y = 7 - p.y;
		// i = (p.x + p.y * 8) as usize;
		i = 63 - i;
	}

	let worth = match piece & 0b_0111 {
		PAWN => 100   + PAWN_HEATMAP[i],
		KNIGHT => 300 + KNIGHT_HEATMAP[i],
		BISHOP => 320 + BISHOP_HEATMAP[i],
		ROOK => 500   + ROOK_HEATMAP[i],
		QUEEN => 900  + QUEEN_HEATMAP[i],
		KING => 20000 + KING_MIDDLEGAME_HEATMAP[i],

		_ => 0,
	};

	worth
}

pub fn is_white(piece: u8) -> bool {
	piece & 0b_1000 == WHITE
}

// If somebody knows a better way to do this please @ me :/
pub fn index_from_coordinate(coordinate: &'static str) -> Option<usize> {
	if coordinate.len() != 2 {
		return None;
	}


	let split = coordinate.to_string().chars().collect::<Vec<char>>();



	let file_index = match split[0] {
		'a' => 0,
		'b' => 1,
		'c' => 2,
		'd' => 3,
		'e' => 4,
		'f' => 5,
		'g' => 6,
		'h' => 7,
		_ => 69,
	};

	let rank = if split[1].is_digit(10) {
		split[1].to_digit(10).unwrap() as usize
	} else {
		69
	};



	let full_index = file_index + rank * 8;



	if full_index >= 64 {
		return None;
	}
	Some(full_index)
}