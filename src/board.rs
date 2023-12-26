use crate::value_holder::ValueHolder;
use crate::utils::{pop_lsb, print_bitboard, coordinate_to_index};
use crate::piece_square_tables::{get_base_worth_of_piece, get_full_worth_of_piece, ROOK_WORTH, BISHOP_WORTH};
use crate::precalculated_move_data::*;
use crate::move_data::*;
use crate::zobrist::Zobrist;
use crate::pieces::*;
use crate::castling_rights::*;
use colored::Colorize;

pub const BITBOARD_COUNT: usize = PIECE_COUNT;
pub const MAX_ENDGAME_MATERIAL: f32 = (ROOK_WORTH * 2 + BISHOP_WORTH * 2) as f32;

// TODO: tweak these
pub const DOUBLED_PAWN_PENALTY: i32 = 35;
pub const ISOLATED_PAWN_PENALTY: i32 = 20;
pub const PASSED_PAWN_BOOST: [i32; 8] = [0, 15, 15, 30, 50, 90, 150, 0];

pub struct Board {
	pub precalculated_move_data: PrecalculatedMoveData,

	pub piece_bitboards: [u64; BITBOARD_COUNT],
	pub color_bitboards: [u64; 2],
	pub attacked_squares_bitboards: [u64; 2],

	pub castling_rights: ValueHolder<u8>,
	pub fifty_move_draw: ValueHolder<u8>,

	pub en_passant_file: usize,
	pub white_to_move: bool,

	pub total_material_without_pawns: i32,

	pub zobrist: Zobrist,

	pub moves: Vec<MoveData>,

	attacked_squares_calculated: [bool; 2],
}

