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

pub const MAX_DEPTH: u8 = 128;
pub const MAX_SEARCH_EXTENSIONS: u8 = 20;

#[derive(Clone, Debug)]
pub struct BotConfig {
	pub fen: String,
	pub debug_output: bool,
	pub opening_book: bool,
	pub time_management: bool,
	pub hash_size: usize,
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
			hash_size: (Self::get_arg_value(&args, "hash_size").unwrap_or("256".to_string())).parse::<usize>().unwrap_or(256),
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

	sel_depth: u8,

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
			transposition_table: TranspositionTable::empty(config.hash_size),

			best_move: NULL_MOVE,
			best_move_this_iteration: NULL_MOVE,

			evaluation: 0,
			evaluation_this_iteration: 0,

			sel_depth: 0,

			positions_searched: 0,
			quiescence_searched: 0,
		}
	}

	pub fn debugln(&self, output: String) {
		if self.config.debug_output {
			println!("{}", output);
		}
	}

	pub fn print_uci_info(&self, current_depth: u8, score_type: &'static str, score: i32, pv: String) {
		let total_nodes = self.positions_searched + self.quiescence_searched;
		let time_elapsed = self.think_timer.elapsed();

		println!("info depth {depth} seldepth {seldepth} score {score_type} {score} currmove {currmove} pv {pv}nodes {nodes} time {time} nps {nps}",
			depth = current_depth,
			seldepth = self.sel_depth,
			score_type = score_type,
			score = score,
			currmove = self.best_move.to_coordinates(),
			pv = pv,
			nodes = total_nodes,
			time = time_elapsed.as_millis(),
			nps = total_nodes as f32 / time_elapsed.as_secs_f32(),
		);
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

				(my_time * time_percentage).clamp(0.05, 30.0)
			} else {
				my_time
			};

		self.search_cancelled = false;

		self.best_move = NULL_MOVE;

		self.positions_searched = 0;
		self.quiescence_searched = 0;
		self.transposition_table.hits = 0;

		self.move_sorter.clear();

		// TODO: tweak this
		let mut window = 40;

		self.think_timer = Instant::now();
		for current_depth in 1..=depth {
			self.searched_one_move = false;
			self.best_move_this_iteration = NULL_MOVE;
			self.evaluation_this_iteration = 0;
			// self.move_sorter.new_pv.clear();
			self.sel_depth = 0;

			loop {
				let (alpha, beta) = (self.evaluation - window, self.evaluation + window);

				let evaluation = self.alpha_beta_search(board, current_depth, 0, alpha, beta, 0);

				if evaluation > alpha
				&& evaluation < beta {
					break;
				}

				window *= 4;
			}

			if !self.search_cancelled
			|| self.searched_one_move {
				self.best_move = self.best_move_this_iteration;
				self.evaluation = self.evaluation_this_iteration;
			}

			let pv = self.find_pv(board, current_depth);

			if evaluation_is_mate(self.evaluation) {
				let moves_until_mate = ply_from_mate(self.evaluation);
				if moves_until_mate <= current_depth {
					let mate_evaluation = (moves_until_mate as f32 * 0.5).ceil() as i32 * (if self.evaluation > 0 { 1 } else { -1 });

					self.print_uci_info(
						current_depth,
						"mate",
						mate_evaluation,
						pv,
					);

					break;
				}
			}

			self.print_uci_info(
				current_depth,
				"cp",
				self.evaluation,
				pv,
			);

			if self.search_cancelled {
				break;
			}
		}

		if self.best_move == NULL_MOVE {
			let legal_moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, false);
			for m in legal_moves {
				if board.make_move(m) {
					board.undo_last_move(); // Technically not necessary, because all moves get undone upon receiving the "position" command, but makes me happy :>
					self.best_move = m;
					break;
				}
			}
			self.debugln("Failed to find a move in time, defaulting to first legal move :(".to_string());
		}

		self.debugln(format!("{} seconds", self.think_timer.elapsed().as_secs_f32()));

		if self.config.debug_output {
			self.transposition_table.print_size();
		}
	}

	fn should_cancel_search(&mut self) -> bool {
		self.search_cancelled = (self.search_cancelled || self.think_timer.elapsed().as_secs_f32() >= self.time_to_think) && self.time_to_think > 0.0;
		self.search_cancelled
	}

	fn find_pv(&mut self, board: &mut Board, depth: u8) -> String {
		if depth == 0 {
			return String::new();
		}

		if let Some(data) = self.transposition_table.get(board.zobrist.key.current) {
			let hash_move = MoveData::from_binary(data.best_move);

			if !board.play_move(hash_move) {
				return String::new();
			}

			let pv = format!("{} {}", hash_move.to_coordinates(), self.find_pv(board, depth - 1));

			board.undo_last_move();

			return pv;
		}

		String::new()
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

		if ply > 0 {
			self.sel_depth = u8::max(self.sel_depth, ply);
			self.positions_searched += 1;

			if board.is_draw() {
				// Should I use this to discourage making a draw in a winning position?
				// return -self.quiescence_search(board, alpha, beta);
				return 0;
			}

			// Mate Distance Pruning
			let mate_value = CHECKMATE_EVAL - ply as i32;
			let alpha = i32::max(alpha, -mate_value);
			let beta = i32::min(beta, mate_value - 1);
			if alpha >= beta {
				return alpha;
			}
		}

		let (tt_eval, hash_move) = self.transposition_table.lookup(board.zobrist.key.current, ply, depth, alpha, beta);

		// We don't really want to return from the root node, because if a hash collision occurs (although very rare)
		// It will return an illegal move
		if ply > 0 {
			if let Some(tt_eval) = tt_eval {
				return tt_eval;
			}

			// Internal Iterative Reductions
			if depth > 1
			&& hash_move.is_none() {
				depth -= 1;
			}
		}

		// This detects a null / zero window search, which is used in non PV nodes
		// This will also never be true if ply == 0 because the bounds will never be zero at ply 0
		let not_pv = alpha == beta - 1;
		let in_check = board.king_in_check(board.white_to_move);

		if not_pv
		&& depth > 0
		&& !in_check
		&& !evaluation_is_mate(alpha)
		&& !evaluation_is_mate(beta) {
			let static_eval = board.hc_evaluate();

			// Reverse Futility Pruning
			if depth < 8 // TODO: mess around with this
			&& static_eval - (55 + 50 * (depth as i32 - 1).pow(2)) >= beta { // TODO: continue tweaking this
				return beta;
			}

			// Strelka Razoring (Slightly modified)
			// if depth < 4 {
			// 	let razoring_threshold = static_eval +
			// 		if depth == 1 {
			// 			150
			// 		} else {
			// 			350
			// 		};

			// 	if razoring_threshold < beta {
			// 		let evaluation = self.quiescence_search(board, alpha, beta);
			// 		if evaluation < beta {
			// 			return i32::max(evaluation, razoring_threshold);
			// 		}
			// 	}
			// }

			// Null Move Pruning
			if depth > 1
			&& static_eval >= beta // Fruit used || depth > X here, but I haven't found great results with that
			&& board.total_material_without_pawns[board.white_to_move as usize] > 0
			&& board.get_last_move().capture == NO_PIECE as u8 // Moving the check from the above check to only NMP was a decent improvement
			&& board.try_null_move() {
				let reduction = 3 + (depth - 2) / 3;
				let evaluation = -self.alpha_beta_search(board, depth.saturating_sub(reduction), ply + 1, -beta, -beta + 1, total_extensions);

				board.undo_null_move();

				if evaluation >= beta {
					return evaluation;
				}
			}

			// Razoring
			if depth < 4 // TODO: try different values for this
			&& static_eval + (300 + 150 * (depth as i32 - 1)) < alpha { // TODO: tweak this some more
				depth -= 1;
			}
		}

		if depth == 0 {
			return self.quiescence_search(board, alpha, beta, true);
		}

		let mut best_move_this_search = NULL_MOVE;
		// let mut eval_bound = EvalBound::UpperBound;

		let sorted_moves = self.move_sorter.sort_moves(
			board.white_to_move,
			board.get_pseudo_legal_moves_for_color(board.white_to_move, false),
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

		let mut found_pv = false;

		let mut legal_moves_found = 0;
		for (_score, m) in sorted_moves {
			if !board.make_move(m) {
				continue;
			}

			let mut extension = 0;
			if board.king_in_check(board.white_to_move) {
				extension += 1;
			}

			if m.piece == PAWN as u8 {
				let rank = m.to / 8;
				if rank == 1 || rank == 6 {
					extension += 1;
				}
			}

			extension = u8::min(extension, MAX_SEARCH_EXTENSIONS - total_extensions);

			let mut evaluation = 0;
			let mut needs_fuller_search = true;

			// Late Move Reductions
			if legal_moves_found > 3
			&& depth > 1
			&& extension == 0
			&& m.capture == NO_PIECE as u8 {
				let mut reduction = 2;

				if found_pv {
					reduction += 1;
				}

				if not_pv {
					reduction += 1; // TODO: + depth / 6
				}

				evaluation = -self.alpha_beta_search(board, depth.saturating_sub(reduction), ply + 1, -alpha - 1, -alpha, total_extensions);
				needs_fuller_search = evaluation > alpha; // && evaluation < beta?
			}

			// Principal Variation Search
			if needs_fuller_search
			&& found_pv {
				evaluation = -self.alpha_beta_search(board, depth + extension - 1, ply + 1, -alpha - 1, -alpha, total_extensions + extension);
				needs_fuller_search = evaluation > alpha; // && evaluation < beta?
			}

			if needs_fuller_search {
				evaluation = -self.alpha_beta_search(board, depth + extension - 1, ply + 1, -beta, -alpha, total_extensions + extension);
			}

			board.undo_last_move();

			if self.should_cancel_search() {
				return 0;
			}

			if evaluation >= beta {
				self.transposition_table.store(board.zobrist.key.current, depth, ply, beta, m, EvalBound::LowerBound);

				if m.capture == NO_PIECE as u8 {
					self.move_sorter.add_killer_move(m, ply as usize);
					self.move_sorter.history[board.white_to_move as usize][m.from as usize][m.to as usize] += (depth * depth) as i32;
				}

				return beta;
			}

			if evaluation > alpha {
				found_pv = true;

				best_move_this_search = m;
				// eval_bound = EvalBound::Exact;
				alpha = evaluation;

				if ply == 0 {
					self.searched_one_move = true;
					self.best_move_this_iteration = best_move_this_search;
					self.evaluation_this_iteration = evaluation;
				}
			}

			legal_moves_found += 1;
		}

		if legal_moves_found == 0 {
			if in_check {
				let mate_score = CHECKMATE_EVAL - ply as i32;
				return -mate_score;
			}
			return 0;
		}

		// I've seen a small improvement if I don't store EvalBound::UpperBound, is this normal?
		if best_move_this_search != NULL_MOVE {
			self.transposition_table.store(board.zobrist.key.current, depth, ply, alpha, best_move_this_search, EvalBound::Exact);
		}

		alpha
	}

	fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, is_root: bool) -> i32 {
		if self.should_cancel_search() {
			return 0;
		}

		if is_root {
			self.positions_searched -= 1;
		}

		self.quiescence_searched += 1;

		let evaluation = board.hc_evaluate();
		if evaluation >= beta {
			return beta;
		}

		if evaluation > alpha {
			alpha = evaluation;
		}

		let moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, true);
		if moves.is_empty() {
			return evaluation;
		}

		let sorted_moves = self.move_sorter.sort_moves(board.white_to_move, moves, NULL_MOVE, usize::MAX);
		for (_score, m) in sorted_moves {
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

			let evaluation = -self.quiescence_search(board, -beta, -alpha, false);
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