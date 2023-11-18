use crate::utils::*;
use crate::board::*;
use std::cmp::{max, min};
use crate::piece::*;

pub struct Maxwell<'a> {
	pub board: &'a mut Board,
	pub move_to_play: u32,

	pub positions_searched: u128,
}

impl<'a> Maxwell<'a> {
	pub fn new(board: &'a mut Board) -> Self {
		Self {
			board,
			move_to_play: 0,

			positions_searched: 0,
		}
	}

	pub fn get_sorted_moves(&mut self) -> Vec<u32> {
		let legal_moves = self.board.get_legal_moves_for_color(self.board.whites_turn);
		if legal_moves.is_empty() {
			return vec![];
		}

		let num_of_moves = legal_moves.len();
		let mut scores = vec![(0, 0); num_of_moves];

		let potentially_weak_squares = self.board.attacked_squares_bitboards[!self.board.whites_turn as usize] & !self.board.attacked_squares_bitboards[self.board.whites_turn as usize];


		for i in 0..num_of_moves {
			let m = legal_moves[i];
			let mut score = 0;

			let move_flag = get_move_flag(m);
			let move_from = get_move_from(m);
			let move_to = get_move_to(m);

			let moved_piece_type = self.board.board[move_from];
			let captured_piece_type = self.board.board[move_to];


			if captured_piece_type != 0 {
				score += 15 * get_full_piece_worth(captured_piece_type, move_to) - get_full_piece_worth(moved_piece_type, move_from);
			}

			if potentially_weak_squares & (1 << move_to) != 0 {
				score -= get_full_piece_worth(moved_piece_type, move_to);
			}

			if self.board.fullmove_counter >= 4 // Promotions can't occur early in the game, so don't bother checking if it's still the opening
			&& PROMOTABLE_PIECES.contains(&move_flag) {
				score += get_full_piece_worth(move_flag, move_to);
			}


			scores[i] = (score, i);
		}

		scores.sort_by(|a, b| b.0.cmp(&a.0));

		let mut ordered = vec![0; num_of_moves];
		for i in 0..num_of_moves {
			ordered[i] = legal_moves[scores[i].1];
		}

		ordered
	}

	pub fn search_moves(&mut self, depth_left: u16, depth: u16, mut alpha: i32, beta: i32) -> i32 {
		let legal_moves = self.get_sorted_moves();

		if legal_moves.is_empty() {
			self.positions_searched += 1;
			if self.board.king_in_check(self.board.whites_turn) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		let zobrist_key = self.board.current_zobrist_key();
		if let Some(transposition) = self.board.transposition_table.get(&zobrist_key) {
			if transposition.depth == depth {
				return transposition.evaluation;
			}
		}

		self.positions_searched += 1;

		if depth_left == 0 {
			return self.board.evaluate();
		}

		for m in legal_moves {
			self.board.make_move(m);

			let eval_after_move = -self.search_moves(depth_left - 1, depth + 1, -beta, -alpha);

			self.board.undo_last_move();

			if eval_after_move >= beta {
				return beta;
			}

			if eval_after_move > alpha {
				alpha = eval_after_move;
			}
		}

		self.board.transposition_table.insert(zobrist_key,
			TranspositionData {
				depth,
				evaluation: alpha,
			});

		alpha
	}

	pub fn start_search(&mut self, depth: u16) -> i32 {
		// self.board.transposition_table.clear(); // ?

		let mut alpha = -i32::MAX;
		let beta = i32::MAX;

		let legal_moves = self.get_sorted_moves();

		for m in legal_moves {
			self.board.make_move(m);

			let eval_after_move = -self.search_moves(depth - 1, 1, -beta, -alpha);

			self.board.undo_last_move();

			if eval_after_move > alpha {
				alpha = eval_after_move;
				self.move_to_play = m;
			}
		}

		alpha
	}
}