use crate::Point;
use crate::heatmaps::*;
use crate::piece::*;

pub fn generate_starting_position(string: String) -> [Piece; 64] {
	let mut board: [Piece; 64] = [Piece::new(PieceType::None, false); 64];

	for i in 0..64 {
		board[i] = match string.chars().collect::<Vec<char>>()[i] {
			'♟' => Piece::new(PieceType::Pawn, true),
			'♝' => Piece::new(PieceType::Bishop, true),
			'♞' => Piece::new(PieceType::Knight, true),
			'♜' => Piece::new(PieceType::Rook, true),
			'♛' => Piece::new(PieceType::Queen, true),
			'♚' => Piece::new(PieceType::King, true),

			'♙' => Piece::new(PieceType::Pawn, false),
			'♗' => Piece::new(PieceType::Bishop, false),
			'♘' => Piece::new(PieceType::Knight, false),
			'♖' => Piece::new(PieceType::Rook, false),
			'♕' => Piece::new(PieceType::Queen, false),
			'♔' => Piece::new(PieceType::King, false),

			_ => Piece::new(PieceType::None, false),
		};
	}

	board
}

pub fn get_index_for_piece(piece: Piece) -> usize {
	match (piece.piece_type, piece.is_white) {
		(PieceType::Pawn, true) => 1,
		(PieceType::Bishop, true) => 2,
		(PieceType::Knight, true) => 3,
		(PieceType::Rook, true) => 4,
		(PieceType::Queen, true) => 5,
		(PieceType::King, true) => 6,

		(PieceType::Pawn, false) => 7,
		(PieceType::Bishop, false) => 8,
		(PieceType::Knight, false) => 9,
		(PieceType::Rook, false) => 10,
		(PieceType::Queen, false) => 11,
		(PieceType::King, false) => 12,

		(PieceType::None, ..) => 0,
	}
}

pub fn get_worth_for_piece(piece: Piece, mut i: usize) -> i32 {
	if !piece.is_white {
		// let mut p = Point::from_index(i);
		// p.y = 7 - p.y;
		// i = (p.x + p.y * 8) as usize;
		i = 63 - i;
	}

	let worth = match piece.piece_type {
		PieceType::Pawn => 100   + PAWN_HEATMAP[i],
		PieceType::Bishop => 320 + BISHOP_HEATMAP[i],
		PieceType::Knight => 300 + KNIGHT_HEATMAP[i],
		PieceType::Rook => 500   + ROOK_HEATMAP[i],
		PieceType::Queen => 900  + QUEEN_HEATMAP[i],
		PieceType::King => 20000 + KING_MIDDLEGAME_HEATMAP[i],

		PieceType::None => 0,
	};

	worth
}