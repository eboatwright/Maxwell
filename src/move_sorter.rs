use crate::killer_moves::KillerMoves;
use crate::move_data::{MoveData, NULL_MOVE};
use crate::pieces::*;
use crate::Board;

pub const MAX_SORT_MOVE_PLY: usize = 32;

pub const MVV_LVA: [i32; 36] = [
	15, 25, 35, 45, 55, 65, // Pawn
	14, 24, 34, 44, 54, 64, // Knight
	13, 23, 33, 43, 53, 63, // Bishop
	12, 22, 32, 42, 52, 62, // Rook
	11, 21, 31, 41, 51, 61, // Queen
	10, 20, 30, 40, 50, 60, // King
];

pub struct MoveSorter {
	pub killer_moves: [KillerMoves; MAX_SORT_MOVE_PLY],
	pub history: [[[i32; 64]; 64]; 2],
}

impl MoveSorter {
	pub fn new() -> Self {
		Self {
			killer_moves: [KillerMoves::new(); MAX_SORT_MOVE_PLY],
			history: [[[0; 64]; 64]; 2],
		}
	}

	pub fn clear(&mut self) {
		self.killer_moves = [KillerMoves::new(); MAX_SORT_MOVE_PLY];
		self.history = [[[0; 64]; 64]; 2];
	}

	pub fn add_killer_move(&mut self, data: MoveData, ply: usize) {
		if ply < MAX_SORT_MOVE_PLY {
			self.killer_moves[ply].add_killer_move(data);
		}
	}

	pub fn sort_moves(&mut self, white_to_move: bool, moves: Vec<MoveData>, hash_move: MoveData, ply: usize) -> Vec<(i32, MoveData)> {
		if moves.is_empty() {
			return vec![];
		}

		let mut pairs = vec![];

		for m in moves {
			let mut score = 0;

			if m == hash_move {
				score = i32::MAX;
			} else {
				if m.capture == NO_PIECE as u8 {
					if ply < MAX_SORT_MOVE_PLY
					&& self.killer_moves[ply].is_killer(m) {
						score += 5000;
					}

					score += self.history[white_to_move as usize][m.from as usize][m.to as usize];
				} else {
					score += 8000 + MVV_LVA[get_piece_type(m.piece as usize) * 6 + get_piece_type(m.capture as usize)];

					// TODO: static exchange evaluation
				}

				// TODO: re-test this
				// if m.flag == SHORT_CASTLE_FLAG
				// || m.flag == LONG_CASTLE_FLAG {
				// 	score += 2000;
				// } else if PROMOTABLE.contains(&m.flag) {
				// 	score += BASE_WORTHS_OF_PIECE_TYPE[m.flag as usize] + 12000;
				// }
			}

			pairs.push((score, m));
		}

		pairs.sort_by(|a, b| b.0.cmp(&a.0));
		pairs
	}
}