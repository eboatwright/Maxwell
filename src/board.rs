use crate::nnue_weights::*;
use crate::nnue::{self, NNUE, NNUE_EVAL_SCALE};
use crate::value_holder::ValueHolder;
use crate::utils::{pop_lsb, get_lsb, print_bitboard, coordinate_to_index};
use crate::piece_square_tables::{BASE_WORTHS_OF_PIECE_TYPE, get_full_worth_of_piece, ROOK_WORTH, BISHOP_WORTH};
use crate::precalculated_move_data::*;
use crate::move_data::*;
use crate::zobrist::Zobrist;
use crate::pieces::*;
use crate::castling_rights::*;
use colored::Colorize;

pub const MAX_ENDGAME_MATERIAL: f32 = (ROOK_WORTH * 2 + BISHOP_WORTH * 2) as f32;

pub const DOUBLED_PAWN_PENALTY: i32 = 35; // TODO
pub const ISOLATED_PAWN_PENALTY: i32 = 20; // TODO
pub const PASSED_PAWN_BOOST: [i32; 8] = [0, 15, 15, 30, 50, 90, 150, 0]; // TODO

#[derive(Copy, Clone)]
pub struct BoardState {
	pub castling_rights: u8,
	pub fifty_move_counter: u8,
	pub attacked_squares: [Option<u64>; 2],
}

impl BoardState {
	pub fn new(castling_rights: u8, fifty_move_counter: u8) -> Self {
		Self {
			castling_rights,
			fifty_move_counter,
			attacked_squares: [None, None],
		}
	}
}

pub struct Board {
	pub precalculated_move_data: PrecalculatedMoveData,

	pub piece_bitboards: [u64; PIECE_COUNT],
	pub color_bitboards: [u64; 2],

	pub en_passant_file: usize,
	pub white_to_move: bool,

	pub total_material_without_pawns: [i32; 2],

	pub zobrist: Zobrist,

	pub moves: Vec<MoveData>,

	// TODO: smaller version of the transposition table for evaluation cache?

	pub board_state: ValueHolder<BoardState>,

	pub nnue: NNUE,
}

impl Board {
	// Pieces, side to move, castling rights, en passant square, fifty move draw, fullmove counter
	pub fn from_fen(
		fen: &str,
		nnue_hidden_layer_weights: Vec<f32>,
		nnue_hidden_layer_biases: Vec<f32>,
		nnue_output_layer_weights: Vec<f32>,
		nnue_output_layer_biases: Vec<f32>,
	) -> Self {
		let fen = fen.split(' ').collect::<Vec<&str>>();

		let mut castling_rights = 0b0000;
		if fen[2].contains('Q') { castling_rights ^= WHITE_CASTLE_LONG; }
		if fen[2].contains('K') { castling_rights ^= WHITE_CASTLE_SHORT; }
		if fen[2].contains('q') { castling_rights ^= BLACK_CASTLE_LONG; }
		if fen[2].contains('k') { castling_rights ^= BLACK_CASTLE_SHORT; }

		let fifty_move_counter = fen[4].parse::<u8>().unwrap_or(0);

		let mut board = Self {
			precalculated_move_data: PrecalculatedMoveData::calculate(),

			piece_bitboards: [0; PIECE_COUNT],
			color_bitboards: [0; 2],

			en_passant_file: 0, // This isn't implemented at all
			// en_passant_file: if fen[3] == "-" { 0 } else { (coordinate_to_index(fen[3]) % 8) + 1 },
			white_to_move: fen[1] == "w",

			total_material_without_pawns: [0, 0],

			zobrist: Zobrist::default(),

			moves: vec![],

			board_state: ValueHolder::new(BoardState::new(castling_rights, fifty_move_counter)),

			nnue: NNUE::new(vec![], vec![], vec![], vec![]),
		};

		let piece_rows = fen[0].split('/').collect::<Vec<&str>>();
		let mut i = 0;

		for row in piece_rows {
			for piece in row.chars() {
				if let Ok(empty_squares) = piece.to_string().parse::<usize>() {
					i += empty_squares;
				} else {
					let piece = char_to_piece(piece);

					board.piece_bitboards[piece] |= 1 << i;
					board.color_bitboards[is_piece_white(piece) as usize] |= 1 << i;
					i += 1;

					let piece_is_white = is_piece_white(piece);
					let piece_type = get_piece_type(piece);

					if piece_type != PAWN
					&& piece_type != KING {
						let piece_worth = BASE_WORTHS_OF_PIECE_TYPE[piece_type];
						board.total_material_without_pawns[piece_is_white as usize] += piece_worth;
					}
				}
			}
		}

		board.zobrist = Zobrist::generate(&board);
		board.calculate_attacked_squares();

		board.nnue = NNUE::initialize(
			&board,

			nnue_hidden_layer_weights,
			nnue_hidden_layer_biases,
			nnue_output_layer_weights,
			nnue_output_layer_biases,
		);

		board
	}

