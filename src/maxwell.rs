use std::collections::HashMap;
use std::cmp::{max, min};
use crate::piece::*;
use crate::game::*;

pub fn search_moves(mut game: Game, depth: i8, mut alpha: i32, beta: i32, eval_cache: &mut HashMap<GameData, i32>) -> (Option<PieceMove>, i32) {
	let legal_moves = game.get_legal_moves_for_color(game.game_data.whites_turn);

	if legal_moves.len() == 0 {
		if game.king_in_check(game.game_data.whites_turn) {
			return (None, -9999999 - depth as i32);
		}
		return (None, 0);
	}

	if depth == 0 {
		return (None, -(if let Some(eval) = eval_cache.get(&game.game_data) {
			*eval
		} else {
			let eval = game.eval();
			eval_cache.insert(game.game_data, eval);
			eval
		}));
	}

	let mut best_move = 0;

	for i in 0..legal_moves.len() {
		game.make_move(legal_moves[i]);

		let (_, mut eval_after_move) = search_moves(game.clone(), depth - 1, -beta, -alpha, eval_cache);
		eval_after_move *= -1;

		game.undo_last_move();

		if eval_after_move >= beta {
			return (Some(legal_moves[i]), beta);
		}

		if eval_after_move > alpha {
			best_move = i;
			alpha = eval_after_move;
		}
	}

	(Some(legal_moves[best_move]), alpha)
}