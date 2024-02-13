// NOT BEING USED

use crate::MoveData;

pub struct ScoredMoveList {
	pub pairs: Vec<(i32, MoveData)>,
}

impl ScoredMoveList {
	pub fn new() -> Self {
		Self {
			pairs: vec![],
		}
	}

	pub fn len(&self) -> usize { self.pairs.len() }

	pub fn push(&mut self, score: i32, move_data: MoveData) {
		self.pairs.push((score, move_data));
	}

	pub fn get(&mut self, move_index: usize) -> MoveData {
		for i in (move_index + 1)..self.pairs.len() {
			if self.pairs[i].0 > self.pairs[move_index].0 {
				self.pairs.swap(move_index, i);
			}
		}

		self.pairs[move_index].1
	}
}