#[derive(Copy, Clone, PartialEq)]
pub enum PieceType {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King,
	None,
}

#[derive(Copy, Clone)]
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