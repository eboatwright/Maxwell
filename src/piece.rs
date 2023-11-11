pub const PROMOTABLE_PIECES: [PieceType; 4] = [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen];

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(Eq, Hash)]
pub enum PieceType {
	Pawn,
	Knight,
	Bishop,
	Rook,
	Queen,
	King,
	None,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(Eq, Hash)]
pub struct Piece {
	pub piece_type: PieceType,
	pub is_white: bool,
}

impl Piece {
	pub fn new(piece_type: PieceType, is_white: bool) -> Self {
		Self {
			piece_type,
			is_white,
		}
	}

	pub fn none() -> Self {
		Self {
			piece_type: PieceType::None,
			is_white: false,
		}
	}
}

#[derive(Copy, Clone, Debug)]
#[derive(Eq, Hash)]
pub struct PieceMove {
	pub from: usize,
	pub to: usize,

	pub promotion_type: PieceType,
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

			promotion_type: PieceType::None,
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