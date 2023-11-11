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

#[derive(Copy, Clone, Debug)]
#[derive(Eq, Hash)]
pub struct PieceMove {
	pub from: usize,
	pub to: usize,

	pub promotion_type: u8,
	pub pawn_moving_twice: bool,
	pub en_passant_capture: Option<usize>,

	pub short_castle: bool,
	pub long_castle: bool,
}

impl Default for PieceMove {
	fn default() -> Self {
		Self {
			from: 0,
			to: 0,

			promotion_type: 0,
			pawn_moving_twice: false,
			en_passant_capture: None,

			short_castle: false,
			long_castle: false,
		}
	}
}

impl PartialEq for PieceMove {
	fn eq(&self, other: &PieceMove) -> bool {
			self.from == other.from
		 && self.to == other.to
	}
}