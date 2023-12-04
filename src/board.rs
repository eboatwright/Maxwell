use macroquad::prelude::clamp;
use std::collections::HashMap;
use crate::zobrist::Zobrist;
use std::ops::Range;
use crate::precomputed_data::*;
use crate::utils::*;
use crate::piece::*;

#[derive(Copy, Clone, PartialEq)]
pub enum NodeType {
	UpperBound,
	LowerBound,
	Exact,
}

#[derive(Copy, Clone)]
pub struct TranspositionData {
	pub depth: u16,
	pub evaluation: i32,
	pub best_move: u32,
	pub age: u8,
	pub node_type: NodeType,
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
	pub current_zobrist_key: u64,

	pub castle_rights_history: Vec<u8>,
	pub current_castle_rights: u8,

	pub fifty_move_draw_history: Vec<u8>,
	pub current_fifty_move_draw: u8,

	pub moves: Vec<u32>,

	pub zobrist: Zobrist,
	pub transposition_table: HashMap<u64, TranspositionData>,

	pub total_material_without_pawns: i32,
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
		let mut total_material_without_pawns = 0;

		for i in 0..pieces.len() {
			if let Ok(number_of_empty_squares) = pieces[i].to_string().parse::<usize>() {
				board_index += number_of_empty_squares;
			} else {
				board[board_index] = match pieces[i] {
					'P' => WHITE | PAWN,
					'N' => { total_material_without_pawns += KNIGHT_WORTH; WHITE | KNIGHT },
					'B' => { total_material_without_pawns += BISHOP_WORTH; WHITE | BISHOP },
					'R' => { total_material_without_pawns += ROOK_WORTH; WHITE | ROOK },
					'Q' => { total_material_without_pawns += QUEEN_WORTH; WHITE | QUEEN },
					'K' => WHITE | KING,

					'p' => BLACK | PAWN,
					'n' => { total_material_without_pawns += KNIGHT_WORTH; BLACK | KNIGHT },
					'b' => { total_material_without_pawns += BISHOP_WORTH; BLACK | BISHOP },
					'r' => { total_material_without_pawns += ROOK_WORTH; BLACK | ROOK },
					'q' => { total_material_without_pawns += QUEEN_WORTH; BLACK | QUEEN },
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
			current_zobrist_key: 0,

			castle_rights_history: vec![castling_rights],
			current_castle_rights: castling_rights,

			fifty_move_draw_history: vec![fifty_move_draw],
			current_fifty_move_draw: fifty_move_draw,

			moves: vec![],

			zobrist: Zobrist::generate(),
			transposition_table: HashMap::new(),

			total_material_without_pawns,
		};


		for i in 0..64 {
			if board[i] != 0 {
				let piece_color = is_white(board[i]) as usize;
				let piece_type = get_piece_type(board[i]) as usize;

				final_board.piece_bitboards[piece_color][piece_type - 1] |= 1 << i;
			}
		}


		final_board.compute_all_piece_bitboards();
		final_board.calculate_attacked_squares_bitboards();


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

	pub fn is_threefold_repetition(&self) -> bool {
		let mut count = 0;

		for zobrist_key in self.zobrist_key_history.iter() {
			if *zobrist_key == self.current_zobrist_key {
				count += 1;
				if count >= 3 {
					return true;
				}
			}
		}

		false
	}

	pub fn is_repetition(&self) -> bool {
		let mut count = 0;

		for zobrist_key in self.zobrist_key_history.iter() {
			if *zobrist_key == self.current_zobrist_key {
				count += 1;
				if count > 1 {
					return true;
				}
			}
		}

		false
	}

	pub fn compute_all_piece_bitboards(&mut self) {
		self.all_piece_bitboards = [
			self.piece_bitboards[0][0] | self.piece_bitboards[0][1] | self.piece_bitboards[0][2] | self.piece_bitboards[0][3] | self.piece_bitboards[0][4] | self.piece_bitboards[0][5],
			self.piece_bitboards[1][0] | self.piece_bitboards[1][1] | self.piece_bitboards[1][2] | self.piece_bitboards[1][3] | self.piece_bitboards[1][4] | self.piece_bitboards[1][5],
		];
	}

	pub fn king_in_check(&self, white: bool) -> bool {
		self.piece_bitboards[white as usize][(KING - 1) as usize] & self.attacked_squares_bitboards[!white as usize] != 0
	}

	// This is a performance bottleneck, but I don't know what I'm gonna do about it :[
	pub fn calculate_attacked_squares_bitboards(&mut self) {
		self.attacked_squares_bitboards = [0; 2];

		for i in 0..64 {
			let piece = self.board[i];
			if piece != 0 {
				let piece_color = is_white(piece) as usize;
				self.attacked_squares_bitboards[piece_color] |= match get_piece_type(piece) {
					PAWN   => self.precomputed_data.pawn_bitboards[piece_color][i],
					KNIGHT => self.precomputed_data.knight_bitboards[i],
					BISHOP => self.generate_sliding_attacks_bitboard(i, 4..8),
					ROOK   => self.generate_sliding_attacks_bitboard(i, 0..4),
					QUEEN  => self.generate_sliding_attacks_bitboard(i, 0..8),
					KING   => self.precomputed_data.king_bitboards[i],
					_      => 0,
				};
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
		if self.moves.is_empty() {
			return 0;
		}
		self.moves[self.moves.len() - 1]
	}

	pub fn get_legal_moves_for_color(&mut self, white_pieces: bool, only_captures: bool) -> Vec<u32> {
		let mut result = vec![];

		for i in 0..64 {
			let piece = self.board[i];
			if piece != 0
			&& is_white(piece) == white_pieces {
				result = [result, self.get_legal_moves_for_piece(i, only_captures)].concat();
			}
		}

		result
	}

	pub fn get_legal_moves_for_piece(&mut self, piece_index: usize, only_captures: bool) -> Vec<u32> {
		let piece_color = is_white(self.board[piece_index]) as usize;
		let other_color = !is_white(self.board[piece_index]) as usize;

		let mut result = vec![];

		let piece = 1 << piece_index;

		match get_piece_type(self.board[piece_index]) {
			PAWN => {
				let empty_squares = !(self.all_piece_bitboards[0] | self.all_piece_bitboards[1]);

				if piece_color == 1 {



					let will_promote = rank_of_index(piece_index) == 7;




					if !only_captures {
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




					if !only_captures {
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
				let bitboard = self.precomputed_data.king_bitboards[piece_index] & !(self.attacked_squares_bitboards[other_color] | self.all_piece_bitboards[piece_color]);

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

				if !only_captures {
					let all_pieces_bitboard = self.all_piece_bitboards[0] | self.all_piece_bitboards[1];
					let empty_and_not_attacked_squares = all_pieces_bitboard | self.attacked_squares_bitboards[other_color];

					if self.can_castle_short(piece_color == 1)
					&& SHORT_CASTLE_MASK[piece_color] & empty_and_not_attacked_squares == 0 {
						result.push(build_move(CASTLE_SHORT_FLAG, 0, piece_index, piece_index + 2));
					}

					if self.can_castle_long(piece_color == 1)
					&& LONG_CASTLE_MASK[piece_color] & empty_and_not_attacked_squares == 0
					&& (piece >> 3) & all_pieces_bitboard == 0 {
						result.push(build_move(CASTLE_LONG_FLAG, 0, piece_index, piece_index - 2));
					}
				}
			}

			_ => {}
		};

		for i in (0..result.len()).rev() {
			let m = result[i];
			if only_captures
			&& get_move_capture(m) == 0 {
				result.remove(i);
				continue;
			}

			self.make_move(m);

			if self.king_in_check(!self.whites_turn) {
				result.remove(i);
			}

			self.undo_last_move();
		}

		result
	}

	pub fn play_move(&mut self, promotion: u8, from: usize, to: usize) {
		let promoting = promotion != 0;

		for m in self.get_legal_moves_for_piece(from, false) {
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

	pub fn can_castle_short(&self, is_white: bool) -> bool {
		self.current_castle_rights >> (if is_white { 2 } else { 0 }) & 1 == 1
	}

	pub fn can_castle_long(&self, is_white: bool) -> bool {
		self.current_castle_rights >> (if is_white { 3 } else { 1 }) & 1 == 1
	}

	pub fn make_move(&mut self, piece_move: u32) {
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
			self.toggle_bit(piece_is_white, piece_type, from);
			self.toggle_bit(piece_is_white, flag, to);

			self.board[to] = build_piece(piece_is_white, flag);

			self.total_material_without_pawns += PIECE_WORTH[flag as usize - 1];
		} else {
			self.toggle_bit(piece_is_white, piece_type, to);
			self.toggle_bit(piece_is_white, piece_type, from);
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
		} else if flag == EN_PASSANT_FLAG {
			let pawn_square = if piece_is_white {
				to + 8
			} else {
				to - 8
			};

			self.board[pawn_square] = 0;
			self.toggle_bit(is_white(capture), get_piece_type(capture), pawn_square);
		} else if capture != 0 {
			let captured_piece_type = get_piece_type(capture);
			self.toggle_bit(is_white(capture), captured_piece_type, to);

			if captured_piece_type > PAWN {
				self.total_material_without_pawns -= PIECE_WORTH[captured_piece_type as usize - 1];
			}
		}

		if piece_type == KING {
			self.current_castle_rights &= !CASTLING[piece_is_white as usize];
		} else if from == 0
		|| to == 0 {
			self.current_castle_rights &= !BLACK_LONGCASTLE;
		} else if from == 7
		|| to == 7 {
			self.current_castle_rights &= !BLACK_SHORTCASTLE;
		} else if from == 56
		|| to == 56 {
			self.current_castle_rights &= !WHITE_LONGCASTLE;
		} else if from == 63
		|| to == 63 {
			self.current_castle_rights &= !WHITE_SHORTCASTLE;
		}

		self.compute_all_piece_bitboards();
		self.calculate_attacked_squares_bitboards();

		self.castle_rights_history.push(self.current_castle_rights);
		self.make_move_on_zobrist_key(piece_move);
		self.moves.push(piece_move);


		if piece_type == PAWN
		|| capture != 0 {
			self.current_fifty_move_draw = 0;
		} else {
			self.current_fifty_move_draw += 1;
		}
		self.fifty_move_draw_history.push(self.current_fifty_move_draw);


		self.whites_turn = !self.whites_turn;
	}

	pub fn undo_last_move(&mut self) {
		if self.moves.is_empty() {
			return;
		}


		self.zobrist_key_history.pop();
		self.current_zobrist_key = self.zobrist_key_history[self.zobrist_key_history.len() - 1];

		self.castle_rights_history.pop();
		self.current_castle_rights = self.castle_rights_history[self.castle_rights_history.len() - 1];

		self.fifty_move_draw_history.pop();
		self.current_fifty_move_draw = self.fifty_move_draw_history[self.fifty_move_draw_history.len() - 1];

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

			self.total_material_without_pawns -= PIECE_WORTH[flag as usize - 1];
		} else {
			self.toggle_bit(piece_is_white, piece_type, to);
			self.toggle_bit(piece_is_white, piece_type, from);
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
		} else if flag == EN_PASSANT_FLAG {
			let pawn_square = if piece_is_white {
				to + 8
			} else {
				to - 8
			};

			self.board[pawn_square] = capture;
			self.board[to] = 0;

			self.toggle_bit(is_white(capture), get_piece_type(capture), pawn_square);
		} else if capture != 0 {
			let captured_piece_type = get_piece_type(capture);
			self.toggle_bit(is_white(capture), captured_piece_type, to);

			if captured_piece_type > PAWN {
				self.total_material_without_pawns += PIECE_WORTH[captured_piece_type as usize - 1];
			}
		}


		self.compute_all_piece_bitboards();
		self.calculate_attacked_squares_bitboards();


		self.whites_turn = !self.whites_turn;
	}



	pub fn evaluate(&self) -> i32 {
		let endgame = self.endgame_multiplier();

		let mut white_material = 0;
		let mut black_material = 0;

		for i in 0..64 {
			let piece = self.board[i];

			if piece != 0 {
				let worth = get_full_piece_worth(piece, i, endgame);

				if is_white(piece) {
					white_material += worth;
				} else {
					black_material += worth;
				}
			}
		}

		let perspective = if self.whites_turn { 1 } else { -1 };
		(white_material - black_material) * perspective
	}


	// Returns a value between 0.0 and 1.0 to reflect whether you're in an endgame or not
	// the closer to 1.0, the more of an endgame it is
	pub fn endgame_multiplier(&self) -> f32 {
		clamp(1.5 - self.total_material_without_pawns as f32 * (0.9 / MAX_ENDGAME_MATERIAL as f32), 0.0, 1.0)
	}



	pub fn calculate_initial_zobrist_key(&mut self) {
		for i in 0..64 {
			let piece = self.board[i];
			if piece != 0 {
				self.current_zobrist_key ^= self.zobrist.pieces[get_piece_color(piece) as usize][get_piece_type(piece) as usize - 1][i];
			}
		}

		self.current_zobrist_key ^= self.zobrist.castling_rights[self.current_castle_rights as usize];

		self.current_zobrist_key ^= self.zobrist.en_passant[self.en_passant_file];

		if !self.whites_turn {
			self.current_zobrist_key ^= self.zobrist.side_to_move;
		}

		self.zobrist_key_history.push(self.current_zobrist_key);
	}

	pub fn make_move_on_zobrist_key(&mut self, m: u32) {
		let flag = get_move_flag(m);
		let capture = get_piece_type(get_move_capture(m));
		let from = get_move_from(m);
		let to = get_move_to(m);

		let piece = self.board[to];
		let piece_is_white = is_white(piece);
		let piece_type = get_piece_type(piece) as usize;


		self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][piece_type - 1][from];
		self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][piece_type - 1][to];

		if flag == CASTLE_SHORT_FLAG {
			self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to + 1];
			self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to - 1];
		} else if flag == CASTLE_LONG_FLAG {
			self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to - 2];
			self.current_zobrist_key ^= self.zobrist.pieces[piece_is_white as usize][ROOK as usize - 1][to + 1];
		} else if capture != 0 {
			self.current_zobrist_key ^= self.zobrist.pieces[(!piece_is_white) as usize][capture as usize - 1][to];
		}

		self.current_zobrist_key ^= self.zobrist.castling_rights[self.castle_rights_history[self.castle_rights_history.len() - 2] as usize];
		self.current_zobrist_key ^= self.zobrist.castling_rights[self.current_castle_rights as usize];

		let last_move = self.get_last_move();
		if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
			let file = (get_move_to(last_move) % 8) + 1;
			self.current_zobrist_key ^= self.zobrist.en_passant[file];
		}

		self.en_passant_file = 0;
		if flag == DOUBLE_PAWN_PUSH_FLAG {
			self.en_passant_file = (to % 8) + 1;
			self.current_zobrist_key ^= self.zobrist.en_passant[self.en_passant_file];
		}

		self.current_zobrist_key ^= self.zobrist.side_to_move;


		self.zobrist_key_history.push(self.current_zobrist_key);
	}





	pub fn store_transposition(&mut self, depth: u16, evaluation: i32, best_move: u32, node_type: NodeType) {
		self.transposition_table.insert(self.current_zobrist_key,
			TranspositionData {
				depth,
				evaluation,
				best_move,
				age: 0,
				node_type,
			}
		);
	}

	pub fn lookup_transposition(&mut self, depth: u16, alpha: i32, beta: i32) -> Option<TranspositionData> {
		if let Some(data) = self.transposition_table.get_mut(&self.current_zobrist_key) {
			if data.depth >= depth {
				match data.node_type {
					NodeType::UpperBound => {
						if data.evaluation <= alpha {
							data.age = 0;
							return Some(*data);
						}
					}

					NodeType::LowerBound => {
						if data.evaluation >= beta {
							data.age = 0;
							return Some(*data);
						}
					}

					NodeType::Exact => {
						data.age = 0;
						return Some(*data);
					}
				}
			}
		}
		None
	}

	pub fn update_transposition_table(&mut self) {
		self.transposition_table.retain(|_, data| {
			data.age += 1;
			data.age < 10
		});
	}


	pub fn checkmating_material_on_board(&self) -> bool {
		   self.total_material_without_pawns >= ROOK_WORTH
		|| self.board.contains(&(WHITE | PAWN))
		|| self.board.contains(&(BLACK | PAWN))
	}
}