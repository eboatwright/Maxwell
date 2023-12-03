use std::time::Instant;
use crate::opening_repertoire::OPENING_REPERTOIRE;
use macroquad::rand::{srand, gen_range};
use crate::utils::*;
use crate::board::*;
use std::cmp::{max, min};
use crate::piece::*;

#[derive(PartialEq)]
pub enum MaxwellPlaying {
	None,
	White,
	Black,
	Both,
}

pub const MAXWELL_PLAYING: MaxwellPlaying = MaxwellPlaying::Black;
const MAXWELL_THINKING_TIME: f32 = 10.0;
const MAX_SEARCH_EXTENSIONS: usize = 16;

pub struct Maxwell {
	pub best_move: u32,
	pub best_move_this_iteration: u32,

	pub evaluation: i32,

	pub in_opening: bool,
	pub positions_searched: u128,

	pub turn_timer: Instant,
	pub cancelled_search: bool,
}

impl Maxwell {
	pub fn new() -> Self {
		Self {
			best_move: 0,
			best_move_this_iteration: 0,

			evaluation: 0,

			in_opening: true,
			positions_searched: 0,

			cancelled_search: false,
			turn_timer: Instant::now(),
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

			if OPENING_REPERTOIRE.contains(&board.current_zobrist_key) {
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



	pub fn get_sorted_moves(&mut self, board: &mut Board, depth: u16) -> Vec<u32> {
		let legal_moves = board.get_legal_moves_for_color(board.whites_turn);
		if legal_moves.is_empty() {
			return vec![];
		}

		let num_of_moves = legal_moves.len();
		let mut scores = vec![(0, 0); num_of_moves];

		let endgame = board.endgame_multiplier();
		let squares_opponent_attacks = board.attacked_squares_bitboards[!board.whites_turn as usize];

		let hash_move = if depth == 0 {
			self.best_move
		} else {
			if let Some(data) = board.transposition_table.get(&board.current_zobrist_key) {
				data.best_move
			} else {
				0
			}
		};


		for i in 0..num_of_moves {
			let m = legal_moves[i];

			let mut score = 0;

			if m == hash_move {
				score = 9999;
			} else {
				let move_flag = get_move_flag(m);
				let move_from = get_move_from(m);
				let move_to = get_move_to(m);

				let moved_piece = board.board[move_from];
				let captured_piece = board.board[move_to];


				if captured_piece != 0 {
					score += 10 * get_full_piece_worth(captured_piece, move_to, endgame) - get_full_piece_worth(moved_piece, move_from, endgame);
				}

				if squares_opponent_attacks & (1 << move_to) != 0 {
					score -= get_full_piece_worth(moved_piece, move_to, endgame);
				}

				if PROMOTABLE_PIECES.contains(&move_flag) {
					score += 5 * get_full_piece_worth(move_flag, move_to, endgame);
				}
			}


			scores[i] = (score, i);
		}

		// TODO: this can definitely be improved :/
		scores.sort_by(|a, b| b.0.cmp(&a.0));

		let mut ordered = vec![0; num_of_moves];
		for i in 0..num_of_moves {
			ordered[i] = legal_moves[scores[i].1];
		}

		ordered
	}


	pub fn search_only_captures(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
		if self.cancelled_search
		|| self.turn_timer.elapsed().as_secs_f32() >= MAXWELL_THINKING_TIME {
			self.cancelled_search = true;
			return 0;
		}


		self.positions_searched += 1;


		if !board.checkmating_material_on_board() {
			return 0;
		}


		let evaluation = board.evaluate();
		if evaluation >= beta {
			return beta;
		}

		if evaluation > alpha {
			alpha = evaluation;
		}


		let legal_moves = board.get_legal_captures_for_color(board.whites_turn);

		for m in legal_moves {
			board.make_move(m);

			let eval_after_move = -self.search_only_captures(board, -beta, -alpha);

			board.undo_last_move();

			if eval_after_move >= beta {
				return beta;
			}

			if eval_after_move > alpha {
				alpha = eval_after_move;
			}
		}

		return alpha;
	}


	pub fn search_moves(
		&mut self,
		board: &mut Board,
		mut depth_left: u16,
		depth: u16,
		number_of_extensions: u16,
		mut alpha: i32,
		beta: i32,
	) -> i32 {
		if self.cancelled_search
		|| self.turn_timer.elapsed().as_secs_f32() >= MAXWELL_THINKING_TIME {
			self.cancelled_search = true;
			return 0;
		}

		self.positions_searched += 1;

		if board.current_fifty_move_draw == 100
		|| board.is_repetition()
		|| !board.checkmating_material_on_board() {
			return 0;
		}

		let legal_moves = self.get_sorted_moves(board, depth);

		if legal_moves.is_empty() {
			if board.king_in_check(board.whites_turn) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		if let Some(data) = board.lookup_transposition(depth_left, alpha, beta) {
			self.positions_searched -= 1;

			if depth == 0 {
				self.best_move_this_iteration = data.best_move;
			}

			return data.evaluation;
		}

		if depth_left == 0 {
			return self.search_only_captures(board, alpha, beta);
		}

		// Razoring :D
		if depth_left == 3
		&& depth != 0 {
			let eval = board.evaluate();
			if eval + QUEEN_WORTH < alpha {
				// return self.search_only_captures(board, alpha, beta);
				depth_left -= 1;
			}
		}

		let mut best_move_this_search = 0;
		let mut best_move_depth_searched_at = depth_left;
		let mut node_type = NodeType::UpperBound;

		for m in legal_moves {
			board.make_move(m);

			let mut search_extension = 0;
			if number_of_extensions < MAX_SEARCH_EXTENSIONS as u16 {
				if board.king_in_check(board.whites_turn) {
					search_extension += 1;
				} else {
					let to = get_move_to(m);
					if get_piece_type(board.board[to]) == PAWN {
						let rank = to / 8;
						if rank == 1 || rank == 7 {
							search_extension += 1;
						}
					}
				}
			}
			let total_depth_left = depth_left + search_extension;

			let eval_after_move = -self.search_moves(board, total_depth_left - 1, depth + 1, number_of_extensions + search_extension, -beta, -alpha);

			board.undo_last_move();

			if self.cancelled_search {
				return 0;
			}

			if eval_after_move >= beta {
				board.store_transposition(total_depth_left, beta, m, NodeType::LowerBound);
				return beta;
			}

			if eval_after_move > alpha {
				node_type = NodeType::Exact;
				best_move_this_search = m;
				best_move_depth_searched_at = total_depth_left;
				alpha = eval_after_move;

				if depth == 0 {
					self.best_move_this_iteration = best_move_this_search;
					self.evaluation = eval_after_move;
				}
			}
		}

		if best_move_this_search != 0 {
			board.store_transposition(best_move_depth_searched_at, alpha, best_move_this_search, node_type);
		}

		alpha
	}

	pub fn start(&mut self, board: &mut Board) {
		self.best_move = 0;
		self.best_move_this_iteration = 0;
		self.evaluation = 0;
		self.cancelled_search = false;
		self.positions_searched = 0;


		if self.in_opening {
			srand(macroquad::miniquad::date::now() as u64); // to randomize openings :D
			let opening_move = self.get_opening_move(board);
			if opening_move != 0 {
				self.best_move = opening_move;
				println!("Book move\n");
				return;
			}
		}


		// board.transposition_table.clear();


		self.turn_timer = Instant::now();

		for depth in 1..256 {
			println!("Searching depth {}...", depth);

			self.best_move_this_iteration = 0;

			let evaluation_this_search = self.search_moves(board, depth as u16, 0, 0, -i32::MAX, i32::MAX);
			if self.best_move_this_iteration != 0 {
				self.best_move = self.best_move_this_iteration;
				self.evaluation = evaluation_this_search;
			}


			if self.cancelled_search {
				println!("Search cancelled\n\n\n");
				break;
			}


			println!("Time since start of turn: {}", self.turn_timer.elapsed().as_secs_f32());
			println!("Positions searched: {}", self.positions_searched);

			let evaluation = self.evaluation * (if board.whites_turn { 1 } else { -1 });

			if evaluation_is_mate(evaluation) {
				let sign = if evaluation < 0 { "-" } else { "" };
				println!("Final evaluation: {}#{}\n\n\n", sign, moves_from_mate(evaluation));
				break;
			}
			println!("Final evaluation: {}", evaluation as f32 * 0.01);


			println!("\n");
		}


		if self.best_move == 0 {
			self.best_move = board.get_legal_moves_for_color(board.whites_turn)[0];
			println!("Could not search in time, defaulting to first legal move :(\n\n\n");
		}


		let size_of_entry = std::mem::size_of::<u64>() + std::mem::size_of::<TranspositionData>();

		let size: usize = board.transposition_table.capacity() * size_of_entry;
		println!("Transposition table size before filter: {} MB\n", size as f32 / 1_000_000.0);

		let timer = Instant::now();

		board.update_transposition_table();

		println!("Time to filter table: {}\n", timer.elapsed().as_secs_f32());

		let size: usize = board.transposition_table.capacity() * size_of_entry;
		println!("Transposition table size after filter: {} MB\n\n\n", size as f32 / 1_000_000.0);
	}
}