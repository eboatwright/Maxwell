use crate::precomputed_data::*;
use crate::utils::*;
use crate::piece::*;

#[derive(Clone)]
pub struct Board {
	pub precomputed_data: PrecomputedData,
	pub piece_bitboards: [[u64; 6]; 2],
	pub all_piece_bitboards: [u64; 2],

	pub board: [u8; 64],
	pub whites_turn: bool,

	pub white_short_castle_rights: bool,
	pub white_long_castle_rights: bool,
	pub black_short_castle_rights: bool,
	pub black_long_castle_rights: bool,

	pub en_passant_capture: Option<usize>,
	pub moves_without_capture_or_pawn_push: u16,
	pub fullmove_counter: u16,

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



		let white_short_castle_rights = fen_sections[2].contains('K');
		let white_long_castle_rights = fen_sections[2].contains('Q');
		let black_short_castle_rights = fen_sections[2].contains('k');
		let black_long_castle_rights = fen_sections[2].contains('q');



		let en_passant_capture = index_from_coordinate(fen_sections[3]);



		let moves_without_capture_or_pawn_push = fen_sections[4].parse::<u16>().expect("Invalid FEN: no 'Halfmove Clock'");
		let fullmove_counter = fen_sections[5].parse::<u16>().expect("Invalid FEN: no 'Fullmove Counter'");



		let mut final_board = Board {
			precomputed_data: PrecomputedData::initialize(),
			piece_bitboards: [[0; 6]; 2],
			all_piece_bitboards: [0; 2],

			board,
			whites_turn,

			white_short_castle_rights,
			white_long_castle_rights,
			black_short_castle_rights,
			black_long_castle_rights,

			en_passant_capture,
			moves_without_capture_or_pawn_push,
			fullmove_counter,

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


		final_board
	}

	pub fn compute_all_piece_bitboards(&mut self) {
		self.all_piece_bitboards = [
			self.piece_bitboards[0][0] | self.piece_bitboards[0][1] | self.piece_bitboards[0][2] | self.piece_bitboards[0][3] | self.piece_bitboards[0][4] | self.piece_bitboards[0][5],
			self.piece_bitboards[1][0] | self.piece_bitboards[1][1] | self.piece_bitboards[1][2] | self.piece_bitboards[1][3] | self.piece_bitboards[1][4] | self.piece_bitboards[1][5],
		];
	}

	pub fn get_last_move(&self) -> u32 {
		if self.moves.len() == 0 {
			return 0;
		}
		self.moves[self.moves.len() - 1]
	}

	pub fn get_legal_moves_for_piece(&self, piece_index: usize) -> Vec<u32> {
		let piece_color = is_white(self.board[piece_index]) as usize;
		let other_color = !is_white(self.board[piece_index]) as usize;
		let piece_type = get_piece_type(self.board[piece_index]);

		let mut result = vec![];

		match piece_type {
			PAWN => {
				let piece = 1 << piece_index;
				let empty_squares = !(self.all_piece_bitboards[0] | self.all_piece_bitboards[1]);

				if piece_color == 1 {




					// Pushing
					if (piece >> 8) & empty_squares != 0 {
						result.push(build_move(0, 0, piece_index, piece_index - 8));
						if rank_of_index(piece_index) == 2
						&& (piece >> 16) & empty_squares != 0 {
							result.push(build_move(DOUBLE_PAWN_PUSH_FLAG as u32, 0, piece_index, piece_index - 16));
						}
					}




					// Capturing
					if (piece >> 7) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						result.push(build_move(0, self.board[piece_index - 7] as u32, piece_index, piece_index - 7));
					}
					if (piece >> 9) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						result.push(build_move(0, self.board[piece_index - 9] as u32, piece_index, piece_index - 9));
					}




				} else {




					// Pushing
					if (piece << 8) & empty_squares != 0 {
						result.push(build_move(0, 0, piece_index, piece_index + 8));
						if rank_of_index(piece_index) == 7
						&& (piece << 16) & empty_squares != 0 {
							result.push(build_move(DOUBLE_PAWN_PUSH_FLAG as u32, 0, piece_index, piece_index + 16));
						}
					}




					// Capturing
					if (piece << 7) & self.all_piece_bitboards[other_color] & NOT_A_FILE != 0 {
						result.push(build_move(0, self.board[piece_index + 7] as u32, piece_index, piece_index + 7));
					}
					if (piece << 9) & self.all_piece_bitboards[other_color] & NOT_H_FILE != 0 {
						result.push(build_move(0, self.board[piece_index + 9] as u32, piece_index, piece_index + 9));
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

			_ => {}
		};

		result
	}

	pub fn make_move(&mut self, piece_move: u32) {
		let flags = get_move_flag(piece_move);
		let capture = get_move_capture(piece_move);
		let from = get_move_from(piece_move);
		let to = get_move_to(piece_move);

		for m in self.get_legal_moves_for_piece(from) {
			if get_move_to(m) == to {
				self.board[to] = self.board[from];
				self.board[from] = 0;

				let piece = self.board[to];
				let pieces_bitboard = &mut self.piece_bitboards[is_white(piece) as usize][get_piece_type(piece) as usize - 1];
				*pieces_bitboard ^= 1 << to;
				*pieces_bitboard ^= 1 << from;

				if capture != 0 {
					self.piece_bitboards[is_white(capture) as usize][get_piece_type(capture) as usize - 1] ^= 1 << to;
				}

				self.compute_all_piece_bitboards();

				self.moves.push(piece_move);

				self.whites_turn = !self.whites_turn;

				break;
			}
		}
	}

	pub fn undo_last_move(&mut self) {
		if self.moves.len() == 0 {
			return;
		}

		let last_move = self.moves.pop().unwrap();

		let flags = get_move_flag(last_move);
		let capture = get_move_capture(last_move);
		let from = get_move_from(last_move);
		let to = get_move_to(last_move);

		self.board[from] = self.board[to];
		self.board[to] = capture;

		let piece = self.board[from];
		let pieces_bitboard = &mut self.piece_bitboards[is_white(piece) as usize][get_piece_type(piece) as usize - 1];
		*pieces_bitboard ^= 1 << to;
		*pieces_bitboard ^= 1 << from;

		if capture != 0 {
			self.piece_bitboards[is_white(capture) as usize][get_piece_type(capture) as usize - 1] ^= 1 << to;
		}

		self.compute_all_piece_bitboards();

		self.whites_turn = !self.whites_turn;
	}
}