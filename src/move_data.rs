use crate::pieces::{char_to_piece, get_piece_type};
use crate::pieces::{PROMOTABLE, piece_to_char, NO_PIECE};
use crate::utils::{coordinate_to_index, index_to_coordinate};

// 1, 2, 3, 4, 7, 8, 9, 10 for promoting pieces
pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 11;
pub const EN_PASSANT_FLAG: u8       = 12;
pub const SHORT_CASTLE_FLAG: u8     = 13;
pub const LONG_CASTLE_FLAG: u8      = 14;

pub const NULL_MOVE: MoveData = MoveData {
	flag: 0,
	capture: NO_PIECE as u8,
	piece: NO_PIECE as u8,
	from: 0,
	to: 0,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MoveData {
	pub flag: u8,
	pub capture: u8,
	pub piece: u8,
	pub from: u8,
	pub to: u8,
}

impl Default for MoveData {
	fn default() -> Self { NULL_MOVE }
}

impl MoveData {
	pub fn from_coordinates(coordinates: String) -> Self {
		Self {
			flag: if coordinates.len() > 4 {
				get_piece_type(char_to_piece(coordinates.chars().nth(4).unwrap())) as u8
			} else {
				0
			},
			capture: 0,
			piece: NO_PIECE as u8,
			from: coordinate_to_index(&coordinates[0..2]),
			to: coordinate_to_index(&coordinates[2..4]),
		}
	}

	pub fn to_coordinates(&self) -> String {
		let promotion = if PROMOTABLE.contains(&self.flag) {
				piece_to_char(self.flag as usize).to_string()
			} else {
				String::new()
			};

		format!("{}{}{}",
			index_to_coordinate(self.from),
			index_to_coordinate(self.to),
			promotion,
		)
	}
}