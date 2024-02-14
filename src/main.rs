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
mod pv_table;
mod move_sorter;
mod scored_move_list;

use crate::utils::move_str_is_valid;
use crate::castling_rights::print_castling_rights;
use crate::bot::{Bot, BotConfig, MAX_SEARCH_EXTENSIONS};
use crate::perft::*;
use crate::move_data::MoveData;
use crate::pieces::*;
use crate::log::Log;
use crate::board::Board;
use std::io;
use colored::Colorize;
use std::time::Instant;

pub const STARTING_FEN:         &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE_FEN:         &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const TEST_POSITION_4:      &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const DRAWN_ENDGAME_FEN:    &str = "8/8/8/3k4/R5p1/P5r1/4K3/8 w - - 0 1";
pub const MATE_IN_5_FEN:        &str = "4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1";
pub const PAWN_ENDGAME_FEN:     &str = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1";
pub const ONE_PAWN_ENDGAME_FEN: &str = "8/8/1k6/8/8/1K6/1P6/8 w - - 0 1";
pub const ENDGAME_POSITION:     &str = "8/pk4p1/2prp3/3p1p2/3P2p1/R2BP3/2P2KPP/8 w - - 8 35";
pub const PAWN_EVAL_TESTING:    &str = "4k3/p1pp4/8/4pp1P/2P4P/8/P5P1/4K3 w - - 0 1";

fn main() {
	let mut bot_config = BotConfig::from_args(std::env::args().collect::<Vec<String>>());

	// let mut log = Log::none();

	let mut board = Board::from_fen(&bot_config.fen);
	let mut bot = Bot::new(bot_config.clone());

	let mut command = String::new();
	let mut moves = String::new();

	loop {
		command.clear();

		io::stdin()
			.read_line(&mut command)
			.expect("Failed to read terminal input");

		// log.write(format!("Got command: {}\n", command));

		// The length of this Vec will always be > 0
		let command_split = command.trim()
			.split(' ')
			.collect::<Vec<&str>>();

		match command_split[0] {
			// UCI protocol

			"uci" => {
				println!("id name Maxwell v3.1-1");
				println!("id author eboatwright");
				println!("option name Hash type spin default 256 min 0 max 4000");

				println!("uciok");
			}

			"setoption" => {
				// 0         1    2             3     4
				// setoption name <Option name> value <Value>

				if let Some(option_name) = command_split.get(2) {
					if let Some(value) = command_split.get(4) {
						match *option_name {
							"Hash" => {
								bot_config.hash_size = value.parse::<usize>().unwrap_or(256);
							}

							_ => {}
						}
					}
				}
			}

			"isready" => println!("readyok"),

			"ucinewgame" => {
				// log = Log::new();
				board = Board::from_fen(STARTING_FEN);
				bot = Bot::new(bot_config.clone());
			}

			// TODO: add support for "position fen"
			// Format: position startpos (moves e2e4 e7e5 ...)
			"position" => {
				// Reset the board to the initial position
				for _ in 0..board.moves.len() {
					board.undo_last_move();
				}

				// "moves" is for the opening book
				moves.clear();
				for coordinates in command_split.iter().skip(3) {
					moves += &format!("{} ", coordinates);

					if !move_str_is_valid(coordinates) {
						println!("Illegal move: {}", coordinates);
						break;
					}

					let data = MoveData::from_coordinates(coordinates.to_string());
					if !board.play_move(data) {
						break;
						// log.write(err);
					}
				}
				moves.pop();
			}

			"go" => {
				let mut my_time = 0.0;
				let mut depth_to_search = 255 - MAX_SEARCH_EXTENSIONS;

				// Anything below 3 words is treated is treated as "go infinite"
				if command_split.len() > 2 {
					let go_type = command_split[1];

					// This is pretty dang ugly, but it works, and is safe :D
					match go_type {
						// go depth X
						"depth" => {
							if let Some(_depth_to_search) = command_split.get(2) {
								if let Ok(_depth_to_search) = _depth_to_search.parse::<u8>() {
									depth_to_search = _depth_to_search;
								}
							}
						}

						// go movetime X
						"movetime" => {
							if let Some(time_in_millis) = command_split.get(2) {
								// These are capped to one millisecond, because 0.0 time is treated as "go infinite"
								my_time = f32::max(1.0, time_in_millis.parse::<f32>().unwrap_or(0.0)) / 1000.0;
							}
						}

						// go wtime X btime Y
						"wtime" => {
							let time_index = if board.white_to_move { 2 } else { 4 };
							if let Some(time_in_millis) = command_split.get(time_index) {
								my_time = f32::max(1.0, time_in_millis.parse::<f32>().unwrap_or(0.0)) / 1000.0;
							}
						}

						_ => {}
					}
				}

				bot.start(&mut board, moves.clone(), my_time, depth_to_search);

				println!("bestmove {}", bot.best_move.to_coordinates());
				// log.write(format!("bestmove {}", bot.best_move.to_coordinates()));
			}

			"stop" => bot.search_cancelled = true, // Now that I think about it, this doesn't actually do anything LMAO
			"quit" => break,

			// My debug tools

			// "play" => play(command_split[1] == "white"),

			"move" => {
				if let Some(move_coordinates) = command_split.get(1) {
					if move_str_is_valid(move_coordinates) {
						let data = MoveData::from_coordinates(move_coordinates.to_string());
						if board.play_move(data) {
							board.print();
						}
					} else {
						println!("Invalid move");
					}
				}
			}

			"undo" => {
				if board.undo_last_move() {
					board.print();
				}
			}

			// "clearlogs" => {
			// 	if let Ok(logs_folder) = std::fs::read_dir("./logs/") {
			// 		for log_file in logs_folder {
			// 			std::fs::remove_file(log_file.unwrap().path()).expect("Failed to clear logs");
			// 		}
			// 	} else {
			// 		println!("No logs folder");
			// 	}
			// }

			"print" => board.print(),
			"bitboards" => board.print_bitboards(),
			"castlingrights" => print_castling_rights(board.board_state.current.castling_rights),
			"zobrist" => println!("{}", board.zobrist.key.current),
			"eval" => println!("{}", board.evaluate() * board.perspective()),
			"fiftymoves" => println!("{}", board.board_state.current.fifty_move_counter),

			"ttsize" => bot.transposition_table.print_size(),
			"cleartt" => {
				bot.transposition_table.table.clear();
				println!("Cleared transposition table");
			}

			"perft" => {
				if let Some(depth) = command_split.get(1) {
					let depth = depth.parse::<u8>().unwrap_or(0);
					PerftResults::calculate(&mut board, depth);
				}
			}

			// "test" => {
			// 	let mut old_best_time  =  f32::MAX;
			// 	let mut old_worst_time = -f32::MAX;

			// 	let mut new_best_time  =  f32::MAX;
			// 	let mut new_worst_time = -f32::MAX;

			// 	let piece = BLACK_ROOK as u8;
			// 	let capture = WHITE_KNIGHT as u8;

			// 	for _ in 0..5 {
			// 		let timer = Instant::now();
			// 		for _ in 0..1_000_000_000 {
			// 			let score = MVV_LVA[get_piece_type(piece as usize) * 6 + get_piece_type(capture as usize)];
			// 		}
			// 		let old_time = timer.elapsed().as_secs_f32();

			// 		let timer = Instant::now();
			// 		for _ in 0..1_000_000_000 {
			// 			let score = (5 - get_piece_type(piece as usize)) + (get_piece_type(capture as usize) + 1) * 10;
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

			// _ => log.write(format!("Unknown command: {}", command)),
			_ => {}
		}
	}
}