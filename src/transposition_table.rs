use crate::utils::evaluation_is_mate;
use crate::MoveData;
use std::collections::HashMap;

pub const ENTRY_SIZE: usize = std::mem::size_of::<u64>() + std::mem::size_of::<TranspositionData>();

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeType {
	UpperBound,
	LowerBound,
	Exact,
}

#[derive(Copy, Clone)]
pub struct TranspositionData {
	pub depth_left: u8,
	pub evaluation: i32,
	pub best_move: MoveData,
	pub age: u8,
	pub node_type: NodeType,
}

pub struct TranspositionTable {
	pub table: HashMap<u64, TranspositionData>,
}

impl TranspositionTable {
	pub fn empty() -> Self {
		Self {
			table: HashMap::new(),
		}
	}

	pub fn store(&mut self, key: u64, depth_left: u8, depth: u8, evaluation: i32, best_move: MoveData, node_type: NodeType) {
		let mut fixed_mate_evaluation = evaluation;
		if evaluation_is_mate(evaluation) {
			let sign = if evaluation > 0 { 1 } else { -1 };
			fixed_mate_evaluation = (evaluation * sign + depth as i32) * sign;
		}

		self.table.insert(key,
			TranspositionData {
				depth_left,
				evaluation: fixed_mate_evaluation,
				best_move,
				age: 0,
				node_type,
			}
		);
	}

	pub fn lookup(&mut self, key: u64, depth_left: u8, depth: u8, alpha: i32, beta: i32) -> Option<TranspositionData> {
		if let Some(data) = self.table.get_mut(&key) {
			if data.depth_left >= depth_left {
				let mut fixed_mate_evaluation = data.evaluation;
				if evaluation_is_mate(data.evaluation) {
					let sign = if data.evaluation > 0 { 1 } else { -1 };
					fixed_mate_evaluation = (data.evaluation * sign - depth as i32) * sign;
				}


				match data.node_type {
					NodeType::UpperBound => {
						if fixed_mate_evaluation <= alpha {
							data.age = 0;
							return Some(TranspositionData {
								evaluation: fixed_mate_evaluation,
								..*data
							});
						}
					}

					NodeType::LowerBound => {
						if fixed_mate_evaluation >= beta {
							data.age = 0;
							return Some(TranspositionData {
								evaluation: fixed_mate_evaluation,
								..*data
							});
						}
					}

					NodeType::Exact => {
						data.age = 0;
						return Some(TranspositionData {
							evaluation: fixed_mate_evaluation,
							..*data
						});
					}
				}
			}
		}
		None
	}

	pub fn update(&mut self) {
		self.table.retain(|_, data| {
			data.age += 1;
			data.age <= 10
		});
	}

	pub fn print_size(&self) {
		let length = (self.table.len() * ENTRY_SIZE) as f32 / 1_000_000.0;
		let capacity = (self.table.capacity() * ENTRY_SIZE) as f32 / 1_000_000.0;
		println!("Transposition table size: {} MB / {} MB", length, capacity);
	}
}