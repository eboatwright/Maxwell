use std::ops::Range;
use crate::precomputed_data::*;
use crate::utils::*;
use crate::piece::*;

#[derive(Clone)]
pub struct Board {
	pub precomputed_data: PrecomputedData,
	pub piece_bitboards: [[u64; 6]; 2],
	pub all_piece_bitboards: [u64; 2],
	pub attacked_squares_bitboards: [u64; 2],

	pub board: [u8; 64],
	pub whites_turn: bool,

	pub en_passant_capture: Option<usize>,
	pub moves_without_capture_or_pawn_push: u16,
	pub fullmove_counter: u16,

	pub castle_rights_history: Vec<[[bool; 2]; 2]>,
	pub moves: Vec<u32>,
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




		let en_passant_capture = index_from_coordinate(fen_sections[3]);



		let moves_without_capture_or_pawn_push = fen_sections[4].parse::<u16>().expect("Invalid FEN: no 'Halfmove Clock'");
		let fullmove_counter = fen_sections[5].parse::<u16>().expect("Invalid FEN: no 'Fullmove Counter'");



		let mut final_board = Board {
			precomputed_data: PrecomputedData::calculate(),
			piece_bitboards: [[0; 6]; 2],
			all_piece_bitboards: [0; 2],
			attacked_squares_bitboards: [0; 2],

			board,
			whites_turn,

			en_passant_capture,
			moves_without_capture_or_pawn_push,
			fullmove_counter,

			castle_rights_history: vec![
				[
					[fen_sections[2].contains('k'), fen_sections[2].contains('q')],
					[fen_sections[2].contains('K'), fen_sections[2].contains('Q')],
				],
			],
			moves: vec![],
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

				result.push(build_move(0, self.board[to] as u32, piece_from, to));

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
				result.append(&mut self.get_legal_moves_for_piece(i));
			}
		}

		result
	}

	pub fn get_legal_moves_for_piece(&mut self, piece_index: usize) -> Vec<u32> {
		let piece_color = is_white(self.board[piece_index]) as usize;
		let other_color = !is_white(self.board[piece_index]) as usize;
		let piece_type = get_piece_type(self.board[piece_index]);

		let mut result = vec![];

		match piece_type {
			PAWN => {
				let piece = 1 << piece_index;
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
						}

						if rank_of_index(piece_index) == 2
						&& (piece >> 16) & empty_squares != 0 {
							result.push(build_move(DOUBLE_PAWN_PUSH_FLAG, 0, piece_index, piece_index - 16));
						}
					}




					// Capturing
					if (piece >> 7) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index - 7] as u32, piece_index, piece_index - 7));
							}
						} else {
							result.push(build_move(0, self.board[piece_index - 7] as u32, piece_index, piece_index - 7));
						}
					}

					if (piece >> 9) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index - 9] as u32, piece_index, piece_index - 9));
							}
						} else {
							result.push(build_move(0, self.board[piece_index - 9] as u32, piece_index, piece_index - 9));
						}
					}



					// En Passant
					let last_move = self.get_last_move();
					if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
						let pawn_index = get_move_to(last_move);
						if (pawn_index == piece_index - 1
						&& piece_index % 8 != 0)
						|| pawn_index == piece_index + 1 {
							result.push(build_move(EN_PASSANT_FLAG, self.board[pawn_index] as u32, piece_index, pawn_index - 8));
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
						}

						if rank_of_index(piece_index) == 7
						&& (piece << 16) & empty_squares != 0 {
							result.push(build_move(DOUBLE_PAWN_PUSH_FLAG, 0, piece_index, piece_index + 16));
						}
					}




					// Capturing
					if (piece << 7) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index + 7] as u32, piece_index, piece_index + 7));
							}
						} else {
							result.push(build_move(0, self.board[piece_index + 7] as u32, piece_index, piece_index + 7));
						}
					}

					if (piece << 9) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						if will_promote {
							for promotion in PROMOTABLE_PIECES.iter() {
								result.push(build_move(*promotion, self.board[piece_index + 9] as u32, piece_index, piece_index + 9));
							}
						} else {
							result.push(build_move(0, self.board[piece_index + 9] as u32, piece_index, piece_index + 9));
						}
					}



					// En Passant
					let last_move = self.get_last_move();
					if get_move_flag(last_move) == DOUBLE_PAWN_PUSH_FLAG {
						let pawn_index = get_move_to(last_move);
						if pawn_index == piece_index - 1
						|| (pawn_index == piece_index + 1
						&& piece_index % 8 != 7) {
							result.push(build_move(EN_PASSANT_FLAG, self.board[pawn_index] as u32, piece_index, pawn_index + 8));
						}
					}




				}
			}




			KNIGHT => {
				let bitboard = self.precomputed_data.knight_bitboards[piece_index] & !self.all_piece_bitboards[piece_color];

				for i in 0..64 {
					if (bitboard >> i) & 1 == 1 {
						result.push(build_move(0, self.board[i] as u32, piece_index, i));
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
				let bitboard = self.precomputed_data.king_bitboards[piece_index] & !self.all_piece_bitboards[piece_color];

				for i in 0..64 {
					if (bitboard >> i) & 1 == 1 {
						result.push(build_move(0, self.board[i] as u32, piece_index, i));
					}
				}

				if self.can_castle_short(piece_color)
				&& self.board[piece_index + 1] == 0
				&& self.board[piece_index + 2] == 0 {
					result.push(build_move(CASTLE_SHORT_FLAG, 0, piece_index, piece_index + 2));
				}

				if self.can_castle_long(piece_color)
				&& self.board[piece_index - 1] == 0
				&& self.board[piece_index - 2] == 0
				&& self.board[piece_index - 3] == 0 {
					result.push(build_move(CASTLE_LONG_FLAG, 0, piece_index, piece_index - 2));
				}
			}

			_ => {}
		};

		for i in (0..result.len()).rev() {
			let move_flag = get_move_flag(result[i]);

			if (move_flag == CASTLE_SHORT_FLAG
			|| move_flag == CASTLE_LONG_FLAG)
			&& self.king_in_check(self.whites_turn) {
				result.remove(i);
				continue;
			}

			let mut already_removed = false;

			self.make_move(result[i]);

			if self.king_in_check(!self.whites_turn) {
				result.remove(i);
				already_removed = true;
			}

			self.undo_last_move();

			if already_removed {
				continue;
			}

			if move_flag == CASTLE_SHORT_FLAG {
				if self.piece_bitboards[self.whites_turn as usize][(KING - 1) as usize] << 1 & self.attacked_squares_bitboards[!self.whites_turn as usize] != 0 {
					result.remove(i);
				}
			} else if move_flag == CASTLE_LONG_FLAG {
				if self.piece_bitboards[self.whites_turn as usize][(KING - 1) as usize] >> 1 & self.attacked_squares_bitboards[!self.whites_turn as usize] != 0 {
					result.remove(i);
				}
			}
		}

		result
	}

	pub fn play_move(&mut self, promotion: u8, from: usize, to: usize) {
		for m in self.get_legal_moves_for_piece(from) {
			let flag = get_move_flag(m);
			if (!PROMOTABLE_PIECES.contains(&flag)
			|| flag == promotion)
			&& get_move_from(m) == from
			&& get_move_to(m) == to {
				self.make_move(m);
				break;
			}
		}
	}

	fn toggle_bit(&mut self, is_white: bool, piece: u8, index: usize) {
		self.piece_bitboards[is_white as usize][piece as usize - 1] ^= 1 << index;
	}

	pub fn get_all_castle_rights(&self) -> [[bool; 2]; 2] {
		self.castle_rights_history[self.castle_rights_history.len() - 1]
	}

	pub fn can_castle_short(&self, color: usize) -> bool {
		self.get_all_castle_rights()[color][0]
	}

	pub fn can_castle_long(&self, color: usize) -> bool {
		self.get_all_castle_rights()[color][1]
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

		self.board[to] = self.board[from];
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
			new_castle_rights[piece_is_white as usize] = [false, false];
		}

		if from == 0
		|| to == 0 {
			new_castle_rights[0][1] = false;
		} else if from == 7
		|| to == 7 {
			new_castle_rights[0][0] = false;
		} else if from == 56
		|| to == 56 {
			new_castle_rights[1][1] = false;
		} else if from == 63
		|| to == 63 {
			new_castle_rights[1][0] = false;
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
		self.moves.push(piece_move);

		if !self.whites_turn {
			self.moves_without_capture_or_pawn_push += 1;
			self.fullmove_counter += 1;
		}

		self.whites_turn = !self.whites_turn;
	}

	pub fn undo_last_move(&mut self) {
		if self.moves.len() == 0 {
			return;
		}

		self.castle_rights_history.pop();
		let last_move = self.moves.pop().unwrap();

		let flag = get_move_flag(last_move);
		let capture = get_move_capture(last_move);
		let from = get_move_from(last_move);
		let to = get_move_to(last_move);

		let piece = self.board[to];
		let piece_is_white = is_white(piece);
		let piece_type = get_piece_type(piece);

		self.board[from] = self.board[to];
		self.board[to] = capture;

		if PROMOTABLE_PIECES.contains(&flag) {
			self.toggle_bit(piece_is_white, PAWN, from);
			self.toggle_bit(piece_is_white, flag, to);

			self.board[from] = (piece_is_white as u8) << 3 | PAWN;
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

		if !self.whites_turn {
			self.moves_without_capture_or_pawn_push -= 1;
			self.fullmove_counter -= 1;
		}

		self.whites_turn = !self.whites_turn;
	}



	pub fn evaluate(&self) -> i32 {
		let mut white_material = 0;
		let mut black_material = 0;

		for i in 0..64 {
			let piece = self.board[i];

			if piece != 0 {
				let worth = get_piece_worth(piece, i);

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
}