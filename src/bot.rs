use crate::STARTING_FEN;
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
pub const FUTILITY_PRUNING_THESHOLD_PER_PLY: i32 = 60;
pub const RAZORING_THRESHOLD_PER_PLY: i32 = 300;

pub const PERCENT_OF_TIME_TO_USE_BEFORE_6_FULL_MOVES: f32 = 0.025; // 2.5%
pub const PERCENT_OF_TIME_TO_USE_AFTER_6_FULL_MOVES: f32 = 0.07; // 7%

pub const MIN_TIME_PER_MOVE: f32 = 0.25; // seconds
pub const MAX_TIME_PER_MOVE: f32 = 20.0;

#[derive(Clone, Debug)]
pub struct BotConfig {
	pub fen: String,
	pub debug_output: bool,
	pub opening_book: bool,
	pub time_management: bool,
}

impl BotConfig {
	pub fn from_args(args: Vec<String>) -> Self {
		let _true = "true".to_string();
		Self { // This is so ugly lol
			fen: Self::get_arg_value(&args, "fen").unwrap_or(STARTING_FEN.to_string()),
			debug_output: Self::get_arg_value(&args, "debug_output").unwrap_or(_true.clone()) == _true,
			opening_book: Self::get_arg_value(&args, "opening_book").unwrap_or(_true.clone()) == _true,
			time_management: Self::get_arg_value(&args, "time_management").unwrap_or(_true.clone()) == _true,
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
	best_move_depth_searched_at: u8,

	evaluation: i32,
	evaluation_this_iteration: i32,

	positions_searched: u128,
	quiescence_searched: u128,
	transposition_hits: u128,
}

impl Bot {
	pub fn new(config: BotConfig) -> Self {
		let in_opening_book = config.opening_book;

		Self {
			config,

			time_to_think: 0.0,
			think_timer: Instant::now(),
			search_cancelled: false,
			searched_one_move: false,

			opening_book: OpeningBook::create(),
			in_opening_book,

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

	pub fn println(&self, output: String) {
		if self.config.debug_output {
			println!("{}", output);
		}
	}

	pub fn start(&mut self, board: &mut Board, moves: String, my_time: f32) {
		if self.in_opening_book {
			let opening_move = self.opening_book.get_opening_move(moves);
			if opening_move == NULL_MOVE {
				self.in_opening_book = false;
			} else {
				self.best_move = opening_move;
				return;
			}
		}

		self.time_to_think = if self.config.time_management {
			let time_percentage = if board.moves.len() / 2 <= 6 {
				PERCENT_OF_TIME_TO_USE_BEFORE_6_FULL_MOVES
			} else {
				PERCENT_OF_TIME_TO_USE_AFTER_6_FULL_MOVES
			};

			(my_time * time_percentage).clamp(MIN_TIME_PER_MOVE, MAX_TIME_PER_MOVE)
		} else {
			my_time
		};

		self.search_cancelled = false;

		let last_evaluation = self.evaluation;

		self.best_move = NULL_MOVE;
		self.evaluation = 0;

		self.positions_searched = 0;
		self.quiescence_searched = 0;
		self.transposition_hits = 0;

		self.move_sorter.clear();

		let mut window = 40;

		self.think_timer = Instant::now();
		for depth in 1..=(255 - MAX_SEARCH_EXTENSIONS) {
			self.searched_one_move = false;
			self.best_move_this_iteration = NULL_MOVE;
			self.evaluation_this_iteration = 0;


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

			self.println(format!("Depth: {}, Window: {}, Evaluation: {}, Best move: {}, Positions searched: {} + Quiescence positions searched: {} = {}, Transposition Hits: {}",
				depth,
				window,
				self.evaluation * board.perspective(),
				self.best_move.to_coordinates(),
				self.positions_searched,
				self.quiescence_searched,
				self.positions_searched + self.quiescence_searched,
				self.transposition_hits,
			));

			if evaluation_is_mate(self.evaluation) {
				let moves_until_mate = moves_ply_from_mate(self.evaluation);
				if moves_until_mate <= depth {
					self.println(format!("Mate found in {}", (moves_until_mate as f32 * 0.5).ceil()));
					break;
				}
			}

			if self.search_cancelled {
				self.println("Search cancelled".to_string());
				break;
			}
		}

		self.transposition_table.update();
		if self.config.debug_output {
			self.transposition_table.print_size();
		}
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

		if depth > 0
		&& (board.fifty_move_draw.current >= 50
		|| board.insufficient_checkmating_material()
		|| board.zobrist.is_threefold_repetition()) {
			// Should I use this to discourage making a draw in a winning position?
			// return -self.quiescence_search(board, alpha, beta);
			return 0;
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

		let is_pv = alpha != beta - 1;

		if !is_pv
		&& depth > 0
		&& depth_left > 0
		&& !board.king_in_check(board.white_to_move) {
			// Null Move Pruning
			if depth_left >= 3
			&& board.try_null_move() {
				// let reduction = 3 - (depth_left - 3) / 2; // This didn't work at all lol
				let evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 3, -beta, -beta + 1, number_of_extensions);

				board.undo_null_move();

				if evaluation >= beta {
					return evaluation;
				}
			}

			let static_eval = board.evaluate();

			// Reverse Futility Pruning
			if depth_left <= 4
			&& static_eval - (FUTILITY_PRUNING_THESHOLD_PER_PLY * depth_left as i32) >= beta {
				return static_eval;
			}

			// Razoring
			if depth_left <= 3
			&& board.get_last_move().capture == NO_PIECE as u8
			&& static_eval + RAZORING_THRESHOLD_PER_PLY * (depth_left as i32) < alpha {
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



			// Late Move Reduction / (Kind of) Principal Variation Search
			let mut evaluation = 0;
			let mut needs_full_search = true;

			if search_extension == 0
			&& i >= 3
			&& depth_left >= 3
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

		// Depth is set to u8::MAX because it's only used for killer moves, and we don't need that here
		let sorted_moves = self.move_sorter.sort_moves(board, legal_moves, NULL_MOVE, u8::MAX);

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