	pub fn calculate_attacked_squares(&mut self) {
		self.calculate_attacked_squares_for_color(0);
		self.calculate_attacked_squares_for_color(1);
	}

	pub fn get_attacked_squares_for_color(&mut self, color: usize) -> u64 {
		self.calculate_attacked_squares_for_color(color);
		self.board_state.current.attacked_squares[color].unwrap()
	}

	// This is SLOOOOOOOOOOOOOWWWWWWW :[
	pub fn calculate_attacked_squares_for_color(&mut self, color: usize) {
		if self.board_state.current.attacked_squares[color].is_some() {
			return;
		}

		let mut attacked_squares = 0;

		for piece_type in PAWN..=KING {
			let piece = build_piece(color == 1, piece_type);
			let mut pieces_bitboard = self.piece_bitboards[piece];

			while pieces_bitboard != 0 {
				let piece_index = pop_lsb(&mut pieces_bitboard) as usize;

				attacked_squares |= match piece_type {
					PAWN   => self.precalculated_move_data.pawn_attacks[color][piece_index],
					KNIGHT => self.precalculated_move_data.knight_attacks[piece_index],
					BISHOP => self.calculate_bishop_attack_bitboard(piece_index),
					ROOK   => self.calculate_rook_attack_bitboard(piece_index),
					QUEEN  => self.calculate_queen_attack_bitboard(piece_index),
					KING   => self.precalculated_move_data.king_attacks[piece_index],
					_ => 0,
				};
			}
		}

		self.board_state.current.attacked_squares[color] = Some(attacked_squares);
	}

	pub fn print(&self) {
		let mut output = String::new();

		output += "---------------------------------\n";
		for rank in 0..8 {
			for file in 0..8 {
				let i = file + rank * 8;

				let mut c = " ".normal();
				for piece in 0..PIECE_COUNT {
					if self.piece_bitboards[piece] & (1 << i) != 0 {
						c = if is_piece_white(piece) {
							piece_to_char(piece).to_string().bold().italic().black().on_white()
						} else {
							piece_to_char(piece).to_string().bold().italic().white().on_black()
						};
						break;
					}
				}

				output += &format!("| {} ", c);
			}
			output += "|\n---------------------------------\n";
		}
		output.pop(); // remove the last new line

		println!("{}", output);
	}

	pub fn print_bitboards(&mut self) {
		for piece in 0..PIECE_COUNT {
			let c = piece_to_char(piece);
			print_bitboard(
				&format!("{}", c),
				if c.is_uppercase() {
					"1".bold().italic().normal().on_white()
				} else {
					"1".bold().italic().white().on_black()
				},
				self.piece_bitboards[piece],
			);
		}

		print_bitboard("Black pieces", "1".bold().italic().white().on_black(), self.color_bitboards[0]);
		print_bitboard("White pieces", "1".bold().italic().normal().on_white(), self.color_bitboards[1]);

		let black_attacked_squares = self.get_attacked_squares_for_color(0);
		let white_attacked_squares = self.get_attacked_squares_for_color(1);
		print_bitboard("Black attacked squares", "1".bold().italic().white().on_black(), black_attacked_squares);
		print_bitboard("White attacked squares", "1".bold().italic().normal().on_white(), white_attacked_squares);
	}

	pub fn get_last_move(&self) -> MoveData {
		if self.moves.is_empty() {
			return NULL_MOVE;
		}
		self.moves[self.moves.len() - 1]
	}

	pub fn get_piece(&self, i: u8) -> usize {
		// if self.unoccupied_bitboard() & (1 << i) != 0 {
		// 	return NO_PIECE;
		// }

		for piece in 0..PIECE_COUNT {
			if self.piece_bitboards[piece] & (1 << i) != 0 {
				return piece;
			}
		}
		NO_PIECE
	}

	pub fn occupied_bitboard(&self) -> u64 { self.color_bitboards[0] | self.color_bitboards[1] }
	pub fn unoccupied_bitboard(&self) -> u64 { !self.occupied_bitboard() }

	pub fn square_is_empty(&self, i: u8) -> bool {
		self.unoccupied_bitboard() & (1 << i) != 0
	}

	pub fn play_move(&mut self, data: MoveData) -> bool {
		let promoting = PROMOTABLE.contains(&data.flag);
		if self.white_to_move == is_piece_white(self.get_piece(data.from)) {
			let moves = self.get_moves_for_piece(data.from, false);
			for m in moves {
				if (!promoting || data.flag == m.flag)
				&& m.from == data.from
				&& m.to == data.to
				&& self.make_move(m) {
					return true;
				}
			}
		}

		println!("Illegal move: {}", data.to_coordinates());
		false
	}

