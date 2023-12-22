use crate::piece_square_tables::QUEEN_WORTH;
use crate::PAWN;
use crate::NO_PIECE;
use crate::utils::{CHECKMATE_EVAL, evaluation_is_mate, moves_ply_from_mate};
use std::time::Instant;
use crate::move_sorter::MoveSorter;
use crate::transposition_table::{TranspositionTable, NodeType};
use crate::move_data::{MoveData, NULL_MOVE};
use crate::opening_book::OpeningBook;
use crate::Board;

pub const MAX_SEARCH_EXTENSIONS: u8 = 16;

pub struct Bot {
	time_to_think: f32,
	think_timer: Instant,
	pub search_cancelled: bool,
	searched_one_move: bool,

	opening_book: OpeningBook,
	in_opening: bool,

	move_sorter: MoveSorter,
	pub transposition_table: TranspositionTable,

	pub best_move: MoveData,
	best_move_this_iteration: MoveData,
	best_move_depth_searched_at: u8,

	evaluation: i32,
	evaluation_this_iteration: i32,

	positions_searched: u128,
	quiescence_searched: u128,
	transposition_hits: u128,
}

impl Bot {
	pub fn new(in_opening: bool) -> Self {
		Self {
			time_to_think: 0.0,
			think_timer: Instant::now(),
			search_cancelled: false,
			searched_one_move: false,

			opening_book: OpeningBook::create(),
			in_opening,

			move_sorter: MoveSorter::new(),
			transposition_table: TranspositionTable::empty(),

			best_move: NULL_MOVE,
			best_move_this_iteration: NULL_MOVE,
			best_move_depth_searched_at: 0,

			evaluation: 0,
			evaluation_this_iteration: 0,

			positions_searched: 0,
			quiescence_searched: 0,
			transposition_hits: 0,
		}
	}

	pub fn start(&mut self, board: &mut Board, moves: String, my_time: f32) {
		if self.in_opening {
			let opening_move = self.opening_book.get_opening_move(moves);
			if opening_move == NULL_MOVE {
				self.in_opening = false;
			} else {
				self.best_move = opening_move;
				return;
			}
		}

		let time_percentage = if board.moves.len() / 2 <= 6 {
			0.025
		} else {
			0.07
		};
		self.time_to_think = (my_time * time_percentage).clamp(0.5, 20.0);

		self.search_cancelled = false;

		let last_evaluation = self.evaluation;

		self.best_move = NULL_MOVE;
		self.evaluation = 0;

		self.positions_searched = 0;
		self.quiescence_searched = 0;
		self.transposition_hits = 0;

		self.move_sorter.clear();

		self.think_timer = Instant::now();
		for depth in 1..=(255 - MAX_SEARCH_EXTENSIONS) {
			self.searched_one_move = false;
			self.best_move_this_iteration = NULL_MOVE;
			self.evaluation_this_iteration = 0;


			let mut window = 40;
			loop {
				let (alpha, beta) = (last_evaluation - window, last_evaluation + window);

				let evaluation = self.alpha_beta_search(board, 0, depth, alpha, beta, 0);

				if alpha < evaluation && evaluation < beta {
					break;
				}

				window *= 4;
			}


			if !self.search_cancelled
			|| self.searched_one_move {
				self.best_move = self.best_move_this_iteration;
				self.evaluation = self.evaluation_this_iteration;
			}

			println!("Depth: {}, Window: {}, Evaluation: {}, Best move: {}, Positions searched: {}, Quiescence positions searched: {}, Total: {}, Transposition Hits: {}",
				depth,
				window,
				self.evaluation * board.perspective(),
				self.best_move.to_coordinates(),
				self.positions_searched,
				self.quiescence_searched,
				self.positions_searched + self.quiescence_searched,
				self.transposition_hits,
			);

			if evaluation_is_mate(self.evaluation) {
				let moves_until_mate = moves_ply_from_mate(self.evaluation);
				if moves_until_mate <= depth {
					println!("Mate found in {}", (moves_until_mate as f32 * 0.5).ceil());
					break;
				}
			}

			if self.search_cancelled {
				println!("Search cancelled");
				break;
			}
		}

		self.transposition_table.update();
		self.transposition_table.print_size();
	}

	fn should_cancel_search(&mut self) -> bool {
		self.search_cancelled = self.search_cancelled || self.think_timer.elapsed().as_secs_f32() >= self.time_to_think;
		self.search_cancelled
	}

