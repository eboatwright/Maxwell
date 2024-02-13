use crate::move_data::{MoveData, NULL_MOVE};

#[derive(Copy, Clone)]
pub struct KillerMoves {
	pub moves: [MoveData; 2],
}

impl KillerMoves {
	pub fn new() -> Self {
		Self {
			moves: [NULL_MOVE; 2],
		}
	}

	pub fn add_killer_move(&mut self, new_move: MoveData) {
		if self.moves[0] == new_move {
			return;
		}

		self.moves.rotate_right(1);
		self.moves[0] = new_move;
	}

	pub fn is_killer(&self, data: MoveData) -> bool {
		self.moves.contains(&data)
	}
}