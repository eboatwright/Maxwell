use crate::utils::index_from_coordinate;
use crate::piece::*;

pub struct Board {
	pub board: [u8; 64],
	pub whites_turn: bool,

	pub white_short_castle_rights: bool,
	pub white_long_castle_rights: bool,
	pub black_short_castle_rights: bool,
	pub black_long_castle_rights: bool,

	pub en_passant_capture: Option<usize>,
	pub moves_without_capture_or_pawn_push: u16,
	pub fullmove_counter: u16,
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



		Board {
			board,
			whites_turn,

			white_short_castle_rights,
			white_long_castle_rights,
			black_short_castle_rights,
			black_long_castle_rights,

			en_passant_capture,
			moves_without_capture_or_pawn_push,
			fullmove_counter,
		}
	}
}