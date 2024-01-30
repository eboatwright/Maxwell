// NOT BEING USED

use crate::MoveData;

pub struct MoveList {
	pub moves: Vec<MoveData>,
	pub scores: Vec<i32>,
	pub moves_tried: Vec<bool>,
}

impl MoveList {
	pub fn new() -> Self {
		Self {
			scores: vec![],
			moves: vec![],
			moves_tried: vec![],
		}
	}

	pub fn len(&self) -> usize { self.moves.len() }

	pub fn push(&mut self, move_data: MoveData, score: i32) {
		self.moves.push(move_data);
		self.scores.push(score);
		self.moves_tried.push(false);
	}

	pub fn get_next_move(&mut self) -> MoveData {
		let mut best_move_i = 0;
		let mut best_score = self.scores[0];

		for i in 0..self.moves.len() {
			if self.moves_tried[i] {
				continue;
			}

			let this_move_score = self.scores[i];
			if this_move_score > best_score {
				best_move_i = i;
				best_score = this_move_score;
			}
		}

		self.moves_tried[best_move_i] = true;
		self.moves[best_move_i]
	}
}