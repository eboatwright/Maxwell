use crate::piece_square_tables::get_base_worth_of_piece;
use crate::killer_moves::KillerMoves;
use crate::move_data::{MoveData, NULL_MOVE, SHORT_CASTLE_FLAG, LONG_CASTLE_FLAG};
use crate::pieces::*;
use crate::piece_square_tables::get_full_worth_of_piece;
use crate::Board;

pub const MAX_KILLER_MOVE_PLY: usize = 32;

pub const MVV_LVA: [i32; 36] = [
	15, 25, 35, 45, 55, 65, // Pawn
	14, 24, 34, 44, 54, 64, // Knight
	13, 23, 33, 43, 53, 63, // Bishop
	12, 22, 32, 42, 52, 62, // Rook
	11, 21, 31, 41, 51, 61, // Queen
	10, 20, 30, 40, 50, 60, // King
];

pub struct MoveSorter {
	pub killer_moves: [KillerMoves; MAX_KILLER_MOVE_PLY],
	pub history: [[i32; 64]; PIECE_COUNT],
}

impl MoveSorter {
	pub fn new() -> Self {
		Self {
			killer_moves: [KillerMoves::new(); MAX_KILLER_MOVE_PLY],
			history: [[0; 64]; PIECE_COUNT],
		}
	}

	pub fn clear(&mut self) {
		self.killer_moves = [KillerMoves::new(); MAX_KILLER_MOVE_PLY];
		self.history = [[0; 64]; PIECE_COUNT];
	}

	pub fn push_killer_move(&mut self, data: MoveData, depth: u8) {
		if depth < MAX_KILLER_MOVE_PLY as u8 {
			self.killer_moves[depth as usize].push(data);
		}
	}

	pub fn sort_moves(&mut self, board: &mut Board, moves: Vec<MoveData>, hash_move: MoveData, depth: u8) -> Vec<MoveData> {
		if moves.is_empty() {
			return vec![];
		}

		let num_of_moves = moves.len();
		let mut scores = vec![(0, 0); num_of_moves];

		// board.calculate_attacked_squares();
		board.calculate_attacked_squares_for_color((!board.white_to_move) as usize);

		// let squares_i_attack = board.attacked_squares_bitboards[board.white_to_move as usize];
		let squares_opponent_attacks = board.attacked_squares_bitboards[!board.white_to_move as usize];

		for i in 0..num_of_moves {
			let m = moves[i];

			let mut score = 0;

			if m == hash_move {
				score = i32::MAX;
			} else {
				if m.capture != NO_PIECE as u8 {
					// score += (5 * get_full_worth_of_piece(m.capture as usize, m.to as usize, endgame) - get_full_worth_of_piece(m.piece as usize, m.from as usize, endgame)) + 8000;
					score += MVV_LVA[get_piece_type(m.piece as usize) * 6 + get_piece_type(m.capture as usize)] + 8000;
				} else {
					if depth < MAX_KILLER_MOVE_PLY as u8
					&& self.killer_moves[depth as usize].is_killer(m) {
						score += 5000;
					}

					score += self.history[m.piece as usize][m.to as usize];
				}

				if m.flag == SHORT_CASTLE_FLAG
				|| m.flag == LONG_CASTLE_FLAG {
					score += 2000;
				}

				// if squares_i_attack & (1 << m.to) != 0 {
				// 	score += get_full_worth_of_piece(m.piece as usize, m.to as usize, endgame);
				// }

				if squares_opponent_attacks & (1 << m.to) != 0 {
					score -= 2 * get_base_worth_of_piece(m.piece as usize);
				}

				if PROMOTABLE.contains(&m.flag) {
					score += get_base_worth_of_piece(build_piece(is_piece_white(m.piece as usize), m.flag as usize)) + 12000;
				}
			}

			scores[i] = (score, i);
		}

		scores.sort_by(|a, b| b.0.cmp(&a.0));

		let mut ordered = vec![NULL_MOVE; num_of_moves];
		for i in 0..num_of_moves {
			ordered[i] = moves[scores[i].1];
		}

		ordered
	}
}