use crate::STARTING_FEN;
use crate::piece_square_tables::{PAWN_WORTH, QUEEN_WORTH};
use crate::pieces::{PAWN, PROMOTABLE, NO_PIECE};
use crate::utils::{CHECKMATE_EVAL, evaluation_is_mate, ply_from_mate};
use std::time::Instant;
use crate::move_sorter::MoveSorter;
use crate::transposition_table::{TranspositionTable, EvalBound};
use crate::move_data::{MoveData, NULL_MOVE};
use crate::opening_book::OpeningBook;
use crate::Board;

pub const MAX_SEARCH_EXTENSIONS: u8 = 16; // TODO

#[derive(Clone, Debug)]
pub struct BotConfig {
	pub fen: String,
	pub debug_output: bool,
	pub opening_book: bool,
	pub time_management: bool,
	pub tt_size_in_mb: usize,
}

impl BotConfig {
	pub fn from_args(args: Vec<String>) -> Self {
		let _true = "true".to_string();
		let _false = "false".to_string();

		Self { // This is so ugly lol
			fen: Self::get_arg_value(&args, "fen").unwrap_or(STARTING_FEN.to_string()),
			debug_output: Self::get_arg_value(&args, "debug_output").unwrap_or(_true.clone()) == _true,
			opening_book: Self::get_arg_value(&args, "opening_book").unwrap_or(_false.clone()) == _true,
			time_management: Self::get_arg_value(&args, "time_management").unwrap_or(_true.clone()) == _true,
			tt_size_in_mb: (Self::get_arg_value(&args, "tt_size").unwrap_or("256".to_string())).parse::<usize>().unwrap_or(256),
		}
	}

	fn get_arg_value(args: &Vec<String>, key: &'static str) -> Option<String> {
		for arg in args.iter() {
			if arg.contains(key) {
				return Some(arg[key.len() + 1..].to_string());
			}
		}

		None
	}
}

pub struct Bot {
	pub config: BotConfig,

	time_to_think: f32,
	think_timer: Instant,
	pub search_cancelled: bool,
	searched_one_move: bool,

	opening_book: OpeningBook,
	in_opening_book: bool,

	move_sorter: MoveSorter,
	pub transposition_table: TranspositionTable,

	pub best_move: MoveData,
	best_move_this_iteration: MoveData,

	evaluation: i32,
	evaluation_this_iteration: i32,

	positions_searched: u128,
	quiescence_searched: u128,
}

impl Bot {
	pub fn new(config: BotConfig) -> Self {
		Self {
			config: config.clone(),

			time_to_think: 0.0,
			think_timer: Instant::now(),
			search_cancelled: false,
			searched_one_move: false,

			opening_book: OpeningBook::create(),
			in_opening_book: config.opening_book,

			move_sorter: MoveSorter::new(),
			transposition_table: TranspositionTable::empty(config.tt_size_in_mb),

			best_move: NULL_MOVE,
			best_move_this_iteration: NULL_MOVE,

			evaluation: 0,
			evaluation_this_iteration: 0,

			positions_searched: 0,
			quiescence_searched: 0,
		}
	}

	pub fn println(&self, output: String) {
		if self.config.debug_output {
			println!("{}", output);
		}
	}