	fn alpha_beta_search(
		&mut self,
		board: &mut Board,
		depth: u8,
		mut depth_left: u8,
		mut alpha: i32,
		beta: i32,
		number_of_extensions: u8,
	) -> i32 {
		if self.should_cancel_search() {
			return 0;
		}

		self.positions_searched += 1;

		if depth > 0 {
			if board.zobrist.is_repetition() {
				// This is to discourage making draws in winning positions
				// if it really is an equal position, it will still return 0
				return -self.quiescence_search(board, alpha, beta);
			}

			if board.insufficient_checkmating_material() {
				return 0;
			}
		}

		if let Some(data) = self.transposition_table.lookup(board.zobrist.key, depth_left, depth, alpha, beta) {
			self.positions_searched -= 1;
			self.transposition_hits += 1;

			if depth == 0 {
				self.best_move_this_iteration = data.best_move;
				self.best_move_depth_searched_at = data.depth_left;
				self.evaluation_this_iteration = data.evaluation;
			}

			return data.evaluation;
		}

		// Razoring
		if depth_left == 3
		&& depth != 0
		&& board.get_last_move().capture == NO_PIECE as u8
		&& !board.king_in_check(board.white_to_move) {
			let eval = board.evaluate();
			if eval + QUEEN_WORTH < alpha {
				depth_left -= 1;
			}
		}

		if depth_left == 0 {
			return self.quiescence_search(board, alpha, beta);
		}

		let mut best_move_this_search = NULL_MOVE;
		let mut node_type = NodeType::UpperBound;

		let legal_moves = board.get_legal_moves_for_color(board.white_to_move, false);
		if legal_moves.is_empty() {
			if board.king_in_check(board.white_to_move) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		let hash_move =
			if depth == 0 {
				self.best_move
			} else if let Some(data) = self.transposition_table.table.get(&board.zobrist.key) {
				data.best_move
			} else {
				NULL_MOVE
			};

		let sorted_moves = self.move_sorter.sort_moves(board, legal_moves, hash_move, depth);

		for i in 0..sorted_moves.len() {
			let m = sorted_moves[i];
			board.make_move(m);

			let mut search_extension = 0;
			if number_of_extensions < MAX_SEARCH_EXTENSIONS as u8 {
				if board.king_in_check(board.white_to_move) {
					search_extension = 1;
				} else {
					if m.piece == PAWN as u8 {
						let rank = m.to / 8;
						if rank == 1 || rank == 6 {
							search_extension = 1;
						}
					}
				}
			}




			let mut evaluation = 0;
			let mut needs_full_search = true;

			if search_extension == 0
			&& depth_left >= 3
			&& i >= 3
			&& m.capture == NO_PIECE as u8 {
				evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 1 - 1, -alpha - 1, -alpha, number_of_extensions);
				needs_full_search = evaluation > alpha;
			}

			if needs_full_search {
				evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 1 + search_extension, -beta, -alpha, number_of_extensions + search_extension);
			}




			board.undo_last_move();

			if self.should_cancel_search() {
				return 0;
			}

			if evaluation >= beta {
				self.transposition_table.store(board.zobrist.key, depth_left, depth, beta, m, NodeType::LowerBound);

				if m.capture == NO_PIECE as u8 {
					self.move_sorter.push_killer_move(m, depth);
					self.move_sorter.history[m.piece as usize][m.to as usize] += (depth_left * depth_left) as i32;
				}

				return beta;
			}

			if evaluation > alpha {
				best_move_this_search = m;
				node_type = NodeType::Exact;
				alpha = evaluation;

				if depth == 0 {
					self.searched_one_move = true;
					self.best_move_this_iteration = best_move_this_search;
					self.best_move_depth_searched_at = depth_left;
					self.evaluation_this_iteration = evaluation;
				}
			}
		}

		if best_move_this_search != NULL_MOVE {
			self.transposition_table.store(board.zobrist.key, depth_left, depth, alpha, best_move_this_search, node_type);
		}

		alpha
	}

	fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
		if self.should_cancel_search() {
			return 0;
		}

		self.quiescence_searched += 1;

		let evaluation = board.evaluate();
		if evaluation >= beta {
			return beta;
		}

		if evaluation > alpha {
			alpha = evaluation;
		}

		let legal_moves = board.get_legal_moves_for_color(board.white_to_move, true);
		if legal_moves.is_empty() {
			return evaluation;
		}

		let sorted_moves = self.move_sorter.sort_moves(board, legal_moves, NULL_MOVE, 0);
		for m in sorted_moves {
			board.make_move(m);
			let evaluation = -self.quiescence_search(board, -beta, -alpha);
			board.undo_last_move();

			if evaluation >= beta {
				return beta;
			}

			if evaluation > alpha {
				alpha = evaluation;
			}
		}

		alpha
	}
}