	pub fn make_move(&mut self, data: MoveData) -> bool {
		let piece_is_white = is_piece_white(data.piece as usize);
		let piece_color = piece_is_white as usize;
		let other_color = (!piece_is_white) as usize;

		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if !PROMOTABLE.contains(&data.flag) {
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		} else {
			self.piece_bitboards[build_piece(piece_is_white, data.flag as usize)] ^= 1 << data.to;
			self.total_material_without_pawns[piece_color] += BASE_WORTHS_OF_PIECE_TYPE[data.flag as usize];
		}

		self.color_bitboards[piece_color] ^= 1 << data.from;
		self.color_bitboards[piece_color] ^= 1 << data.to;

		if data.capture != NO_PIECE as u8 {
			let capture_type = get_piece_type(data.capture as usize);
			if capture_type != PAWN {
				self.total_material_without_pawns[other_color] -= BASE_WORTHS_OF_PIECE_TYPE[capture_type];
			}

			if data.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(data.piece as usize) {
						data.to + 8
					} else {
						data.to - 8
					};

				self.piece_bitboards[data.capture as usize] ^= 1 << en_passant_square;
				self.color_bitboards[other_color] ^= 1 << en_passant_square;
			} else {
				self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
				self.color_bitboards[other_color] ^= 1 << data.to;
			}
		}

		// I dunno if there's a better way to do this :/
		if data.piece == WHITE_KING as u8 {
			self.board_state.current.castling_rights &= !ALL_WHITE_CASTLING_RIGHTS;

			if data.flag == SHORT_CASTLE_FLAG {
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 63;
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 61;

				self.color_bitboards[1] ^= 1 << 63;
				self.color_bitboards[1] ^= 1 << 61;
			} else if data.flag == LONG_CASTLE_FLAG {
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 56;
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 59;

				self.color_bitboards[1] ^= 1 << 56;
				self.color_bitboards[1] ^= 1 << 59;
			}
		} else if data.piece == BLACK_KING as u8 {
			self.board_state.current.castling_rights &= !ALL_BLACK_CASTLING_RIGHTS;

			if data.flag == SHORT_CASTLE_FLAG {
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 7;
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 5;

				self.color_bitboards[0] ^= 1 << 7;
				self.color_bitboards[0] ^= 1 << 5;
			} else if data.flag == LONG_CASTLE_FLAG {
				self.piece_bitboards[BLACK_ROOK] ^= 1; // << 0
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 3;

				self.color_bitboards[0] ^= 1;
				self.color_bitboards[0] ^= 1 << 3;
			}
		}

		// This is so ugly :`(
		if data.from == 0 {
			self.board_state.current.castling_rights &= !BLACK_CASTLE_LONG;
		} else if data.from == 7 {
			self.board_state.current.castling_rights &= !BLACK_CASTLE_SHORT;
		} else if data.from == 56 {
			self.board_state.current.castling_rights &= !WHITE_CASTLE_LONG;
		} else if data.from == 63 {
			self.board_state.current.castling_rights &= !WHITE_CASTLE_SHORT;
		}

		if data.to == 0 {
			self.board_state.current.castling_rights &= !BLACK_CASTLE_LONG;
		} else if data.to == 7 {
			self.board_state.current.castling_rights &= !BLACK_CASTLE_SHORT;
		} else if data.to == 56 {
			self.board_state.current.castling_rights &= !WHITE_CASTLE_LONG;
		} else if data.to == 63 {
			self.board_state.current.castling_rights &= !WHITE_CASTLE_SHORT;
		}

		if data.capture != NO_PIECE as u8
		|| get_piece_type(data.piece as usize) == PAWN {
			self.board_state.current.fifty_move_counter = 0;
		} else {
			self.board_state.current.fifty_move_counter += 1;
		}

		self.board_state.current.attacked_squares = [None; 2];
		self.board_state.push();

		self.zobrist.make_move(
			data,
			self.get_last_move(),
			self.board_state.current.castling_rights,
			self.board_state.history[self.board_state.index - 1].castling_rights,
		);

		// self.nnue.make_move(&data);

		self.moves.push(data);
		self.white_to_move = !self.white_to_move;

		if self.king_in_check(!self.white_to_move) {
			self.undo_last_move();
			return false;
		}

