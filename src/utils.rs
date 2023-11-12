use crate::heatmaps::*;
use crate::piece::*;

// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
pub fn generate_board_from_fen(fen: &'static str) -> [u8; 64] {
	let sections: Vec<&str> = fen.split(' ').collect();
	let pieces = sections[0].chars().collect::<Vec<char>>();

	let mut board: [u8; 64] = [0; 64];
	let mut board_index = 0usize;

	for i in 0..pieces.len() {
		if let Ok(number_of_empty_squares) = pieces[i].to_string().parse::<usize>() {
			board_index += number_of_empty_squares;
		} else {
			board[board_index] = match pieces[i] {
				'P' => WHITE | PAWN,
				'N' => WHITE | KNIGHT,
				'B' => WHITE | BISHOP,
				'R' => WHITE | ROOK,
				'Q' => WHITE | QUEEN,
				'K' => WHITE | KING,

				'p' => BLACK | PAWN,
				'n' => BLACK | KNIGHT,
				'b' => BLACK | BISHOP,
				'r' => BLACK | ROOK,
				'q' => BLACK | QUEEN,
				'k' => BLACK | KING,

				_ => 0,
			};
			if board[board_index] != 0 {
				board_index += 1;
			}
		}
	}

	board
}

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