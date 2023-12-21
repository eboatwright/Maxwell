use crate::move_data::{MoveData, NULL_MOVE};

#[derive(Copy, Clone)]
pub struct KillerMoves {
	pub a: MoveData,
	pub b: MoveData,
}

impl KillerMoves {
	pub fn new() -> Self {
		Self {
			a: NULL_MOVE,
			b: NULL_MOVE,
		}
	}

	pub fn push(&mut self, new_move: MoveData) {
		self.b = self.a;
		self.a = new_move;
	}

	pub fn is_killer(&self, check: MoveData) -> bool {
			check == self.a
		 || check == self.b
	}
}