use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::move_data::NULL_MOVE;
use crate::MoveData;

pub struct OpeningBook {
	pub lines: Vec<&'static str>,
}

impl OpeningBook {
	pub fn create() -> Self {
		Self {
			lines: vec![
				"e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f8c5 e1g1 g8f6",
				"e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f8c5 c2c3 g8f6 d2d3",
				"e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f8c5 c2c3 g8f6 d2d4 e5d4 c3d4 c5b4",

				"e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5 d8e7 d1e2 f6d5",

				// Ruy Lopez
				"e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6",
				"e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 b7b5 a4b3 g7g6 e1g1 f8g7",

				"d2d4 d7d5 b1c3 g8f6 c1f4 c8f5",
				"d2d4 d7d5 b1c3 g8f6 c1g5 b8d7 g1f3 g7g6 e2e3 f8g7",

				"d2d4 d7d5 c2c4 e7e6 b1c3 c7c6 c1f4 d5c4 e2e3 b7b5 a2a4 d8b6 g1f3 g8f6",

				"d2d4 d7d5 c2c4 c7c6 b1c3 d5c4 e2e4 b7b5 a2a4 d8b6",
				"d2d4 d7d5 c2c4 c7c6 g1f3 d5c4 e2e3 c8e6 d1c2 b7b5",

				"Xc2c4 e7e5 b1c3 g8f6",

				"e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 b8c6 f1c4 e7e6 c1e3 c8d7 d1e2 a7a6",
				"e2e4 c7c5 Xb1c3 b8c6 f1b5 c6d4 b5c4 a7a6 g1f3 b7b5 c4e2 c8b7 e1g1 e7e6 d2d3 d4e2 d1e2",
			],
		}
	}

	pub fn get_opening_move(&mut self, moves: String) -> MoveData {
		let moves_played = if moves.is_empty() { 0 } else { moves.matches(' ').count() + 1 };
		let mut opening_moves = vec![];

		for line in self.lines.iter() {
			let line_without_xs = line.replace('X', "");
			if line_without_xs.starts_with(&moves) {
				let start = moves.len() + if moves_played == 0 { 0 } else { 1 };

				if start >= line.len() {
					continue;
				}

				let next_move = line.split(' ').collect::<Vec<&str>>()[moves_played];

				if next_move.contains('X') {
					continue;
				}

				let data = MoveData::from_coordinates(next_move.to_string());
				opening_moves.push(data);
			}
		}

		if let Some(m) = opening_moves.choose(&mut thread_rng()) {
			return *m;
		}

		NULL_MOVE
	}
}