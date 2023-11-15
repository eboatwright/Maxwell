use crate::Board;
use std::cmp::{max, min};
use crate::piece::*;

pub struct Maxwell<'a> {
	pub playing_white: bool,

	pub board: &'a mut Board,
	pub move_to_play: u32,
}

impl<'a> Maxwell<'a> {
	pub fn new(playing_white: bool, board: &'a mut Board) -> Self {
		Self {
			playing_white,

			board,
			move_to_play: 0,
		}
	}

	pub fn search_moves(&mut self, depth: i8, mut alpha: i32, beta: i32) -> i32 {
		let legal_moves = self.board.get_legal_moves_for_color(self.board.whites_turn);

		if legal_moves.len() == 0 {
			if self.board.king_in_check(self.board.whites_turn) {
				return -99999999 - depth as i32;
			}
			return 0;
		}

		if depth == 0 {
			return self.board.evaluate();
		}

		let mut best_move = 0;

		for i in 0..legal_moves.len() {
			self.board.make_move(legal_moves[i]);

			let eval_after_move = -self.search_moves(depth - 1, -beta, -alpha);

			self.board.undo_last_move();

			if eval_after_move >= beta {
				return beta;
			}

			if eval_after_move > alpha {
				best_move = i;
				alpha = eval_after_move;
			}
		}

		self.move_to_play = legal_moves[best_move];
		alpha
	}
}