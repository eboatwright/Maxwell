use crate::utils::generate_board_from_fen;

pub struct Board {
	pub board: [u8; 64],
}

impl Board {
	pub fn new(fen: &'static str) -> Self {
		Self {
			board: generate_board_from_fen(fen),
		}
	}
}