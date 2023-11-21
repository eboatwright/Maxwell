use std::collections::HashMap;
use crate::zobrist::Zobrist;
use std::ops::Range;
use crate::precomputed_data::*;
use crate::utils::*;
use crate::piece::*;

pub const MAX_TRANSPOSITION_TABLE_SIZE: u64 = 3355440; // This is roughly 64 MB

#[derive(Copy, Clone)]
pub struct TranspositionData {
	pub depth: u16,
	pub evaluation: i32,
	pub best_move: u32,
}

#[derive(Clone)]
pub struct Board {
	pub precomputed_data: PrecomputedData,
	pub piece_bitboards: [[u64; 6]; 2],
	pub all_piece_bitboards: [u64; 2],
	pub attacked_squares_bitboards: [u64; 2],

	pub board: [u8; 64],
	pub whites_turn: bool,

	pub en_passant_file: usize,

	pub zobrist_key_history: Vec<u64>,
	pub castle_rights_history: Vec<u8>,
	pub fifty_move_draw_history: Vec<u8>,
	pub moves: Vec<u32>,

	pub zobrist: Zobrist,
	pub transposition_table: HashMap<u64, TranspositionData>,
}

impl Board {
	/*
	https://www.chessprogramming.org/Forsyth-Edwards_Notation
	Starting FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
	*/
	pub fn from_fen(fen: &'static str) -> Board {
		let fen_sections: Vec<&str> = fen.split(' ').collect();



		let pieces = fen_sections[0].chars().collect::<Vec<char>>();
		let mut board = [0; 64];
		let mut board_index = 0usize;

		for i in 0..pieces.len() {
			if let Ok(number_of_empty_squares) = pieces[i].to_string().parse::<usize>() {
				board_index += number_of_empty_squares;
			} else {
				board[board_index] = match pieces[i] {
					'P' => WHITE | PAWN,
					'N' => WHITE | KNIGHT,
					'B' => WHITE | BISHOP,
					'R' => WHITE | ROOK,
					'Q' => WHITE | QUEEN,
					'K' => WHITE | KING,

					'p' => BLACK | PAWN,
					'n' => BLACK | KNIGHT,
					'b' => BLACK | BISHOP,
					'r' => BLACK | ROOK,
					'q' => BLACK | QUEEN,
					'k' => BLACK | KING,

					_ => 0,
				};

				// If a piece wasn't placed don't increment index (this is for the '/' characters in the FEN)
				if board[board_index] != 0 {
					board_index += 1;
				}
			}
		}



		let whites_turn = fen_sections[1] == 'w'.to_string();




		let en_passant_file = file_index_from_coordinate(fen_sections[3]).unwrap_or(0);


		let fifty_move_draw = fen_sections[4].parse::<u8>().expect("Invalid FEN: no 'Halfmove Clock'");
		let fullmove_counter = fen_sections[5].parse::<u16>().expect("Invalid FEN: no 'Fullmove Counter'");



		// Order from left bit to right bit: white long, white short, black long, black short
		let mut castling_rights = 0b_0000;
		castling_rights |= (fen_sections[2].contains('Q') as u8) << 3;
		castling_rights |= (fen_sections[2].contains('K') as u8) << 2;
		castling_rights |= (fen_sections[2].contains('q') as u8) << 1;
		castling_rights |= fen_sections[2].contains('k') as u8;




		let mut final_board = Board {
			precomputed_data: PrecomputedData::calculate(),
			piece_bitboards: [[0; 6]; 2],
			all_piece_bitboards: [0; 2],
			attacked_squares_bitboards: [0; 2],

			board,
			whites_turn,

			en_passant_file,

			zobrist_key_history: vec![],
			castle_rights_history: vec![castling_rights],
			fifty_move_draw_history: vec![fifty_move_draw],
			moves: vec![],

			zobrist: Zobrist::generate(),
			transposition_table: HashMap::new(),
		};


		for i in 0..64 {
			if board[i] != 0 {
				let piece_color = is_white(board[i]) as usize;
				let piece_type = get_piece_type(board[i]) as usize;

				final_board.piece_bitboards[piece_color][piece_type - 1] |= 1 << i;
			}
		}


		final_board.compute_all_piece_bitboards();
		final_board.compute_attacked_squares_bitboards();


		final_board.calculate_initial_zobrist_key();


		final_board
	}

