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

pub const WHITE: u8  = 0b_1000;
pub const BLACK: u8  = 0b_0000;