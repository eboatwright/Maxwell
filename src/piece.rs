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


pub fn is_white(piece: u8) -> bool {
	piece & COLOR_MASK == WHITE
}

pub fn get_piece_type(piece: u8) -> u8 {
	piece & PIECE_MASK
}


pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 0b_0001;
pub const EN_PASSANT_FLAG: u8       = 0b_0010;
pub const CASTLE_KINGSIDE_FLAG: u8  = 0b_0011;
pub const CASTLE_QUEENSIDE_FLAG: u8 = 0b_0100;
pub const KNIGHT_PROMO_FLAG: u8     = 0b_0101;
pub const BISHOP_PROMO_FLAG: u8     = 0b_0110;
pub const ROOK_PROMO_FLAG: u8       = 0b_0111;
pub const QUEEN_PROMO_FLAG: u8      = 0b_1000;

pub const MOVE_FLAG_MASK: u32    = 0b_1111_0000_000000_000000;
pub const MOVE_CAPTURE_MASK: u32 = 0b_0000_1111_000000_000000;
pub const MOVE_FROM_MASK: u32    = 0b_0000_0000_111111_000000;
pub const MOVE_TO_MASK: u32      = 0b_0000_0000_000000_111111;


pub fn get_move_flag(m: u32) -> u32 {
	(m & MOVE_FLAG_MASK) >> 16
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

pub fn build_move(flag: u32, capture: u32, from: usize, to: usize) -> u32 {
	((flag << 16) as usize | (capture << 12) as usize | (from << 6) | to) as u32
}