	pub fn print_to_console(&self) {
		for y in 0..8 {
			let mut line = "".to_string();

			for x in 0..8 {
				let index = x + y * 8;
				line.push(piece_to_char(self.board[index]));
				line.push(' ');
			}

			println!("{}", line);
		}

		println!("\n\n");
	}

	pub fn fifty_move_draw(&self) -> u8 { self.fifty_move_draw_history[self.fifty_move_draw_history.len() - 1] }

	pub fn compute_all_piece_bitboards(&mut self) {
		self.all_piece_bitboards = [
			self.piece_bitboards[0][0] | self.piece_bitboards[0][1] | self.piece_bitboards[0][2] | self.piece_bitboards[0][3] | self.piece_bitboards[0][4] | self.piece_bitboards[0][5],
			self.piece_bitboards[1][0] | self.piece_bitboards[1][1] | self.piece_bitboards[1][2] | self.piece_bitboards[1][3] | self.piece_bitboards[1][4] | self.piece_bitboards[1][5],
		];
	}

	pub fn king_in_check(&self, white: bool) -> bool {
		self.piece_bitboards[white as usize][(KING - 1) as usize] & self.attacked_squares_bitboards[!white as usize] != 0
	}

	pub fn compute_attacked_squares_bitboards(&mut self) {
		self.attacked_squares_bitboards = [0; 2];

		for i in 0..64 {
			let piece = self.board[i];

			if piece != 0 {
				let piece_color = is_white(piece) as usize;

				match get_piece_type(piece) {
					PAWN => {
						self.attacked_squares_bitboards[piece_color] |= self.precomputed_data.pawn_bitboards[piece_color][i];
					}

					KNIGHT => {
						self.attacked_squares_bitboards[piece_color] |= self.precomputed_data.knight_bitboards[i];
					}

					BISHOP => {
						self.attacked_squares_bitboards[piece_color] |= self.generate_sliding_attacks_bitboard(i, 4..8)
					}

					ROOK => {
						self.attacked_squares_bitboards[piece_color] |= self.generate_sliding_attacks_bitboard(i, 0..4)
					}

					QUEEN => {
						self.attacked_squares_bitboards[piece_color] |= self.generate_sliding_attacks_bitboard(i, 0..8)
					}

					KING => {
						self.attacked_squares_bitboards[piece_color] |= self.precomputed_data.king_bitboards[i];
					}

					_ => {}
				}
			}
		}
	}




	fn generate_sliding_moves(&self, piece_from: usize, direction_range: Range<usize>) -> Vec<u32> {
		let mut result = vec![];

		for direction_index in direction_range {
			for n in 1..=self.precomputed_data.squares_to_edge[piece_from][direction_index] {
				let to = (piece_from as i8 + DIRECTION_OFFSETS[direction_index] * n as i8) as usize;
				let is_piece_on_target_square = self.board[to] != 0;

				if is_piece_on_target_square
				&& is_white(self.board[to]) == is_white(self.board[piece_from]) {
					break;
				}

				result.push(build_move(0, self.board[to], piece_from, to));

				if is_piece_on_target_square {
					break;
				}
			}
		}

		result
	}

	pub fn generate_sliding_attacks_bitboard(&self, piece_from: usize, direction_range: Range<usize>) -> u64 {
		let mut result_bitboard = 0;

		for direction_index in direction_range {
			for n in 1..=self.precomputed_data.squares_to_edge[piece_from][direction_index] {
				let to = (piece_from as i8 + DIRECTION_OFFSETS[direction_index] * n as i8) as usize;

				result_bitboard |= 1 << to;

				if self.board[to] != 0 {
					break;
				}
			}
		}

		result_bitboard
	}




	pub fn get_last_move(&self) -> u32 {
		if self.moves.len() == 0 {
			return 0;
		}
		self.moves[self.moves.len() - 1]
	}

