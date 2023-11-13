use crate::PrecomputedBitboards;
use crate::utils::index_from_coordinate;
use crate::piece::*;

pub struct Board {
	pub precomputed_bitboards: PrecomputedBitboards,
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
			precomputed_bitboards: PrecomputedBitboards::initialize(),
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

		final_board.compute_all_piece_bitboards();

		final_board
	}

	pub fn compute_all_piece_bitboards(&mut self) {
		self.all_piece_bitboards = [0; 2];

		for i in 0..64 {
			if self.board[i] != 0 {
				let color = is_white(self.board[i]) as usize;
				self.all_piece_bitboards[color] |= 1 << i;
			}
		}
	}

	pub fn get_last_move(&self) -> u32 {
		if self.moves.len() == 0 {
			return 0;
		}
		self.moves[self.moves.len() - 1]
	}

	pub fn get_legal_moves_for_piece(&self, piece_index: usize) -> Vec<u32> {
		let piece_color = is_white(self.board[piece_index]) as usize;
		let piece_type = get_piece_type(self.board[piece_index]);

		let bitboard = match piece_type {
			KNIGHT => self.precomputed_bitboards.knight_bitboards[piece_index] & !self.all_piece_bitboards[piece_color],

			_ => 0,
		};

		let mut result = vec![];

		for i in 0..64 {
			if (bitboard >> i) & 1 == 1 {
				result.push(build_move(0, self.board[i] as u32, piece_index, i));
			}
		}

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

				self.moves.push(piece_move);

				self.compute_all_piece_bitboards();

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

		self.compute_all_piece_bitboards();

		self.whites_turn = !self.whites_turn;
	}
}