use crate::utils::*;
use crate::Board;
use std::cmp::{max, min};
use crate::piece::*;

pub struct Maxwell<'a> {
	pub playing_white: bool,

	pub board: &'a mut Board,
	pub move_to_play: u32,

	pub positions_searched: u128,
}

impl<'a> Maxwell<'a> {
	pub fn new(playing_white: bool, board: &'a mut Board) -> Self {
		Self {
			playing_white,

			board,
			move_to_play: 0,

			positions_searched: 0,
		}
	}

	pub fn get_sorted_moves(&mut self) -> Vec<u32> {
		let legal_moves = self.board.get_legal_moves_for_color(self.board.whites_turn);
		let num_of_moves = legal_moves.len();
		let mut scores = vec![(0, 0); num_of_moves];

		for i in 0..num_of_moves {
			let m = legal_moves[i];
			let mut score = 0;

			let move_flag = get_move_flag(m);
			let move_from = get_move_from(m);
			let move_to = get_move_to(m);

			let moved_piece_type = self.board.board[move_from];
			let captured_piece_type = self.board.board[move_to];


			if captured_piece_type != 0 {
				score += 10 * get_full_piece_worth(captured_piece_type, move_to) - get_full_piece_worth(moved_piece_type, move_from);
			}

			if PROMOTABLE_PIECES.contains(&move_flag) {
				score += get_full_piece_worth(move_flag, move_to);
			}


			scores[i] = (score, i);
		}

		scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

		let mut sorted_moves = vec![0; num_of_moves];
		for i in 0..num_of_moves {
			sorted_moves[i] = legal_moves[scores[i].1];
		}

		sorted_moves
	}

	pub fn search_moves(&mut self, depth_left: i16, depth: i8, mut alpha: i32, beta: i32) -> i32 {
		self.positions_searched += 1;

		let legal_moves = self.get_sorted_moves();

		if legal_moves.len() == 0 {
			if self.board.king_in_check(self.board.whites_turn) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		if depth_left == 0 {
			return self.board.evaluate();
		}

		let mut best_move = 0;

		for i in 0..legal_moves.len() {
			self.board.make_move(legal_moves[i]);

			let eval_after_move = -self.search_moves(depth_left - 1, depth + 1, -beta, -alpha);

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