	pub fn get_legal_moves_for_color(&mut self, white_pieces: bool) -> Vec<u32> {
		let mut result = vec![];

		for i in 0..64 {
			if self.board[i] != 0
			&& is_white(self.board[i]) == white_pieces {
				// result.append(&mut self.get_legal_moves_for_piece(i));
				result = [result, self.get_legal_moves_for_piece(i)].concat();
			}
		}

		result
	}

	pub fn get_legal_moves_for_piece(&mut self, piece_index: usize) -> Vec<u32> {
		let piece_color = is_white(self.board[piece_index]) as usize;
		let other_color = !is_white(self.board[piece_index]) as usize;
		let piece_type = get_piece_type(self.board[piece_index]);

		let mut result = vec![];

		let piece = 1 << piece_index;

		match piece_type {
			PAWN => {
				let empty_squares = !(self.all_piece_bitboards[0] | self.all_piece_bitboards[1]);

				if piece_color == 1 {



					let will_promote = rank_of_index(piece_index) == 7;




					// Pushing
					if (piece >> 8) & empty_squares != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, 0, piece_index, piece_index - 8));
							}
						} else {
							result.push(build_move(0, 0, piece_index, piece_index - 8));

							if rank_of_index(piece_index) == 2
							&& (piece >> 16) & empty_squares != 0 {
								result.push(build_move(DOUBLE_PAWN_PUSH_FLAG, 0, piece_index, piece_index - 16));
							}
						}
					}




					// Capturing
					if (piece >> 7) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index - 7], piece_index, piece_index - 7));
							}
						} else {
							result.push(build_move(0, self.board[piece_index - 7], piece_index, piece_index - 7));
						}
					}

					if (piece >> 9) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index - 9], piece_index, piece_index - 9));
							}
						} else {
							result.push(build_move(0, self.board[piece_index - 9], piece_index, piece_index - 9));
						}
					}



					// En Passant
					let last_move = self.get_last_move();
					if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
						let pawn_index = get_move_to(last_move);
						if (pawn_index == piece_index - 1
						&& piece_index % 8 != 0)
						|| pawn_index == piece_index + 1 {
							result.push(build_move(EN_PASSANT_FLAG, self.board[pawn_index], piece_index, pawn_index - 8));
						}
					}




				} else {



					let will_promote = rank_of_index(piece_index) == 2;




					// Pushing
					if (piece << 8) & empty_squares != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, 0, piece_index, piece_index + 8));
							}
						} else {
							result.push(build_move(0, 0, piece_index, piece_index + 8));

							if rank_of_index(piece_index) == 7
							&& (piece << 16) & empty_squares != 0 {
								result.push(build_move(DOUBLE_PAWN_PUSH_FLAG, 0, piece_index, piece_index + 16));
							}
						}
					}




					// Capturing
					if (piece << 7) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index + 7], piece_index, piece_index + 7));
							}
						} else {
							result.push(build_move(0, self.board[piece_index + 7], piece_index, piece_index + 7));
						}
					}

					if (piece << 9) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index + 9], piece_index, piece_index + 9));
							}
						} else {
							result.push(build_move(0, self.board[piece_index + 9], piece_index, piece_index + 9));
						}
					}



					// En Passant
					let last_move = self.get_last_move();
					if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
						let pawn_index = get_move_to(last_move);
						if pawn_index == piece_index - 1
						|| (pawn_index == piece_index + 1
						&& piece_index % 8 != 7) {
							result.push(build_move(EN_PASSANT_FLAG, self.board[pawn_index], piece_index, pawn_index + 8));
						}
					}




				}
			}




			KNIGHT => {
				let bitboard = self.precomputed_data.knight_bitboards[piece_index] & !self.all_piece_bitboards[piece_color];

				for offset in [10, 17, 15, 6] {
					if (piece >> offset) & bitboard != 0 {
						let i = piece_index - offset;
						result.push(build_move(0, self.board[i], piece_index, i));
					}

					if (piece << offset) & bitboard != 0 {
						let i = piece_index + offset;
						result.push(build_move(0, self.board[i], piece_index, i));
					}
				}
			}




			BISHOP => {
				result = self.generate_sliding_moves(piece_index, 4..8);
			}




			ROOK => {
				result = self.generate_sliding_moves(piece_index, 0..4);
			}




			QUEEN => {
				result = self.generate_sliding_moves(piece_index, 0..8);
			}




			KING => {
				/* !!!!!!!!!!IMPORTANT!!!!!!!!!!
				This bitboard excludes squares the opponent attacks to skip over squares that will put the king into check,
				this will affect the # of nodes in a search test! (I'm probably gonna forget this anyways and lose my mind over why I get the wrong numbers :P)
				*/
				let bitboard = self.precomputed_data.king_bitboards[piece_index] & !self.all_piece_bitboards[piece_color] & !self.attacked_squares_bitboards[other_color];

				for offset in [9, 8, 7, 1] {
					if (piece >> offset) & bitboard != 0 {
						let i = piece_index - offset;
						result.push(build_move(0, self.board[i], piece_index, i));
					}

					if (piece << offset) & bitboard != 0 {
						let i = piece_index + offset;
						result.push(build_move(0, self.board[i], piece_index, i));
					}
				}

				if self.can_castle_short(piece_color == 1)
				&& self.board[piece_index + 1] == 0
				&& self.board[piece_index + 2] == 0 {
					result.push(build_move(CASTLE_SHORT_FLAG, 0, piece_index, piece_index + 2));
				}

				if self.can_castle_long(piece_color == 1)
				&& self.board[piece_index - 1] == 0
				&& self.board[piece_index - 2] == 0
				&& self.board[piece_index - 3] == 0 {
					result.push(build_move(CASTLE_LONG_FLAG, 0, piece_index, piece_index - 2));
				}
			}

			_ => {}
		};

		for i in (0..result.len()).rev() {
			let m = result[i];
			let move_flag = get_move_flag(m);

			if (move_flag == CASTLE_SHORT_FLAG
			|| move_flag == CASTLE_LONG_FLAG)
			&& self.king_in_check(self.whites_turn) {
				result.remove(i);
				continue;
			}

			self.make_move(result[i]);

			if self.king_in_check(!self.whites_turn) {
				result.remove(i);
				self.undo_last_move();
				continue;
			}

			self.undo_last_move();

			if move_flag == CASTLE_SHORT_FLAG {
				if (1 << (get_move_from(m) + 1)) & self.attacked_squares_bitboards[!self.whites_turn as usize] != 0 {
					result.remove(i);
				}
			} else if move_flag == CASTLE_LONG_FLAG {
				if (1 << (get_move_from(m) - 1)) & self.attacked_squares_bitboards[!self.whites_turn as usize] != 0 {
					result.remove(i);
				}
			}
		}

		result
	}

	pub fn play_move(&mut self, promotion: u8, from: usize, to: usize) {
		let promoting = promotion != 0;

		for m in self.get_legal_moves_for_piece(from) {
			if (!promoting || get_move_flag(m) == promotion)
			&& get_move_from(m) == from && get_move_to(m) == to {
				self.make_move(m);
				break;
			}
		}
	}

	fn toggle_bit(&mut self, is_white: bool, piece: u8, index: usize) {
		self.piece_bitboards[is_white as usize][piece as usize - 1] ^= 1 << index;
	}

	pub fn get_all_castle_rights(&self) -> u8 {
		self.castle_rights_history[self.castle_rights_history.len() - 1]
	}

	pub fn can_castle_short(&self, is_white: bool) -> bool {
		self.get_all_castle_rights() >> (if is_white { 2 } else { 0 }) & 1 == 1
	}

	pub fn can_castle_long(&self, is_white: bool) -> bool {
		self.get_all_castle_rights() >> (if is_white { 3 } else { 1 }) & 1 == 1
	}

	pub fn make_move(&mut self, piece_move: u32) {
		let mut new_castle_rights = self.get_all_castle_rights();

		let from = get_move_from(piece_move);
		let to = get_move_to(piece_move);
		let flag = get_move_flag(piece_move);
		let capture = get_move_capture(piece_move);

		let piece = self.board[from];
		let piece_is_white = is_white(piece);
		let piece_type = get_piece_type(piece);

		self.board[to] = piece;
		self.board[from] = 0;

		if PROMOTABLE_PIECES.contains(&flag) {
			self.toggle_bit(piece_is_white, PAWN, from);
			self.toggle_bit(piece_is_white, flag, to);

			self.board[to] = (piece_is_white as u8) << 3 | flag;
		} else {
			self.toggle_bit(piece_is_white, piece_type, to);
			self.toggle_bit(piece_is_white, piece_type, from);
		}

		if flag == EN_PASSANT_FLAG {
			let pawn_square = if piece_is_white {
				to + 8
			} else {
				to - 8
			};

			self.board[pawn_square] = 0;
			self.toggle_bit(is_white(capture), get_piece_type(capture), pawn_square);
		} else if capture != 0 {
			self.toggle_bit(is_white(capture), get_piece_type(capture), to);
		}

		if piece_type == KING {
			new_castle_rights &= !CASTLING[piece_is_white as usize];
		}

		if from == 0
		|| to == 0 {
			new_castle_rights &= !BLACK_LONGCASTLE;
		} else if from == 7
		|| to == 7 {
			new_castle_rights &= !BLACK_SHORTCASTLE;
		} else if from == 56
		|| to == 56 {
			new_castle_rights &= !WHITE_LONGCASTLE;
		} else if from == 63
		|| to == 63 {
			new_castle_rights &= !WHITE_SHORTCASTLE;
		}

		if flag == CASTLE_SHORT_FLAG {
			self.board[to - 1] = self.board[to + 1];
			self.board[to + 1] = 0;

			self.toggle_bit(piece_is_white, ROOK, to + 1);
			self.toggle_bit(piece_is_white, ROOK, to - 1);
		} else if flag == CASTLE_LONG_FLAG {
			self.board[to + 1] = self.board[to - 2];
			self.board[to - 2] = 0;

			self.toggle_bit(piece_is_white, ROOK, to - 2);
			self.toggle_bit(piece_is_white, ROOK, to + 1);
		}

		self.compute_all_piece_bitboards();
		self.compute_attacked_squares_bitboards();

		self.castle_rights_history.push(new_castle_rights);
		self.make_move_on_zobrist_key(piece_move);
		self.moves.push(piece_move);


		let mut new_fifty_move_draw = self.fifty_move_draw();
		if piece_type == PAWN
		|| capture != 0 {
			new_fifty_move_draw = 0;
		} else {
			new_fifty_move_draw += 1;
		}
		self.fifty_move_draw_history.push(new_fifty_move_draw);


		self.whites_turn = !self.whites_turn;
	}

	pub fn undo_last_move(&mut self) {
		if self.moves.len() == 0 {
			return;
		}

		self.zobrist_key_history.pop();
		self.castle_rights_history.pop();
		self.fifty_move_draw_history.pop();
		let last_move = self.moves.pop().unwrap();

		let flag = get_move_flag(last_move);
		let capture = get_move_capture(last_move);
		let from = get_move_from(last_move);
		let to = get_move_to(last_move);

		let piece = self.board[to];
		let piece_is_white = is_white(piece);
		let piece_type = get_piece_type(piece);

		self.board[from] = piece;
		self.board[to] = capture;

		if PROMOTABLE_PIECES.contains(&flag) {
			self.toggle_bit(piece_is_white, PAWN, from);
			self.toggle_bit(piece_is_white, flag, to);

			self.board[from] = build_piece(piece_is_white, PAWN);
		} else {
			self.toggle_bit(piece_is_white, piece_type, to);
			self.toggle_bit(piece_is_white, piece_type, from);
		}

		if flag == EN_PASSANT_FLAG {
			let pawn_square = if piece_is_white {
				to + 8
			} else {
				to - 8
			};

			self.board[pawn_square] = capture;
			self.board[to] = 0;

			self.toggle_bit(is_white(capture), get_piece_type(capture), pawn_square);
		} else if capture != 0 {
			self.toggle_bit(is_white(capture), get_piece_type(capture), to);
		}


		if flag == CASTLE_SHORT_FLAG {
			self.board[to + 1] = self.board[to - 1];
			self.board[to - 1] = 0;

			self.toggle_bit(piece_is_white, ROOK, to + 1);
			self.toggle_bit(piece_is_white, ROOK, to - 1);
		} else if flag == CASTLE_LONG_FLAG {
			self.board[to - 2] = self.board[to + 1];
			self.board[to + 1] = 0;

			self.toggle_bit(piece_is_white, ROOK, to - 2);
			self.toggle_bit(piece_is_white, ROOK, to + 1);
		}


		self.compute_all_piece_bitboards();
		self.compute_attacked_squares_bitboards();

		self.whites_turn = !self.whites_turn;
	}



	pub fn evaluate(&self) -> i32 {
		let mut white_material = 0;
		let mut black_material = 0;

		let mut white_attacked_squares = 0;
		let mut black_attacked_squares = 0;

		for i in 0..64 {
			let piece = self.board[i];

			if piece != 0 {
				let worth = get_full_piece_worth(piece, i);

				if is_white(piece) {
					white_material += worth;
				} else {
					black_material += worth;
				}
			}

			if (1 << i) & self.attacked_squares_bitboards[0] != 0 {
				black_attacked_squares += 1;
			}

			if (1 << i) & self.attacked_squares_bitboards[1] != 0 {
				white_attacked_squares += 1;
			}
		}

		let perspective = if self.whites_turn { 1 } else { -1 };
		((white_material + white_attacked_squares) - (black_material + black_attacked_squares)) * perspective
	}











	pub fn calculate_initial_zobrist_key(&mut self) {
		let mut key = 0;

		for i in 0..64 {
			let piece = self.board[i];
			if piece != 0 {
				key ^= self.zobrist.pieces[get_piece_color(piece) as usize][get_piece_type(piece) as usize - 1][i];
			}
		}

		key ^= self.zobrist.castling_rights[self.get_all_castle_rights() as usize];

		key ^= self.zobrist.en_passant[self.en_passant_file];

		if !self.whites_turn {
			key ^= self.zobrist.side_to_move;
		}

		self.zobrist_key_history.push(key);
	}

	pub fn make_move_on_zobrist_key(&mut self, m: u32) {
		let flag = get_move_flag(m);
		let capture = get_piece_type(get_move_capture(m));
		let from = get_move_from(m);
		let to = get_move_to(m);

		let piece = self.board[to];
		let piece_is_white = is_white(piece);
		let piece_type = get_piece_type(piece) as usize;


		let mut key = self.current_zobrist_key();


		key ^= self.zobrist.pieces[piece_is_white as usize][piece_type - 1][from];
		key ^= self.zobrist.pieces[piece_is_white as usize][piece_type - 1][to];

		if flag == CASTLE_SHORT_FLAG {
			key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to + 1];
			key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to - 1];
		} else if flag == CASTLE_LONG_FLAG {
			key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to - 2];
			key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to + 1];
		}

		if capture != 0 {
			key ^= self.zobrist.pieces[(!piece_is_white) as usize][capture as usize - 1][to];
		}

		key ^= self.zobrist.castling_rights[self.castle_rights_history[self.castle_rights_history.len() - 2] as usize];
		key ^= self.zobrist.castling_rights[self.get_all_castle_rights() as usize];

		let last_move = self.get_last_move();
		if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
			let file = (get_move_to(last_move) % 8) + 1;
			key ^= self.zobrist.en_passant[file];
		}

		self.en_passant_file = 0;
		if flag == DOUBLE_PAWN_PUSH_FLAG {
			self.en_passant_file = (to % 8) + 1;
			key ^= self.zobrist.en_passant[self.en_passant_file];
		}

		key ^= self.zobrist.side_to_move;


		self.zobrist_key_history.push(key);
	}

	pub fn current_zobrist_key(&self) -> u64 { self.zobrist_key_history[self.zobrist_key_history.len() - 1] }





	pub fn store_transposition(&mut self, depth: u16, evaluation: i32, best_move: u32) {
		self.transposition_table.insert(self.current_zobrist_key(),
			TranspositionData {
				depth,
				evaluation,
				best_move,
			}
		);
	}

	pub fn lookup_transposition(&mut self, depth: u16) -> Option<TranspositionData> {
		let zobrist_key = self.current_zobrist_key();
		if let Some(data) = self.transposition_table.get(&zobrist_key) {
			if data.depth >= depth {
				return Some(*data);
			}
			self.transposition_table.remove(&zobrist_key);
		}
		None
	}
}