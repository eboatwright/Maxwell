/* TODO
calculate my own magic numbers; currently "borrowing" Sebastian Lague's ^^
check out pin detection to speed up check detection?
try to write a neural network to evaluate positions? :o
figure out how to implement "pondering" to think on opponent's time

Ideas I've tried, but they didn't help, or made it play worse (Or I implemented them wrong :P)
https://www.chessprogramming.org/Futility_Pruning
https://www.chessprogramming.org/Principal_Variation_Search
https://www.chessprogramming.org/Internal_Iterative_Deepening

Random ideas to try (from other engines and chessprogramming.org)
History reduction
https://www.chessprogramming.org/History_Leaf_Pruning
https://www.chessprogramming.org/Futility_Pruning#MoveCountBasedPruning
https://www.chessprogramming.org/Delta_Pruning
https://www.chessprogramming.org/Triangular_PV-Table
https://www.chessprogramming.org/Razoring (look into a better implementation)

Some random resources I found:
https://analog-hors.github.io/site/magic-bitboards/ (didn't use this for my initial implementation, but that might change ;))
https://web.archive.org/web/20071030220825/http://www.brucemo.com/compchess/programming/pvs.htm
https://github.com/lynx-chess/Lynx/
https://github.com/Heiaha/Weiawaga/
*/

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

mod utils;
mod log;
mod value_holder;
mod pieces;
mod castling_rights;
mod piece_square_tables;
mod precalculated_move_data;
mod magic_numbers;
mod move_data;
mod transposition_table;
mod killer_moves;
mod opening_book;
mod board;
mod zobrist;
mod perft;
mod bot;
mod move_sorter;

use crate::castling_rights::print_castling_rights;
use crate::bot::Bot;
use crate::perft::*;
use crate::move_data::MoveData;
use crate::pieces::*;
use crate::log::Log;
use crate::board::Board;
use std::io;
use colored::Colorize;
use std::time::Instant;

pub const STARTING_FEN:      &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE_FEN:      &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const TEST_POSITION_4:   &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const DRAWN_ENDGAME_FEN: &str = "8/8/8/3k4/R5p1/P5r1/4K3/8 w - - 0 1";
pub const MATE_IN_5_FEN:     &str = "4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1";
pub const PAWN_ENDGAME_FEN:  &str = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1";
pub const ENDGAME_POSITION:  &str = "8/pk4p1/2prp3/3p1p2/3P2p1/R2BP3/2P2KPP/8 w - - 8 35";
pub const PAWN_EVAL_TESTING: &str = "4k3/p1pp4/8/4pp1P/2P4P/8/P5P1/4K3 w - - 0 1";

fn main() {
	let mut log = Log::none();

	let mut board = Board::from_fen(STARTING_FEN);
	let mut bot = Bot::new(true);

	let mut command = String::new();
	let mut moves = String::new();

	loop {
		command.clear();

		io::stdin()
			.read_line(&mut command)
			.expect("Failed to read command");

		// log.write(format!("Got command: {}\n", command));

		let command_split = command.trim()
			.split(' ')
			.collect::<Vec<&str>>();

		match command_split[0] {
			// UCI protocol

			"uci" => println!("uciok"),
			"isready" => println!("readyok"),

			"ucinewgame" => {
				// log = Log::new();
				board = Board::from_fen(STARTING_FEN);
				bot = Bot::new(true);
			}

			// Format: position startpos (moves e2e4 e7e5 ...)
			"position" => {
				for _ in 0..board.moves.len() {
					board.undo_last_move();
				}

				moves.clear();
				for coordinates in command_split.iter().skip(3) {
					moves += &format!("{} ", coordinates);
					let data = MoveData::from_coordinates(coordinates.to_string());
					if !board.play_move(data) {
						let err = format!("{}: failed to play move: {}", "FATAL ERROR".white().on_red(), coordinates);
						println!("{}", err);
						log.write(err);
					}
				}
				moves.pop();
			}

			// Format: go (movetime, wtime) X (btime Y)
			"go" => {
				let my_time_label = if board.white_to_move { "wtime" } else { "btime" };
				let mut my_time = 0.0;

				for i in [1, 3] {
					if command_split[i] == my_time_label
					|| command_split[i] == "movetime" {
						if let Ok(time_in_millis) = command_split[i + 1].parse::<i32>() {
							my_time = time_in_millis as f32 / 1000.0;
							break;
						}
					}
				}

				bot.start(&mut board, moves.clone(), my_time);

				println!("bestmove {}", bot.best_move.to_coordinates());
				// log.write(format!("bestmove {}", bot.best_move.to_coordinates()));
			}

			"stop" => bot.search_cancelled = true,
			"quit" => break,

			// My debug tools :]

			// "play" => play(command_split[1] == "white"),

			"move" => {
				let data = MoveData::from_coordinates(command_split[1].to_string());
				if board.play_move(data) {
					board.print();
				}
			}

			"undo" => {
				if board.undo_last_move() {
					board.print();
				}
			}

			"clearlogs" => {
				if let Ok(logs_folder) = std::fs::read_dir("./logs/") {
					for log_file in logs_folder {
						std::fs::remove_file(log_file.unwrap().path()).expect("Failed to clear logs");
					}
				} else {
					println!("No logs folder");
				}
			}

			"print" => board.print(),
			"bitboards" => board.print_bitboards(),
			"castlingrights" => print_castling_rights(board.castling_rights.current),
			"zobrist" => println!("{}", board.zobrist.key),
			"eval" => println!("{}", board.evaluate() * board.perspective()),

			"ttsize" => bot.transposition_table.print_size(),
			"cleartt" => {
				bot.transposition_table.table.clear();
				println!("Cleared transposition table");
			}

			"perft" => {
				let depth = command_split[1].parse::<usize>().expect("Invalid depth");
				PerftResults::calculate(&mut board, depth);
			}

			// "test" => {
			// 	let mut board = Board::from_fen(STARTING_FEN);

			// 	let mut old_best_time  =  f32::MAX;
			// 	let mut old_worst_time = -f32::MAX;

			// 	let mut new_best_time  =  f32::MAX;
			// 	let mut new_worst_time = -f32::MAX;

			// 	for _ in 0..5 {
			// 		board.clear_castling_rights();
			// 		board.zobrist.clear();

			// 		let timer = Instant::now();
			// 		for _ in 0..100_000 {
			// 			let legal_moves = board.get_legal_moves_for_color(board.white_to_move, false);
			// 			let _ = sort_moves(&board, legal_moves, None);
			// 		}
			// 		let old_time = timer.elapsed().as_secs_f32();


			// 		board.clear_castling_rights();
			// 		board.zobrist.clear();

			// 		let timer = Instant::now();
			// 		for _ in 0..100_000 {
			// 			let legal_moves = board.get_legal_moves_for_color(board.white_to_move, false);
			// 			let _ = new_sort_moves(&board, legal_moves, None);
			// 		}
			// 		let new_time = timer.elapsed().as_secs_f32();


			// 		if old_time < old_best_time {
			// 			old_best_time = old_time;
			// 		}

			// 		if old_time > old_worst_time {
			// 			old_worst_time = old_time;
			// 		}

			// 		if new_time < new_best_time {
			// 			new_best_time = new_time;
			// 		}

			// 		if new_time > new_worst_time {
			// 			new_worst_time = new_time;
			// 		}
			// 	}

			// 	println!("Old: worst: {}, best: {}", old_worst_time, old_best_time);
			// 	println!("New: worst: {}, best: {}", new_worst_time, new_best_time);
			// }

			_ => log.write(format!("Unknown command: {}", command)),
		}
	}
}