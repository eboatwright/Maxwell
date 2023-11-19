use crate::opening_repertoire::OPENING_REPERTOIRE;
use macroquad::rand::gen_range;
use crate::utils::*;
use crate::board::*;
use std::cmp::{max, min};
use crate::piece::*;

pub struct Maxwell {
	pub move_to_play: u32,
	pub evaluation: i32,
	pub in_opening: bool,

	pub positions_searched: u128,
}

impl Maxwell {
	pub fn new() -> Self {
		Self {
			move_to_play: 0,
			evaluation: 0,
			in_opening: true,

			positions_searched: 0,
		}
	}



	pub fn get_opening_move(&mut self, board: &mut Board) -> u32 {
		let legal_moves = board.get_legal_moves_for_color(board.whites_turn);

		if legal_moves.is_empty() {
			return 0;
		}

		let mut moves = vec![];

		for m in legal_moves {
			board.make_move(m);

			let zobrist_key = board.current_zobrist_key();
			if OPENING_REPERTOIRE.contains(&zobrist_key) {
				moves.push(m);
			}

			board.undo_last_move();
		}

		if moves.is_empty() {
			self.in_opening = false;
			return 0;
		}
		moves[gen_range(0, moves.len())]
	}



	pub fn get_sorted_moves(&mut self, board: &mut Board) -> Vec<u32> {
		let legal_moves = board.get_legal_moves_for_color(board.whites_turn);
		if legal_moves.is_empty() {
			return vec![];
		}

		let num_of_moves = legal_moves.len();
		let mut scores = vec![(0, 0); num_of_moves];

		let potentially_weak_squares = board.attacked_squares_bitboards[!board.whites_turn as usize] & !board.attacked_squares_bitboards[board.whites_turn as usize];


		for i in 0..num_of_moves {
			let m = legal_moves[i];
			let mut score = 0;

			let move_flag = get_move_flag(m);
			let move_from = get_move_from(m);
			let move_to = get_move_to(m);

			let moved_piece_type = board.board[move_from];
			let captured_piece_type = board.board[move_to];


			if captured_piece_type != 0 {
				score += 15 * get_full_piece_worth(captured_piece_type, move_to) - get_full_piece_worth(moved_piece_type, move_from);
			}

			if potentially_weak_squares & (1 << move_to) != 0 {
				score -= get_full_piece_worth(moved_piece_type, move_to);
			}

			if board.moves.len() >= 8 // Promotions can't occur early in the game, so don't bother checking if it's still the opening
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

	pub fn search_moves(&mut self, board: &mut Board, depth_left: u16, depth: u16, mut alpha: i32, beta: i32) -> i32 {
		self.positions_searched += 1;

		if board.fifty_move_draw() == 100 {
			return 0;
		}

		let legal_moves = self.get_sorted_moves(board);

		if legal_moves.is_empty() {
			if board.king_in_check(board.whites_turn) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		let zobrist_key = board.current_zobrist_key();
		if let Some(evaluation) = board.transposition_table.get(&zobrist_key) {
			self.positions_searched -= 1;
			return *evaluation;
		}

		if depth_left == 0 {
			if let Some(evaluation) = board.evaluation_cache.get(&zobrist_key) {
				return *evaluation;
			}
			let evaluation = board.evaluate();
			board.evaluation_cache.insert(zobrist_key, evaluation);
			return evaluation;
		}

		let mut best_move = 0;

		for m in legal_moves {
			board.make_move(m);

			let eval_after_move = -self.search_moves(board, depth_left - 1, depth + 1, -beta, -alpha);

			board.undo_last_move();

			if eval_after_move >= beta {
				return beta;
			}

			if eval_after_move > alpha {
				best_move = m;
				alpha = eval_after_move;
			}
		}

		board.transposition_table.insert(zobrist_key, alpha);

		self.move_to_play = best_move;

		alpha
	}

	pub fn start(&mut self, board: &mut Board) { // TODO: this needs to be reworked for iterative deepening
		self.move_to_play = 0;
		self.positions_searched = 0;
		self.evaluation = 0;


		if self.in_opening {
			let opening_move = self.get_opening_move(board);
			if opening_move != 0 {
				self.move_to_play = opening_move;
				return;
			}
		}



		board.transposition_table.clear(); // ?

		// for depth in 1..=8 {
			// println!("Searching depth {}", depth);
			self.evaluation = self.search_moves(board, 6, 0, -i32::MAX, i32::MAX);
		// }
	}
}