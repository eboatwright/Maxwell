// CURRENTLY NOT BEING USED

use crate::move_data::{MoveData, NULL_MOVE};
use crate::move_sorter::MAX_SORT_MOVE_PLY;

pub struct PVTable {
	pub length: [usize; MAX_SORT_MOVE_PLY],
	pub table: [[MoveData; MAX_SORT_MOVE_PLY]; MAX_SORT_MOVE_PLY],
}

impl PVTable {
	pub fn new() -> Self {
		Self {
			length: [0; MAX_SORT_MOVE_PLY],
			table: [[NULL_MOVE; MAX_SORT_MOVE_PLY]; MAX_SORT_MOVE_PLY],
		}
	}

	pub fn print(&self) {
		for i in 0..MAX_SORT_MOVE_PLY {
			if self.length[i] == 0 {
				break;
			}

			for j in 0..self.length[i] {
				print!("{} ", self.table[i][j].to_coordinates());
			}
			println!();
		}
		println!();
	}

	pub fn set_pv_length(&mut self, ply: usize) {
		if ply < MAX_SORT_MOVE_PLY {
			self.length[ply] = ply;
		}
	}

	pub fn push_pv_move(&mut self, data: MoveData, ply: usize) {
		if ply + 1 < MAX_SORT_MOVE_PLY {
			self.table[ply][ply] = data;

			for next_depth in (ply + 1)..self.length[ply + 1] {
				self.table[ply][next_depth] = self.table[ply + 1][next_depth];
			}

			self.length[ply] = self.length[ply + 1];
		}
	}

	pub fn pop(&mut self) {
		self.table[0][0] = NULL_MOVE;
		self.table[0].rotate_left(1);
		self.length[0] -= 1;
	}

	pub fn get_pv_move(&self, ply: usize) -> MoveData {
		self.table[ply][ply]
	}
}