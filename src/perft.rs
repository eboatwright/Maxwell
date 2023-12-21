use crate::NO_PIECE;
use crate::move_data::*;
use crate::Board;
use std::time::Instant;
use crate::pieces::PROMOTABLE;

#[derive(Default, Debug)]
pub struct PerftResults {
	pub depth: usize,

	pub positions: u128,
	pub captures: u128,
	pub en_passants: u128,
	pub castles: u128,
	pub promotions: u128,
	pub checks: u128,
	// pub checkmates: u128,
}

impl PerftResults {
	pub fn new(depth: usize) -> Self {
		Self {
			depth,
			..Default::default()
		}
	}

	pub fn calculate(board: &mut Board, depth: usize) {
		let mut results = PerftResults::new(depth);
		let timer = Instant::now();

		let depth = results.depth;
		perft(board, &mut results, depth);

		println!("{} seconds", timer.elapsed().as_secs_f32());
		println!("{:#?}", results);
	}
}

fn perft(board: &mut Board, results: &mut PerftResults, depth_left: usize) {
	if depth_left == 0 {
		results.positions += 1;
		return;
	}

	for data in board.get_legal_moves_for_color(board.white_to_move, false) {
		if data.capture != NO_PIECE as u8 {
			results.captures += 1;

			if data.flag == EN_PASSANT_FLAG {
				results.en_passants += 1;
			}
		} else if data.flag == SHORT_CASTLE_FLAG
		|| data.flag == LONG_CASTLE_FLAG {
			results.castles += 1;
		}

		if PROMOTABLE.contains(&data.flag) {
			results.promotions += 1;
		}

		board.make_move(data);

		if board.king_in_check(board.white_to_move) {
			results.checks += 1;
		}

		perft(board, results, depth_left - 1);
		board.undo_last_move();
	}
}