use crate::STARTING_FEN;
use crate::piece_square_tables::{PAWN_WORTH, QUEEN_WORTH};
use crate::pieces::{PAWN, PROMOTABLE, NO_PIECE};
use crate::utils::{CHECKMATE_EVAL, evaluation_is_mate, moves_ply_from_mate};
use std::time::Instant;
use crate::move_sorter::MoveSorter;
use crate::transposition_table::{TranspositionTable, NodeType};
use crate::move_data::{MoveData, NULL_MOVE};
use crate::opening_book::OpeningBook;
use crate::Board;

pub const MAX_SEARCH_EXTENSIONS: u8 = 16; // TODO

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
		let _false = "false".to_string();

		Self { // This is so ugly lol
			fen: Self::get_arg_value(&args, "fen").unwrap_or(STARTING_FEN.to_string()),
			debug_output: Self::get_arg_value(&args, "debug_output").unwrap_or(_true.clone()) == _true,
			opening_book: Self::get_arg_value(&args, "opening_book").unwrap_or(_false.clone()) == _true,
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

	pub fn start(&mut self, board: &mut Board, moves: String, my_time: f32, depth_to_search: u8) {
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
		for depth in 1..=depth_to_search {
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

		self.println(format!("{} seconds", self.think_timer.elapsed().as_secs_f32()));

		self.transposition_table.update();
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
		depth: u8,
		mut depth_left: u8,
		mut alpha: i32,
		beta: i32,
		total_extensions: u8,
	) -> i32 {
		if self.should_cancel_search() {
			return 0;
		}

		self.positions_searched += 1;

		if depth > 0 {
			if board.is_draw() {
				// Should I use this to discourage making a draw in a winning position?
				// return -self.quiescence_search(board, alpha, beta);
				return 0;
			}

			// Mate Distance Pruning
			let alpha = i32::max(alpha, -CHECKMATE_EVAL + depth as i32);
			if alpha >= i32::min(beta, CHECKMATE_EVAL - depth as i32) {
				return alpha;
			}
		}

		if let Some(data) = self.transposition_table.lookup(board.zobrist.key, depth_left, depth, alpha, beta) {
			self.transposition_hits += 1;

			if depth == 0 {
				self.best_move_this_iteration = data.best_move;
				self.best_move_depth_searched_at = data.depth_left;
				self.evaluation_this_iteration = data.evaluation;
			}

			return data.evaluation;
		}

		let not_pv = alpha == beta - 1;

		if not_pv
		&& depth > 0
		&& depth_left > 0
		&& board.get_last_move().capture == NO_PIECE as u8 // ?
		&& !board.king_in_check(board.white_to_move) {
			// TODO: move these around

			let static_eval = board.evaluate();

			// Null Move Pruning
			if depth_left > 2
			&& static_eval >= beta
			// && board.total_material_without_pawns > 0
			&& board.try_null_move() {
				let evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 3, -beta, -beta + 1, total_extensions);

				board.undo_null_move();

				if evaluation >= beta {
					return evaluation;
				}
			}

			// Reverse Futility Pruning
			if depth_left < 5
			&& static_eval - 60 * (depth_left as i32) >= beta { // TODO: tweak this threshold
				return static_eval;
			}

			// Razoring
			if depth_left < 4
			&& static_eval + 300 * (depth_left as i32) < alpha {
				depth_left -= 1;
			}
		}

		if depth_left == 0 {
			return self.quiescence_search(board, alpha, beta);
		}

		let mut best_move_this_search = NULL_MOVE;
		let mut node_type = NodeType::UpperBound;

		let moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, false);

		let sorted_moves = self.move_sorter.sort_moves(
			board,
			moves,
			if depth == 0
			&& self.best_move != NULL_MOVE {
				self.best_move
			} else {
				match self.transposition_table.table.get(&board.zobrist.key) {
					Some(data) => data.best_move,
					None => NULL_MOVE,
				}
			},
			depth as usize,
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
				} else {
					if m.piece == PAWN as u8 {
						let rank = m.to / 8;
						if rank == 1 || rank == 6 {
							extension = 1;
						}
					}
				}
			}

			// Late Move Reduction / (Kind of) Principal Variation Search
			let mut evaluation = 0;
			let mut needs_full_search = true;

			if i > 2
			&& depth_left > 2
			&& extension == 0
			&& m.capture == NO_PIECE as u8 {
				// let reduction = 1;
				evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 2, -alpha - 1, -alpha, total_extensions);
				needs_full_search = evaluation > alpha;
			}

			if needs_full_search {
				evaluation = -self.alpha_beta_search(board, depth + 1, depth_left - 1 + extension, -beta, -alpha, total_extensions + extension);
			}

			board.undo_last_move();

			if self.should_cancel_search() {
				return 0;
			}

			if evaluation >= beta {
				self.transposition_table.store(board.zobrist.key, depth_left, depth, beta, m, NodeType::LowerBound);

				if m.capture == NO_PIECE as u8 {
					self.move_sorter.push_killer_move(m, depth as usize);
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

			i += 1;
		}

		if i == 0 {
			if board.king_in_check(board.white_to_move) {
				let mate_score = CHECKMATE_EVAL - depth as i32;
				return -mate_score;
			}
			return 0;
		}

		self.transposition_table.store(board.zobrist.key, depth_left, depth, alpha, best_move_this_search, node_type);

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