		true
	}

	pub fn undo_last_move(&mut self) -> bool {
		if self.moves.is_empty() {
			return false;
		}
		let last_move = self.moves.pop().unwrap();

		let piece_is_white = is_piece_white(last_move.piece as usize);
		let piece_color = piece_is_white as usize;
		let other_color = (!piece_is_white) as usize;

		self.piece_bitboards[last_move.piece as usize] ^= 1 << last_move.from;

		if !PROMOTABLE.contains(&last_move.flag) {
			self.piece_bitboards[last_move.piece as usize] ^= 1 << last_move.to;
		} else {
			self.piece_bitboards[build_piece(piece_is_white, last_move.flag as usize)] ^= 1 << last_move.to;
			self.total_material_without_pawns[piece_color] -= BASE_WORTHS_OF_PIECE_TYPE[last_move.flag as usize];
		}

		self.color_bitboards[piece_color] ^= 1 << last_move.from;
		self.color_bitboards[piece_color] ^= 1 << last_move.to;

		if last_move.capture != NO_PIECE as u8 {
			let capture_type = get_piece_type(last_move.capture as usize);
			if capture_type != PAWN {
				self.total_material_without_pawns[other_color] += BASE_WORTHS_OF_PIECE_TYPE[capture_type];
			}

			if last_move.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(last_move.piece as usize) {
						last_move.to + 8
					} else {
						last_move.to - 8
					};

				self.piece_bitboards[last_move.capture as usize] ^= 1 << en_passant_square;
				self.color_bitboards[other_color] ^= 1 << en_passant_square;
			} else {
				self.piece_bitboards[last_move.capture as usize] ^= 1 << last_move.to;
				self.color_bitboards[other_color] ^= 1 << last_move.to;
			}
		} else if last_move.flag == SHORT_CASTLE_FLAG {
			if piece_color == 1 {
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 63;
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 61;

				self.color_bitboards[1] ^= 1 << 63;
				self.color_bitboards[1] ^= 1 << 61;
			} else {
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 7;
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 5;

				self.color_bitboards[0] ^= 1 << 7;
				self.color_bitboards[0] ^= 1 << 5;
			}
		} else if last_move.flag == LONG_CASTLE_FLAG {
			if piece_color == 1 {
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 56;
				self.piece_bitboards[WHITE_ROOK] ^= 1 << 59;

				self.color_bitboards[1] ^= 1 << 56;
				self.color_bitboards[1] ^= 1 << 59;
			} else {
				self.piece_bitboards[BLACK_ROOK] ^= 1; // << 0
				self.piece_bitboards[BLACK_ROOK] ^= 1 << 3;

				self.color_bitboards[0] ^= 1;
				self.color_bitboards[0] ^= 1 << 3;
			}
		}

		self.board_state.pop();
		self.zobrist.key.pop();

		// self.nnue.undo_move(&last_move);

		self.white_to_move = !self.white_to_move;

		true
	}

	pub fn king_in_check(&mut self, king_is_white: bool) -> bool {
		let attacked_squares = self.get_attacked_squares_for_color((!king_is_white) as usize);
		self.piece_bitboards[build_piece(king_is_white, KING)] & attacked_squares != 0
	}

	pub fn get_pseudo_legal_moves_for_color(&mut self, white_pieces: bool, only_captures: bool) -> Vec<MoveData> {
		let mut result = vec![];

		let pieces =
			if white_pieces {
				WHITE_PAWN..=WHITE_KING
			} else {
				BLACK_PAWN..=BLACK_KING
			};

		for piece in pieces {
			let mut bitboard = self.piece_bitboards[piece];

			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard);
				result.append(&mut self.get_moves_for_piece(piece_index, only_captures));
			}
		}

		// for i in (0..result.len()).rev() {
		// 	self.make_move(result[i]);

		// 	if self.king_in_check(!self.white_to_move) {
		// 		result.remove(i);
		// 	}

		// 	self.undo_last_move();
		// }

		result
	}

	pub fn get_moves_for_piece(&mut self, piece_index: u8, only_captures: bool) -> Vec<MoveData> {
		let mut result = vec![];

		let piece = self.get_piece(piece_index);
		let piece_is_white = is_piece_white(piece);
		let piece_type = get_piece_type(piece);

		match piece_type {
			PAWN => {
				if piece_is_white {


					let rank = piece_index / 8;
					let will_promote = rank == 1;


					// Pushing
					if !only_captures
					&& self.square_is_empty(piece_index - 8) {
						if will_promote {
							for promotion in KNIGHT..=QUEEN {
								result.push(
									MoveData {
										flag: promotion as u8,
										capture: NO_PIECE as u8,
										piece: piece as u8,
										from: piece_index,
										to: piece_index - 8,
									},
								);
							}
						} else {
							result.push(
								MoveData {
									flag: 0,
									capture: NO_PIECE as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index - 8,
								},
							);

							if rank == 6
							&& self.square_is_empty(piece_index - 16) {
								result.push(
									MoveData {
										flag: DOUBLE_PAWN_PUSH_FLAG,
										capture: NO_PIECE as u8,
										piece: piece as u8,
										from: piece_index,
										to: piece_index - 16,
									},
								);
							}
						}
					}

					// Captures
					let mut capture_bitboard =
						  self.precalculated_move_data.pawn_attacks[1][piece_index as usize]
						& self.color_bitboards[0];
					while capture_bitboard != 0 {
						let capture_index = pop_lsb(&mut capture_bitboard);
						if will_promote {
							for promotion in KNIGHT..=QUEEN {
								result.push(
									MoveData {
										flag: promotion as u8,
										capture: self.get_piece(capture_index) as u8,
										piece: piece as u8,
										from: piece_index,
										to: capture_index,
									},
								);
							}
						} else {
							result.push(
								MoveData {
									flag: 0,
									capture: self.get_piece(capture_index) as u8,
									piece: piece as u8,
									from: piece_index,
									to: capture_index,
								},
							);
						}
					}

					// En passant
					let last_move = self.get_last_move();
					if last_move.flag == DOUBLE_PAWN_PUSH_FLAG {
						if piece_index % 8 != 0
						&& last_move.to == piece_index - 1 { // Left
							result.push(
								MoveData {
									flag: EN_PASSANT_FLAG,
									capture: self.get_piece(piece_index - 1) as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index - 9,
								},
							);
						} else if piece_index % 8 != 7
						&& last_move.to == piece_index + 1 { // Right
							result.push(
								MoveData {
									flag: EN_PASSANT_FLAG,
									capture: self.get_piece(piece_index + 1) as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index - 7,
								},
							);
						}
					}




				} else {


					let rank = piece_index / 8;
					let will_promote = rank == 6;


					// Pushing
					if !only_captures
					&& self.square_is_empty(piece_index + 8) {
						if will_promote {
							for promotion in KNIGHT..=QUEEN {
								result.push(
									MoveData {
										flag: promotion as u8,
										capture: NO_PIECE as u8,
										piece: piece as u8,
										from: piece_index,
										to: piece_index + 8,
									},
								);
							}
						} else {
							result.push(
								MoveData {
									flag: 0,
									capture: NO_PIECE as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index + 8,
								},
							);

							if rank == 1
							&& self.square_is_empty(piece_index + 16) {
								result.push(
									MoveData {
										flag: DOUBLE_PAWN_PUSH_FLAG,
										capture: NO_PIECE as u8,
										piece: piece as u8,
										from: piece_index,
										to: piece_index + 16,
									},
								);
							}
						}
					}

					// Captures
					let mut capture_bitboard =
						  self.precalculated_move_data.pawn_attacks[0][piece_index as usize]
						& self.color_bitboards[1];
					while capture_bitboard != 0 {
						let capture_index = pop_lsb(&mut capture_bitboard);
						if will_promote {
							for promotion in KNIGHT..=QUEEN {
								result.push(
									MoveData {
										flag: promotion as u8,
										capture: self.get_piece(capture_index) as u8,
										piece: piece as u8,
										from: piece_index,
										to: capture_index,
									},
								);
							}
						} else {
							result.push(
								MoveData {
									flag: 0,
									capture: self.get_piece(capture_index) as u8,
									piece: piece as u8,
									from: piece_index,
									to: capture_index,
								},
							);
						}
					}

					// En passant
					let last_move = self.get_last_move();
					if last_move.flag == DOUBLE_PAWN_PUSH_FLAG {
						if piece_index % 8 != 0
						&& last_move.to == piece_index - 1 { // Left
							result.push(
								MoveData {
									flag: EN_PASSANT_FLAG,
									capture: self.get_piece(piece_index - 1) as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index + 7,
								},
							);
						} else if piece_index % 8 != 7
						&& last_move.to == piece_index + 1 { // Right
							result.push(
								MoveData {
									flag: EN_PASSANT_FLAG,
									capture: self.get_piece(piece_index + 1) as u8,
									piece: piece as u8,
									from: piece_index,
									to: piece_index + 9,
								},
							);
						}
					}


				}
			}

			KNIGHT => {
				let mut bitboard =
					   self.precalculated_move_data.knight_attacks[piece_index as usize]
					& !self.color_bitboards[piece_is_white as usize];

				if only_captures {
					bitboard &= self.piece_bitboards[!piece_is_white as usize];
				}

				while bitboard != 0 {
					let to = pop_lsb(&mut bitboard);
					result.push(
						MoveData {
							flag: 0,
							capture: self.get_piece(to) as u8,
							piece: piece as u8,
							from: piece_index,
							to,
						},
					);
				}
			}

			BISHOP => {
				let mut moves_bitboard =
					   self.calculate_bishop_attack_bitboard(piece_index as usize)
					& !self.color_bitboards[piece_is_white as usize];

				if only_captures {
					moves_bitboard &= self.piece_bitboards[!piece_is_white as usize];
				}

				while moves_bitboard != 0 {
					let to = pop_lsb(&mut moves_bitboard);
					result.push(
						MoveData {
							flag: 0,
							capture: self.get_piece(to) as u8,
							piece: piece as u8,
							from: piece_index,
							to,
						}
					);
				}
			}

			ROOK => {
				let mut moves_bitboard =
					   self.calculate_rook_attack_bitboard(piece_index as usize)
					& !self.color_bitboards[piece_is_white as usize];

				if only_captures {
					moves_bitboard &= self.piece_bitboards[!piece_is_white as usize];
				}

				while moves_bitboard != 0 {
					let to = pop_lsb(&mut moves_bitboard);
					result.push(
						MoveData {
							flag: 0,
							capture: self.get_piece(to) as u8,
							piece: piece as u8,
							from: piece_index,
							to,
						}
					);
				}
			}

			QUEEN => {
				let mut moves_bitboard =
					   self.calculate_queen_attack_bitboard(piece_index as usize)
					& !self.color_bitboards[piece_is_white as usize];

				if only_captures {
					moves_bitboard &= self.piece_bitboards[!piece_is_white as usize];
				}

				while moves_bitboard != 0 {
					let to = pop_lsb(&mut moves_bitboard);
					result.push(
						MoveData {
							flag: 0,
							capture: self.get_piece(to) as u8,
							piece: piece as u8,
							from: piece_index,
							to,
						}
					);
				}
			}

			KING => {
				let mut bitboard =
					   self.precalculated_move_data.king_attacks[piece_index as usize]
					& !self.color_bitboards[piece_is_white as usize];

				if only_captures {
					bitboard &= self.piece_bitboards[!piece_is_white as usize];
				} else {
					if self.can_short_castle(piece_is_white) {
						result.push(
							MoveData {
								flag: SHORT_CASTLE_FLAG,
								capture: NO_PIECE as u8,
								piece: piece as u8,
								from: piece_index,
								to: piece_index + 2,
							},
						);
					}

					if self.can_long_castle(piece_is_white) {
						result.push(
							MoveData {
								flag: LONG_CASTLE_FLAG,
								capture: NO_PIECE as u8,
								piece: piece as u8,
								from: piece_index,
								to: piece_index - 2,
							},
						);
					}
				}

				while bitboard != 0 {
					let to = pop_lsb(&mut bitboard);
					result.push(
						MoveData {
							flag: 0,
							capture: self.get_piece(to) as u8,
							piece: piece as u8,
							from: piece_index,
							to,
						},
					);
				}
			}

			_ => {}
		}

		result
	}

	fn calculate_bishop_attack_bitboard(&self, piece_index: usize) -> u64 {
		let relevant_occupied_squares =
			self.occupied_bitboard()
			& self.precalculated_move_data.bishop_relevant_occupancy_masks[piece_index];
		let key = self.precalculated_move_data.generate_bishop_key(piece_index, relevant_occupied_squares);
		self.precalculated_move_data.bishop_attacks[piece_index][key]
	}

	fn calculate_rook_attack_bitboard(&self, piece_index: usize) -> u64 {
		let relevant_occupied_squares =
			self.occupied_bitboard()
			& self.precalculated_move_data.rook_relevant_occupancy_masks[piece_index];
		let key = self.precalculated_move_data.generate_rook_key(piece_index, relevant_occupied_squares);
		self.precalculated_move_data.rook_attacks[piece_index][key]
	}

	fn calculate_queen_attack_bitboard(&self, piece_index: usize) -> u64 {
		  self.calculate_bishop_attack_bitboard(piece_index)
		| self.calculate_rook_attack_bitboard(piece_index)
	}

	// Returns a value between 0.0 and 1.0 to reflect whether you're in an endgame or not
	// the closer to 1.0, the more of an endgame it is
	pub fn endgame_multiplier(&self) -> f32 {
		(1.5 - self.total_material_without_pawns.iter().sum::<i32>() as f32 * (0.9 / MAX_ENDGAME_MATERIAL)).clamp(0.0, 1.0)
		// (1.0 - self.total_material_without_pawns as f32 * (1.0 / MAX_ENDGAME_MATERIAL)).clamp(0.0, 1.0)
	}

	pub fn perspective(&self) -> i32 { if self.white_to_move { 1 } else { -1 } }

	pub fn hc_evaluate(&mut self) -> i32 {
		let endgame = self.endgame_multiplier();

		let mut white_material = 0;
		let mut black_material = 0;

		let mut white_pawn_evaluation = 0;
		let mut black_pawn_evaluation = 0;

		for piece in 0..PIECE_COUNT {
			let piece_is_white = is_piece_white(piece);
			let piece_type = get_piece_type(piece);

			let mut bitboard = self.piece_bitboards[piece];
			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard) as usize;

				if piece_is_white {
					white_material += get_full_worth_of_piece(piece, piece_index, endgame);

					if piece_type == PAWN {
						if self.precalculated_move_data.file_of_square[piece_index] & self.piece_bitboards[WHITE_PAWN] != 0 { // Doubled pawn
							white_pawn_evaluation -= DOUBLED_PAWN_PENALTY;
						}

						if self.precalculated_move_data.files_beside_square[piece_index] & self.piece_bitboards[WHITE_PAWN] == 0 { // Isolated pawn
							white_pawn_evaluation -= ISOLATED_PAWN_PENALTY;
						}

						if self.precalculated_move_data.squares_ahead_of_pawn[1][piece_index] & self.piece_bitboards[BLACK_PAWN] == 0
						&& self.precalculated_move_data.file_in_front_of_pawn[1][piece_index] & self.piece_bitboards[WHITE_PAWN] == 0 { // Passed pawn
							white_pawn_evaluation += PASSED_PAWN_BOOST[7 - piece_index / 8];
						}
					}
				} else {
					black_material += get_full_worth_of_piece(piece, piece_index, endgame);

					if piece_type == PAWN {
						if self.precalculated_move_data.file_of_square[piece_index] & self.piece_bitboards[BLACK_PAWN] != 0 { // Doubled pawn
							black_pawn_evaluation -= DOUBLED_PAWN_PENALTY;
						}

						if self.precalculated_move_data.files_beside_square[piece_index] & self.piece_bitboards[BLACK_PAWN] == 0 { // Isolated pawn
							black_pawn_evaluation -= ISOLATED_PAWN_PENALTY;
						}

						if self.precalculated_move_data.squares_ahead_of_pawn[0][piece_index] & self.piece_bitboards[WHITE_PAWN] == 0
						&& self.precalculated_move_data.file_in_front_of_pawn[0][piece_index] & self.piece_bitboards[BLACK_PAWN] == 0 { // Passed pawn
							black_pawn_evaluation += PASSED_PAWN_BOOST[piece_index / 8];
						}
					}
				}
			}
		}

		let pawn_evaluation_multiplier = (endgame + 0.3).clamp(0.3, 1.0); // TODO
		white_pawn_evaluation = (white_pawn_evaluation as f32 * pawn_evaluation_multiplier) as i32;
		black_pawn_evaluation = (black_pawn_evaluation as f32 * pawn_evaluation_multiplier) as i32;

		let white_attacks_bitboard = self.get_attacked_squares_for_color(1);
		let black_attacks_bitboard = self.get_attacked_squares_for_color(0);

		// Taking the sqrt of this made it worse
		let white_attacks_score = white_attacks_bitboard.count_ones() as i32 * 10;
		let black_attacks_score = black_attacks_bitboard.count_ones() as i32 * 10;

		let white_king_index = get_lsb(self.piece_bitboards[WHITE_KING]) as usize;
		let black_king_index = get_lsb(self.piece_bitboards[BLACK_KING]) as usize;

		// TODO: weak squares, weak lines, or none?
		// TODO: Or count how many friendly pieces are around the king?
		let white_king_weakness_penalty = ((
				  self.precalculated_move_data.king_attacks[white_king_index]
				& black_attacks_bitboard
			).count_ones() as f32 * (1.0 - endgame)) as i32 * 20;

		let black_king_weakness_penalty = ((
				  self.precalculated_move_data.king_attacks[black_king_index]
				& white_attacks_bitboard
			).count_ones() as f32 * (1.0 - endgame)) as i32 * 20;

		// let weak_lines_from_white_king = (self.calculate_queen_attack_bitboard(white_king_index).count_ones() as f32 * (1.0 - endgame)) as i32;
		// let weak_lines_from_black_king = (self.calculate_queen_attack_bitboard(black_king_index).count_ones() as f32 * (1.0 - endgame)) as i32;

		// TODO: a small boost for having the bishop pair?

		// TODO: rooks on open lines

		 ((white_material + white_attacks_score - white_king_weakness_penalty + white_pawn_evaluation)
		- (black_material + black_attacks_score - black_king_weakness_penalty + black_pawn_evaluation)) * self.perspective()
	}

	pub fn raw_nnue_evaluate(&self) -> f32 {
		self.nnue.evaluate() // self.occupied_bitboard().count_ones() as usize
	}

	pub fn nnue_evaluate(&self) -> i32 {
		(self.nnue.evaluate() * NNUE_EVAL_SCALE) as i32 * self.perspective()
	}

	pub fn can_short_castle(&mut self, white: bool) -> bool {
		// self.king_in_check calculates attacked squares
		   !self.king_in_check(white)
		&&  self.board_state.current.castling_rights & SHORT_CASTLING_RIGHTS[white as usize] != 0
		&& (self.occupied_bitboard() | self.board_state.current.attacked_squares[(!white) as usize].unwrap()) & SHORT_CASTLE_MASK[white as usize] == 0
	}

	pub fn can_long_castle(&mut self, white: bool) -> bool {
		let occupied = self.occupied_bitboard();
		   !self.king_in_check(white)
		&&  self.board_state.current.castling_rights & LONG_CASTLING_RIGHTS[white as usize] != 0
		&&  EXTRA_LONG_CASTLE_SQUARE_CHECK[white as usize] & occupied == 0
		&& (occupied | self.board_state.current.attacked_squares[(!white) as usize].unwrap()) & LONG_CASTLE_MASK[white as usize] == 0
	}

	pub fn insufficient_checkmating_material(&self) -> bool {
		   self.total_material_without_pawns.iter().sum::<i32>() < ROOK_WORTH
		&& self.piece_bitboards[WHITE_PAWN] == 0
		&& self.piece_bitboards[BLACK_PAWN] == 0
	}

	// Must be called when not in check!
	pub fn try_null_move(&mut self) -> bool {
		if self.get_last_move() == NULL_MOVE {
			return false;
		}

		self.white_to_move = !self.white_to_move;
		self.zobrist.make_null_move();
		self.moves.push(NULL_MOVE);

		self.board_state.current.fifty_move_counter = 0;
		self.board_state.push();

		true
	}

	pub fn undo_null_move(&mut self) {
		self.white_to_move = !self.white_to_move;
		self.zobrist.key.pop();
		self.moves.pop();

		self.board_state.pop();
	}

	// Counting only one repetition as a draw seems to perform better than detecting a threefold repetition
	pub fn is_repetition(&self) -> bool {
		// Before the third move, it's impossible to have a repetition
		if self.zobrist.key.index < 2 {
			return false;
		}

		let lookback = self.zobrist.key.index - self.board_state.current.fifty_move_counter as usize;
		let mut i = self.zobrist.key.index - 2;

		while i >= lookback {
			if self.zobrist.key.history[i] == self.zobrist.key.current {
				return true;
			}

			if i < 2 {
				break;
			}

			i -= 2;
		}

		false
	}

	pub fn is_draw(&self) -> bool {
		   self.board_state.current.fifty_move_counter >= 100
		|| self.insufficient_checkmating_material()
		|| self.is_repetition()
	}

	// Only calculates the pieces section of it for now
	pub fn calculate_fen(&self) -> String {
		let mut result = "".to_string();

		for rank in 0..8 {
			let mut empty_spaces = 0;
			for file in 0..8 {
				let index = rank * 8 + file;

				let piece = self.get_piece(index);
				if piece != NO_PIECE {
					if empty_spaces > 0 {
						result = format!("{}{}", result, empty_spaces);
						empty_spaces = 0;
					}

					result = format!("{}{}", result, piece_to_char(piece));
				} else {
					empty_spaces += 1;
				}
			}

			if empty_spaces > 0 {
				result = format!("{}{}", result, empty_spaces);
			}

			result += "/";
		}

		result.pop();

		result
	}

	// This is used in NNUE training
	pub fn get_winner(&mut self) -> Option<f32> {
		if self.is_draw() {
			return Some(0.0);
		}

		let moves = self.get_pseudo_legal_moves_for_color(self.white_to_move, false);
		for m in moves {
			if !self.make_move(m) {
				continue;
			}

			self.undo_last_move();

			return None; // There is a legal move available, so it's not checkmate or stalemate
		}

		if self.king_in_check(self.white_to_move) {
			return Some(if self.white_to_move { -1.0 } else { 1.0 }); // Checkmate
		}

		return Some(0.0); // Stalemate
	}

	// This isn't used anywhere and I haven't tested it, so it might be bugged
	pub fn square_is_attacked_by_color(&self, square_bitboard: u64, white_pieces: bool) -> bool {
		let color = white_pieces as usize;

		let pieces =
			if white_pieces {
				WHITE_PAWN..=WHITE_KING
			} else {
				BLACK_PAWN..=BLACK_KING
			};

		for piece in pieces {
			let piece_type = get_piece_type(piece);
			let mut bitboard = self.piece_bitboards[piece];

			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard) as usize;

				if (match piece_type {
					PAWN   => self.precalculated_move_data.pawn_attacks[color][piece_index],
					KNIGHT => self.precalculated_move_data.knight_attacks[piece_index],
					BISHOP => self.calculate_bishop_attack_bitboard(piece_index),
					ROOK   => self.calculate_rook_attack_bitboard(piece_index),
					QUEEN  => self.calculate_queen_attack_bitboard(piece_index),
					KING   => self.precalculated_move_data.king_attacks[piece_index],
					_ => 0,
				}) & square_bitboard != 0 {
					return true;
				}
			}
		}

		false
	}
}