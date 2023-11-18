pub const PROMOTABLE_PIECES: [u8; 4] = [
	KNIGHT,
	BISHOP,
	ROOK,
	QUEEN,
];


pub const PAWN: u8   = 0b_0001;
pub const KNIGHT: u8 = 0b_0010;
pub const BISHOP: u8 = 0b_0011;
pub const ROOK: u8   = 0b_0100;
pub const QUEEN: u8  = 0b_0101;
pub const KING: u8   = 0b_0110;

pub const WHITE: u8 = 0b_1000;
pub const BLACK: u8 = 0b_0000;

pub const COLOR_MASK: u8 = 0b1000;
pub const PIECE_MASK: u8 = 0b0111;


pub const PIECES_IN_ORDER: [u8; 6] = [
	PAWN,
	KNIGHT,
	BISHOP,
	ROOK,
	QUEEN,
	KING,
];


pub const PAWN_WORTH: i32   = 100;
pub const KNIGHT_WORTH: i32 = 300;
pub const BISHOP_WORTH: i32 = 310;
pub const ROOK_WORTH: i32   = 500;
pub const QUEEN_WORTH: i32  = 900;
pub const KING_WORTH: i32   = 20000;

pub const PIECE_WORTH: [i32; 6] = [
	PAWN_WORTH,
	KNIGHT_WORTH,
	BISHOP_WORTH,
	ROOK_WORTH,
	QUEEN_WORTH,
	KING_WORTH,
];


pub fn is_white(piece: u8) -> bool {
	piece & COLOR_MASK == WHITE
}

pub fn get_piece_color(piece: u8) -> u8 {
	(piece & COLOR_MASK) >> 3
}

pub fn get_piece_type(piece: u8) -> u8 {
	piece & PIECE_MASK
}

pub fn build_piece(is_white: bool, piece_type: u8) -> u8 {
	((is_white as u8) << 3) | piece_type
}


pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 0b_0001;
pub const EN_PASSANT_FLAG: u8       = 0b_0110;
pub const CASTLE_SHORT_FLAG: u8     = 0b_0111;
pub const CASTLE_LONG_FLAG: u8      = 0b_1000;

pub const MOVE_FLAG_MASK: u32    = 0b_1111_0000_000000_000000;
pub const MOVE_CAPTURE_MASK: u32 = 0b_0000_1111_000000_000000;
pub const MOVE_FROM_MASK: u32    = 0b_0000_0000_111111_000000;
pub const MOVE_TO_MASK: u32      = 0b_0000_0000_000000_111111;

pub fn get_move_flag(m: u32) -> u8 {
	((m & MOVE_FLAG_MASK) >> 16) as u8
}

pub fn get_move_capture(m: u32) -> u8 {
	((m & MOVE_CAPTURE_MASK) >> 12) as u8
}

pub fn get_move_from(m: u32) -> usize {
	((m & MOVE_FROM_MASK) >> 6) as usize
}

pub fn get_move_to(m: u32) -> usize {
	(m & MOVE_TO_MASK) as usize
}

pub fn build_move(flag: u8, capture: u8, from: usize, to: usize) -> u32 {
	(((flag as u32) << 16) as usize | ((capture as u32) << 12) as usize | (from << 6) | to) as u32
}


pub const WHITE_LONGCASTLE: u8  = 0b_1000;
pub const WHITE_SHORTCASTLE: u8 = 0b_0100;
pub const BLACK_LONGCASTLE: u8  = 0b_0010;
pub const BLACK_SHORTCASTLE: u8 = 0b_0001;

pub const CASTLING: [u8; 2] = [
	0b_0011, // Black
	0b_1100, // White
];