impl Board {
	// Pieces, side to move, castling rights, en passant square, fifty move draw, fullmove counter
	pub fn from_fen(fen: &str) -> Self {
		let fen = fen.split(' ').collect::<Vec<&str>>();

		let mut castling_rights = 0b0000;
		if fen[2].contains('Q') { castling_rights ^= WHITE_CASTLE_LONG; }
		if fen[2].contains('K') { castling_rights ^= WHITE_CASTLE_SHORT; }
		if fen[2].contains('q') { castling_rights ^= BLACK_CASTLE_LONG; }
		if fen[2].contains('k') { castling_rights ^= BLACK_CASTLE_SHORT; }

		let fifty_move_draw = fen[4].parse::<u8>().unwrap_or(0);

		let mut board = Self {
			precalculated_move_data: PrecalculatedMoveData::calculate(),

			piece_bitboards: [0; BITBOARD_COUNT],
			color_bitboards: [0; 2],
			attacked_squares_bitboards: [0; 2],

			castling_rights: ValueHolder::new(castling_rights),
			fifty_move_draw: ValueHolder::new(fifty_move_draw),

			en_passant_file: 0, // This isn't implemented at all
			// en_passant_file: if fen[3] == "-" { 0 } else { (coordinate_to_index(fen[3]) % 8) + 1 },
			white_to_move: fen[1] == "w",

			total_material_without_pawns: 0,

			zobrist: Zobrist::default(),

			moves: vec![],

			attacked_squares_calculated: [false; 2],
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

					let piece_type = get_piece_type(piece);

					if piece_type != PAWN
					&& piece_type != KING {
						let piece_worth = get_base_worth_of_piece(piece);
						board.total_material_without_pawns += piece_worth;
					}
				}
			}
		}

		// This has to be done after the board is setup (Duh)
		let mut zobrist = Zobrist::generate();
		zobrist.generate_initial_key(&mut board);
		board.zobrist = zobrist;

		board.calculate_attacked_squares();

		board
	}

	pub fn calculate_attacked_squares(&mut self) {
		self.calculate_attacked_squares_for_color(0);
		self.calculate_attacked_squares_for_color(1);
	}

	// This is SLOOOOOOOOOOOOOWWWWWWW :[
	pub fn calculate_attacked_squares_for_color(&mut self, color: usize) {
		if self.attacked_squares_calculated[color] {
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

		self.attacked_squares_bitboards[color] = attacked_squares;
		self.attacked_squares_calculated[color] = true;
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

	pub fn print_bitboards(&self) {
		for piece in 0..BITBOARD_COUNT {
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
		print_bitboard("Black attacked squares", "1".bold().italic().white().on_black(), self.attacked_squares_bitboards[0]);
		print_bitboard("White attacked squares", "1".bold().italic().normal().on_white(), self.attacked_squares_bitboards[1]);
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
			let legal_moves = self.get_legal_moves_for_piece(data.from, false);
			for m in legal_moves {
				if (!promoting || data.flag == m.flag)
				&& m.from == data.from
				&& m.to == data.to {
					self.make_move(m);
					return true;
				}
			}
		}

		println!("Illegal move");
		false
	}

	pub fn make_move(&mut self, data: MoveData) {
		let piece_color = is_piece_white(data.piece as usize) as usize;
		let other_color = !is_piece_white(data.piece as usize) as usize;

		// if data.piece >= NO_PIECE as u8 {
		// 	println!("Illegal piece! Move: {:#?}", data);
		// 	return;
		// }

		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if !PROMOTABLE.contains(&data.flag) {
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		} else {
			self.piece_bitboards[build_piece(piece_color == 1, data.flag as usize)] ^= 1 << data.to;
			self.total_material_without_pawns += get_base_worth_of_piece(data.flag as usize);
		}

		self.color_bitboards[piece_color] ^= 1 << data.from;
		self.color_bitboards[piece_color] ^= 1 << data.to;

		if data.capture != NO_PIECE as u8 {
			if get_piece_type(data.capture as usize) != PAWN {
				self.total_material_without_pawns -= get_base_worth_of_piece(data.capture as usize);
			}

			if data.flag == EN_PASSANT_FLAG {
				let pawn_to_en_passant = if is_piece_white(data.piece as usize) {
					data.to + 8
				} else {
					data.to - 8
				};

				self.piece_bitboards[data.capture as usize] ^= 1 << pawn_to_en_passant;
				self.color_bitboards[other_color] ^= 1 << pawn_to_en_passant;
			} else {
				self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
				self.color_bitboards[other_color] ^= 1 << data.to;
			}
		}

		// I dunno if there's a better way to do this :/
		if data.piece == WHITE_KING as u8 {
			self.castling_rights.current &= !ALL_WHITE_CASTLING_RIGHTS;

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
			self.castling_rights.current &= !ALL_BLACK_CASTLING_RIGHTS;

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
		} else if data.from == 0
		|| data.to == 0 {
			self.castling_rights.current &= !BLACK_CASTLE_LONG;
		} else if data.from == 7
		|| data.to == 7 {
			self.castling_rights.current &= !BLACK_CASTLE_SHORT;
		} else if data.from == 56
		|| data.to == 56 {
			self.castling_rights.current &= !WHITE_CASTLE_LONG;
		} else if data.from == 63
		|| data.to == 63 {
			self.castling_rights.current &= !WHITE_CASTLE_SHORT;
		}


		if data.capture == NO_PIECE as u8
		&& get_piece_type(data.piece as usize) != PAWN {
			self.fifty_move_draw.current += 1;
		} else {
			self.fifty_move_draw.current = 0;
		}

		self.fifty_move_draw.push();
		self.castling_rights.push();

		self.zobrist.make_move(
			data,
			self.get_last_move(),
			self.castling_rights.current,
			self.castling_rights.history[self.castling_rights.index - 1],
		);

		self.attacked_squares_calculated = [false; 2];

		self.moves.push(data);
		self.white_to_move = !self.white_to_move;
	}

	pub fn undo_last_move(&mut self) -> bool {
		if self.moves.is_empty() {
			return false;
		}
		let last_move = self.moves.pop().unwrap();

		let piece_color = is_piece_white(last_move.piece as usize) as usize;
		let other_color = !is_piece_white(last_move.piece as usize) as usize;

		self.piece_bitboards[last_move.piece as usize] ^= 1 << last_move.from;

		if !PROMOTABLE.contains(&last_move.flag) {
			self.piece_bitboards[last_move.piece as usize] ^= 1 << last_move.to;
		} else {
			self.piece_bitboards[build_piece(piece_color == 1, last_move.flag as usize)] ^= 1 << last_move.to;
			self.total_material_without_pawns -= get_base_worth_of_piece(last_move.flag as usize);
		}

		self.color_bitboards[piece_color] ^= 1 << last_move.from;
		self.color_bitboards[piece_color] ^= 1 << last_move.to;

		if last_move.capture != NO_PIECE as u8 {
			if get_piece_type(last_move.capture as usize) != PAWN {
				self.total_material_without_pawns += get_base_worth_of_piece(last_move.capture as usize);
			}

			if last_move.flag == EN_PASSANT_FLAG {
				let pawn_to_en_passant = if is_piece_white(last_move.piece as usize) {
					last_move.to + 8
				} else {
					last_move.to - 8
				};

				self.piece_bitboards[last_move.capture as usize] ^= 1 << pawn_to_en_passant;

				self.color_bitboards[other_color] ^= 1 << pawn_to_en_passant;
			} else {
				self.piece_bitboards[last_move.capture as usize] ^= 1 << last_move.to;
				self.color_bitboards[other_color] ^= 1 << last_move.to;
			}
		} else {
			if last_move.flag == SHORT_CASTLE_FLAG {
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
		}

		self.fifty_move_draw.pop();
		self.castling_rights.pop();
		self.zobrist.pop();

		self.attacked_squares_calculated = [false; 2];

		self.white_to_move = !self.white_to_move;

		true
	}

	pub fn king_in_check(&mut self, king_is_white: bool) -> bool {
		self.calculate_attacked_squares_for_color((!king_is_white) as usize);
		self.piece_bitboards[if king_is_white { WHITE_KING } else { BLACK_KING }] & self.attacked_squares_bitboards[(!king_is_white) as usize] != 0
	}

	pub fn get_legal_moves_for_color(&mut self, white_pieces: bool, only_captures: bool) -> Vec<MoveData> {
		let mut result = vec![];

		let pieces = if white_pieces {
			WHITE_PAWN..=WHITE_KING
		} else {
			BLACK_PAWN..=BLACK_KING
		};

		for piece in pieces {
			let mut bitboard = self.piece_bitboards[piece];

			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard);
				result.extend(self.get_legal_moves_for_piece(piece_index, only_captures));
			}
		}

		result
	}

	pub fn get_legal_moves_for_piece(&mut self, piece_index: u8, only_captures: bool) -> Vec<MoveData> {
		let mut result = vec![];

		let piece = self.get_piece(piece_index);
		// if piece == NO_PIECE {
		// 	println!("NO_PIECE found! piece_index: {}", piece_index);
		// }
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

		for i in (0..result.len()).rev() {
			let data = result[i];

			// if data.piece == NO_PIECE as u8 {
			// 	println!("Illegal move found! {:#?} on piece: {}, and index: {}, captures only: {}", data, piece_type, piece_index, only_captures);
			// }

			self.make_move(data);

			if self.king_in_check(!self.white_to_move) {
				result.remove(i);
			}

			self.undo_last_move();
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
		(1.5 - self.total_material_without_pawns as f32 * (0.9 / MAX_ENDGAME_MATERIAL as f32)).clamp(0.0, 1.0)
		// (1.0 - self.total_material_without_pawns as f32 * (1.0 / MAX_ENDGAME_MATERIAL)).clamp(0.0, 1.0)
	}

	pub fn perspective(&self) -> i32 { if self.white_to_move { 1 } else { -1 } }

	pub fn evaluate(&mut self) -> i32 {
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
							white_pawn_evaluation += PASSED_PAWN_BOOST[8 - piece_index / 8];
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

		self.calculate_attacked_squares();

		let white_attacked_squares = self.attacked_squares_bitboards[1].count_ones() as i32;
		let black_attacked_squares = self.attacked_squares_bitboards[0].count_ones() as i32;

		let white_king_index = pop_lsb(&mut (self.piece_bitboards[WHITE_KING].clone())) as usize;
		let black_king_index = pop_lsb(&mut (self.piece_bitboards[BLACK_KING].clone())) as usize;

		let weak_squares_around_white_king = ((
				  self.precalculated_move_data.king_attacks[white_king_index]
				& self.attacked_squares_bitboards[0]
			).count_ones() as f32 * (1.0 - endgame)) as i32;

		let weak_squares_around_black_king = ((
				  self.precalculated_move_data.king_attacks[black_king_index]
				& self.attacked_squares_bitboards[1]
			).count_ones() as f32 * (1.0 - endgame)) as i32;

		 ((white_material + white_attacked_squares * 10 - weak_squares_around_white_king * 20 + white_pawn_evaluation)
		- (black_material + black_attacked_squares * 10 - weak_squares_around_black_king * 20 + black_pawn_evaluation)) * self.perspective()
	}

	pub fn can_short_castle(&mut self, white: bool) -> bool {
		// self.king_in_check calculates attacked squares
		   !self.king_in_check(white)
		&&  self.castling_rights.current & SHORT_CASTLING_RIGHTS[white as usize] != 0
		&& (self.occupied_bitboard() | self.attacked_squares_bitboards[(!white) as usize]) & SHORT_CASTLE_MASK[white as usize] == 0
	}

	pub fn can_long_castle(&mut self, white: bool) -> bool {
		let occupied = self.occupied_bitboard();
		   !self.king_in_check(white)
		&&  self.castling_rights.current & LONG_CASTLING_RIGHTS[white as usize] != 0
		&&  EXTRA_LONG_CASTLE_SQUARE_CHECK[white as usize] & occupied == 0
		&& (occupied | self.attacked_squares_bitboards[(!white) as usize]) & LONG_CASTLE_MASK[white as usize] == 0
	}

	pub fn insufficient_checkmating_material(&self) -> bool {
		   self.total_material_without_pawns < ROOK_WORTH
		&& self.piece_bitboards[WHITE_PAWN] == 0
		&& self.piece_bitboards[BLACK_PAWN] == 0
	}

	pub fn try_null_move(&mut self) -> bool {
		if self.king_in_check(self.white_to_move)
		|| self.king_in_check(!self.white_to_move) {
			return false;
		}

		self.white_to_move = !self.white_to_move;
		self.zobrist.make_null_move();
		self.moves.push(NULL_MOVE);

		true
	}

	pub fn undo_null_move(&mut self) {
		self.white_to_move = !self.white_to_move;
		self.zobrist.pop();
		self.moves.pop();
	}
}