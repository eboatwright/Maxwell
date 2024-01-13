use crate::pieces::*;

pub const MIDDLEGAME_PAWN_TABLE: [i32; 64] = [
	 0,  0,  0,  0,  0,  0,  0,  0,
	50, 50, 50, 50, 50, 50, 50, 50,
	10, 10, 20, 30, 30, 20, 10, 10,
	 5,  5, 10, 25, 25, 10,  5,  5,
	 0,  0, 10, 20, 20,  0,  0,  0,
	 5,-20,  5,  0,  0,-20,-20,  5,
	10, 10, 10,-20,-20, 10, 10, 10,
	 0,  0,  0,  0,  0,  0,  0,  0,
];

pub const ENDGAME_PAWN_TABLE: [i32; 64] = [ // TODO: tweak this (maybe lower the values?)
	  0,  0,  0,  0,  0,  0,  0,  0,
	100,100,100,100,100,100,100,100,
	 70, 70, 70, 70, 70, 70, 70, 70,
	 50, 50, 50, 50, 50, 50, 50, 50,
	 30, 30, 30, 30, 30, 30, 30, 30,
	 15, 15, 15, 15, 15, 15, 15, 15,
	 15, 15, 15, 15, 15, 15, 15, 15,
	  0,  0,  0,  0,  0,  0,  0,  0,
];

pub const KNIGHT_TABLE: [i32; 64] = [
	-50,-40,-30,-30,-30,-30,-40,-50,
	-40,-20,  0,  0,  0,  0,-20,-40,
	-30,  0, 10, 15, 15, 10,  0,-30,
	-30,  5, 15, 20, 20, 15,  5,-30,
	-30,  0, 15, 20, 20, 15,  0,-30,
	-30,  5, 10, 15, 15, 10,  5,-30,
	-40,-20,  0,  5,  5,  0,-20,-40,
	-50,-40,-30,-30,-30,-30,-40,-50,
];

pub const BISHOP_TABLE: [i32; 64] = [
	-20,-10,-10,-10,-10,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5, 10, 10,  5,  0,-10,
	-10,  5,  5, 10, 10,  5,  5,-10,
	-10,  0, 10, 10, 10, 10,  0,-10,
	-10, 10, 10, 10, 10, 10, 10,-10,
	-10,  5,  0,  0,  0,  0,  5,-10,
	-20,-10,-10,-10,-10,-10,-10,-20,
];

pub const ROOK_TABLE: [i32; 64] = [
	  0,  0,  0,  0,  0,  0,  0,  0,
	  5, 10, 10, 10, 10, 10, 10,  5,
	 -5,  0,  0,  0,  0,  0,  0, -5,
	 -5,  0,  0,  0,  0,  0,  0, -5,
	 -5,  0,  0,  0,  0,  0,  0, -5,
	 -5,  0,  0,  0,  0,  0,  0, -5,
	 -5,  0,  0,  0,  0,  0,  0, -5,
	  0,  0,  0,  5,  5,  0,  0,  0,
];

pub const QUEEN_TABLE: [i32; 64] = [
	-20,-10,-10,  0,  0,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5,  5,  5,  5,  0,-10,
	 -5,  0,  5,  5,  5,  5,  0, -5,
	 -5,  0,  5,  5,  5,  5,  0, -5,
	-10,  5,  5,  5,  5,  5,  0,-10,
	-10,  0,  5,  0,  0,  0,  0,-10,
	-20,-10,-10,  0,  0,-10,-10,-20,
];

pub const MIDDLEGAME_KING_TABLE: [i32; 64] = [
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-20,-30,-30,-40,-40,-30,-30,-20,
	-10,-20,-20,-20,-20,-20,-20,-10,
	 20, 20,-20,-25,-25,-20, 20, 20,
	 20, 30, 20,-40,-10,-40, 30, 20,
];

pub const ENDGAME_KING_TABLE: [i32; 64] = [
	-50,-40,-30,-20,-20,-30,-40,-50,
	-30,-20,-10,  0,  0,-10,-20,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-30,  0,  0,  0,  0,-30,-30,
	-50,-30,-30,-30,-30,-30,-30,-50,
];

pub fn flip_index(i: usize) -> usize { i ^ 56 }

pub const PAWN_WORTH:   i32 = 100;
pub const KNIGHT_WORTH: i32 = 320; // Maybe increase knight and bishop worth to discourage trading for rook + pawn?
pub const BISHOP_WORTH: i32 = 330;
pub const ROOK_WORTH:   i32 = 500;
pub const QUEEN_WORTH:  i32 = 900;
pub const KING_WORTH:   i32 = 0; // chessprogramming.org says this should be 20k but I don't think it matters /\o/\

pub const BASE_WORTHS_OF_PIECE_TYPE: [i32; 6] = [
	PAWN_WORTH,
	KNIGHT_WORTH,
	BISHOP_WORTH,
	ROOK_WORTH,
	QUEEN_WORTH,
	KING_WORTH,
];

pub fn get_full_worth_of_piece(piece: usize, mut i: usize, endgame: f32) -> i32 {
	if !is_piece_white(piece) {
		i = flip_index(i);
	}

	match get_piece_type(piece) {
		PAWN   => PAWN_WORTH   + (MIDDLEGAME_PAWN_TABLE[i] as f32 * (1.0 - endgame) + ENDGAME_PAWN_TABLE[i] as f32 * endgame) as i32,
		KNIGHT => KNIGHT_WORTH + KNIGHT_TABLE[i],
		BISHOP => BISHOP_WORTH + BISHOP_TABLE[i],
		ROOK   => ROOK_WORTH   + ROOK_TABLE[i],
		QUEEN  => QUEEN_WORTH  + QUEEN_TABLE[i],

		KING   => (MIDDLEGAME_KING_TABLE[i] as f32 * (1.0 - endgame) + ENDGAME_KING_TABLE[i] as f32 * endgame) as i32,

		_ => 0,
	}
}