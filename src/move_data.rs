use crate::pieces::{char_to_piece, get_piece_type};
use crate::pieces::{PROMOTABLE, piece_to_char, NO_PIECE};
use crate::utils::{coordinate_to_index, SQUARE_COORDINATES};

// 1, 2, 3, 4 for promoting pieces
pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 5;
pub const EN_PASSANT_FLAG: u8       = 6;
pub const SHORT_CASTLE_FLAG: u8     = 7;
pub const LONG_CASTLE_FLAG: u8      = 8;

pub const NULL_MOVE: MoveData = MoveData {
	flag: 0,
	capture: NO_PIECE as u8,
	piece: NO_PIECE as u8,
	from: 0,
	to: 0,
};

#[derive(Copy, Clone, Debug)]
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

impl PartialEq for MoveData {
	fn eq(&self, other: &MoveData) -> bool {
		   self.flag == other.flag
		&& self.from == other.from
		&& self.to == other.to
	}
}

impl MoveData {
	pub fn from_coordinates(coordinates: String) -> Self {
		Self {
			flag: if coordinates.len() > 4 {
				get_piece_type(char_to_piece(coordinates.chars().nth(4).unwrap())) as u8
			} else {
				0
			},
			capture: NO_PIECE as u8,
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
			SQUARE_COORDINATES[self.from as usize],
			SQUARE_COORDINATES[self.to as usize],
			promotion,
		)
	}

	/*
	flag from   to
	0000 000000 000000
	*/

	pub fn from_binary(binary: u16) -> MoveData {
		MoveData {
			flag:    ((binary & 0b_1111_000000_000000) >> 12) as u8,
			capture: NO_PIECE as u8,
			piece:   NO_PIECE as u8,
			from:    ((binary & 0b_0000_111111_000000) >> 6) as u8,
			to:      (binary &  0b_0000_000000_111111) as u8,
		}
	}

	pub fn to_binary(&self) -> u16 {
		let mut result: u16 = 0b_0000_000000_000000;

		result |= (self.flag as u16) << 12;
		result |= (self.from as u16) << 6;
		result |= self.to as u16;

		result
	}
}