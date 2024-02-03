/*
Rustic, Coding Adventure Bot and Weiawaga were very helpful resources while developing this TT
Thanks for the inspiration!
*/

use std::collections::HashMap;
use crate::utils::evaluation_is_mate;
use crate::move_data::MoveData;
use std::mem::size_of;

pub const MEGABYTE: usize = 1024 * 1024;
pub const ENTRY_SIZE: usize = size_of::<Option<TranspositionData>>();

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EvalBound {
	UpperBound,
	LowerBound,
	Exact,
}

#[derive(Copy, Clone)]
pub struct TranspositionData {
	pub key: u64,
	pub depth: u8,
	pub evaluation: i32,
	pub best_move: u16,
	pub eval_bound: EvalBound,
}

pub struct TranspositionTable {
	size_in_mb: usize,
	entries: usize,
	length: usize,
	// Still not sure if I should use a Vec<Option> or HashMap, but I've been tipped to use Vec so I'll do that :>
	pub table: Vec<Option<TranspositionData>>,

	pub hits: u128,
}

impl TranspositionTable {
	pub fn empty(size_in_mb: usize) -> Self {
		let length = (size_in_mb * MEGABYTE) / ENTRY_SIZE;

		Self {
			size_in_mb,
			entries: 0,
			length,
			table: vec![None; length],

			hits: 0,
		}
	}

	pub fn get_index(&self, key: u64) -> usize { (key as usize) % self.length }

	pub fn store(&mut self, key: u64, depth: u8, ply: u8, evaluation: i32, best_move: MoveData, eval_bound: EvalBound) {
		if self.length == 0 {
			return;
		}

		let mut fixed_mate_evaluation = evaluation;
		if evaluation_is_mate(evaluation) {
			let sign = if evaluation > 0 { 1 } else { -1 };
			fixed_mate_evaluation = (evaluation * sign + ply as i32) * sign;
		}

		let index = self.get_index(key);

		if let Some(data) = self.table[index] {
			// if (data.age - self.halfmove_clock).abs() > 10 {
			// 	return;
			// }

			// If we already have a deeper depth, then we don't care about the shallower
			// data so just return
			if data.depth > depth {
				return;
			}

			// If the new data and the old data were at the exact same depth, prefer an exact evaluation
			if data.depth == depth
			&& (data.eval_bound == EvalBound::Exact
			|| eval_bound != EvalBound::Exact) {
				return;
			}
		} else {
			self.entries += 1;
		}

		self.table[index] =
			Some(TranspositionData {
				key,
				depth,
				evaluation: fixed_mate_evaluation,
				best_move: best_move.to_binary(),
				eval_bound,
				// age: 0,
			});
	}

	pub fn lookup(&mut self, key: u64, ply: u8, depth: u8, alpha: i32, beta: i32) -> (Option<i32>, Option<MoveData>) {
		if let Some(data) = self.table[self.get_index(key)] {
			if data.key == key {
				self.hits += 1;

				let mut return_evaluation = None;

				if data.depth >= depth {
					let mut fixed_mate_evaluation = data.evaluation;
					if evaluation_is_mate(data.evaluation) {
						let sign = if data.evaluation > 0 { 1 } else { -1 };
						fixed_mate_evaluation = (data.evaluation * sign - ply as i32) * sign;
					}

					match data.eval_bound {
						EvalBound::LowerBound => if fixed_mate_evaluation >= beta { return_evaluation = Some(beta); },
						EvalBound::UpperBound => if fixed_mate_evaluation <= alpha { return_evaluation = Some(alpha); },
						EvalBound::Exact => return_evaluation = Some(fixed_mate_evaluation),
					}
				}

				return (return_evaluation, Some(MoveData::from_binary(data.best_move)));
			}
		}

		(None, None)
	}

	pub fn print_size(&self) {
		let size = (self.entries * ENTRY_SIZE) as f32 / MEGABYTE as f32;
		println!("Transposition table size: {} MB / {} MB", size, self.size_in_mb);
	}
}