	pub fn start(&mut self, board: &mut Board, moves: String, my_time: f32, depth: u8) {
		if self.in_opening_book {
			let opening_move = self.opening_book.get_opening_move(moves);
			if opening_move == NULL_MOVE {
				self.in_opening_book = false;
			} else {
				self.best_move = opening_move;
				return;
			}
		}

		self.time_to_think =
			if self.config.time_management
			&& my_time > 0.0 {
				let time_percentage = if board.moves.len() / 2 <= 6 {
					0.025
				} else {
					0.07
				};

				(my_time * time_percentage).clamp(0.2, 20.0)
			} else {
				my_time
			};

		self.search_cancelled = false;

		let last_evaluation = self.evaluation;

		self.best_move = NULL_MOVE;
		self.evaluation = 0;

		self.positions_searched = 0;
		self.quiescence_searched = 0;
		self.transposition_table.hits = 0;

		self.move_sorter.clear();

		let mut window = 40;

		self.think_timer = Instant::now();
		for current_depth in 1..=depth {
			self.searched_one_move = false;
			self.best_move_this_iteration = NULL_MOVE;
			self.evaluation_this_iteration = 0;

			// TODO: work on aspiration windows
			loop {
				let (alpha, beta) = (last_evaluation - window, last_evaluation + window);

				let evaluation = self.alpha_beta_search(board, current_depth, 0, alpha, beta, 0);

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

			self.println(format!("Depth: {}, Window: {}, Evaluation: {}, Best move: {}, Positions searched: {} + Quiescence positions searched: {} = {}, Transposition Hits: {}",
				current_depth,
				window,
				self.evaluation * board.perspective(),
				self.best_move.to_coordinates(),
				self.positions_searched,
				self.quiescence_searched,
				self.positions_searched + self.quiescence_searched,
				self.transposition_table.hits,
			));

			if evaluation_is_mate(self.evaluation) {
				let moves_until_mate = ply_from_mate(self.evaluation);
				if moves_until_mate <= current_depth {
					self.println(format!("Mate found in {}", (moves_until_mate as f32 * 0.5).ceil()));
					break;
				}
			}

			if self.search_cancelled {
				self.println("Search cancelled".to_string());
				break;
			}
		}

		self.println(format!("{} seconds", self.think_timer.elapsed().as_secs_f32()));

		if self.config.debug_output {
			self.transposition_table.print_size();
		}
	}

	fn should_cancel_search(&mut self) -> bool {
		self.search_cancelled = (self.search_cancelled || self.think_timer.elapsed().as_secs_f32() >= self.time_to_think) && self.time_to_think > 0.0;
		self.search_cancelled
	}

	fn alpha_beta_search(
		&mut self,
		board: &mut Board,
		mut depth: u8,
		ply: u8,
		mut alpha: i32,
		beta: i32,
		total_extensions: u8,
	) -> i32 {
		// TODO: try moving this into the move loop and break instead of return 0?
		if self.should_cancel_search() {
			return 0;
		}

		self.positions_searched += 1;

		if ply > 0 {
			if board.is_draw() {
				// Should I use this to discourage making a draw in a winning position?
				// return -self.quiescence_search(board, alpha, beta);
				return 0;
			}

			// Mate Distance Pruning
			let mate_value = CHECKMATE_EVAL - ply as i32;
			let alpha = i32::max(alpha, -CHECKMATE_EVAL + ply as i32);
			let beta = i32::min(beta, CHECKMATE_EVAL - ply as i32);
			if alpha >= beta {
				return alpha;
			}
		}

		let (tt_eval, hash_move) = self.transposition_table.lookup(board.zobrist.key, ply, depth, alpha, beta);

		// We don't really want to return from the root node, because if a hash collision occurs (although very rare)
		// It will return an illegal move
		if ply > 0 {
			if let Some(tt_eval) = tt_eval {
				return tt_eval;
			}
		}

		// This detects a null / zero window search, which is used in non PV nodes
		// This will also never be true if ply == 0 because the bounds will never be zero at ply 0
		let not_pv = alpha == beta - 1;

		// TODO: Maybe allow these pruning techniques during PV nodes?
		// Or maybe just make them more aggressive to allow for more search
		// time on PV lines?
		if not_pv
		&& depth > 0
		&& board.get_last_move().capture == NO_PIECE as u8 // ?
		&& !board.king_in_check(board.white_to_move) {
			// TODO: move these around

			let static_eval = board.evaluate();

			// Null Move Pruning
			if depth > 2
			&& static_eval >= beta
			&& board.total_material_without_pawns > 0
			&& board.try_null_move() {
				let evaluation = -self.alpha_beta_search(board, depth - 3, ply + 1, -beta, -beta + 1, total_extensions);

				board.undo_null_move();

				if evaluation >= beta {
					return evaluation;
				}
			}

			// Reverse Futility Pruning
			if depth < 5
			&& static_eval - 60 * (depth as i32) >= beta { // TODO: tweak this threshold
				return static_eval;
			}

			// Razoring
			if depth < 4
			&& static_eval + 300 * (depth as i32) < alpha {
				depth -= 1;
			}
		}

		if depth == 0 {
			return self.quiescence_search(board, alpha, beta);
		}

		let mut best_move_this_search = NULL_MOVE;
		let mut eval_bound = EvalBound::UpperBound;

		let moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, false);

		let sorted_moves = self.move_sorter.sort_moves(
			board,
			moves,
			/*
			The best move is _not_ the same as the hash move, because we could have
			found a new best move right before exiting the search, before tt.store gets called
			*/
			if ply == 0
			&& self.best_move != NULL_MOVE {
				self.best_move
			} else {
				hash_move.unwrap_or(NULL_MOVE)
			},
			ply as usize,
		);

		let mut i = 0;
		for (_, m) in sorted_moves {
			if !board.make_move(m) {
				continue;
			}

			let mut extension = 0;
			if total_extensions < MAX_SEARCH_EXTENSIONS as u8 {
				if board.king_in_check(board.white_to_move) {
					extension = 1;
				} else { // TODO: does this help at all? Or maybe try checking for a promotion flag
					if m.piece == PAWN as u8 {
						let rank = m.to / 8;
						if rank == 1 || rank == 6 {
							extension = 1;
						}
					}
				}
			}

			// Late Move Reductions
			let mut evaluation = 0;
			let mut needs_full_search = true;

			if i > 2
			&& depth > 2
			&& extension == 0
			&& m.capture == NO_PIECE as u8 {
				// let mut reduction = 1;

				// if hash_move.is_none() {
				// 	// Internal Iterative Reductions
				// 	reduction += 1;
				// }

				evaluation = -self.alpha_beta_search(board, depth - 2, ply + 1, -alpha - 1, -alpha, total_extensions);
				needs_full_search = evaluation > alpha;
			}

			if needs_full_search {
				evaluation = -self.alpha_beta_search(board, depth - 1 + extension, ply + 1, -beta, -alpha, total_extensions + extension);
			}

			board.undo_last_move();

			if self.should_cancel_search() {
				return 0;
			}

			if evaluation >= beta {
				self.transposition_table.store(board.zobrist.key, depth, ply, beta, m, EvalBound::LowerBound);

				if m.capture == NO_PIECE as u8 {
					self.move_sorter.push_killer_move(m, ply as usize);
					self.move_sorter.history[m.piece as usize][m.to as usize] += (depth * depth) as i32;
				}

				return beta;
			}

			if evaluation > alpha {
				best_move_this_search = m;
				eval_bound = EvalBound::Exact;
				alpha = evaluation;

				if ply == 0 {
					self.searched_one_move = true;
					self.best_move_this_iteration = best_move_this_search;
					self.evaluation_this_iteration = evaluation;
				}
			}

			i += 1;
		}

		if i == 0 {
			if board.king_in_check(board.white_to_move) {
				let mate_score = CHECKMATE_EVAL - ply as i32;
				return -mate_score;
			}
			return 0;
		}

		if best_move_this_search != NULL_MOVE {
			self.transposition_table.store(board.zobrist.key, depth, ply, alpha, best_move_this_search, eval_bound);
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

		let legal_moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, true);
		if legal_moves.is_empty() {
			return evaluation;
		}

		let sorted_moves = self.move_sorter.sort_moves(board, legal_moves, NULL_MOVE, usize::MAX);

		for (_, m) in sorted_moves {
			// Delta Pruning
			if !board.king_in_check(board.white_to_move) {
				let threshold = QUEEN_WORTH +
					if PROMOTABLE.contains(&m.flag) {
						QUEEN_WORTH - PAWN_WORTH
					} else {
						0
					};

				if evaluation < alpha - threshold {
					continue;
				}
			}

			if !board.make_move(m) {
				continue;
			}

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