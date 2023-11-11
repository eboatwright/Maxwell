use crate::Point;
use crate::heatmaps::*;
use crate::piece::*;

pub fn generate_starting_position(string: &'static str) -> [u8; 64] {
	let mut board: [u8; 64] = [0; 64];

	for i in 0..64 {
		board[i] = match string.chars().collect::<Vec<char>>()[i] {
			'♟' => PAWN | WHITE,
			'♞' => KNIGHT | WHITE,
			'♝' => BISHOP | WHITE,
			'♜' => ROOK | WHITE,
			'♛' => QUEEN | WHITE,
			'♚' => KING | WHITE,

			'♙' => PAWN | BLACK,
			'♘' => KNIGHT | BLACK,
			'♗' => BISHOP | BLACK,
			'♖' => ROOK | BLACK,
			'♕' => QUEEN | BLACK,
			'♔' => KING | BLACK,

			_ => 0,
		};
	}

	board
}

pub fn get_index_for_piece(piece: u8) -> usize {
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