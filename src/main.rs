/* TODO
magic bitboards
penalize pieces in front of e2 and d2 pawns?

look into null move pruning and aspiration windows

evaluate pawn structures (isolated, doubled, passed and shield pawns)

insentivise pushing the opponent's king to the edge of the board in endgames,
	and bringing your king closer to the opponent's king if it's trying to checkmate

if you start from a non starting FEN position with en passant on the board, it won't be processed as a legal move
*/


#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod opening_repertoire;
mod resources;
mod precomputed_data;
mod heatmaps;
mod zobrist;
mod piece;
mod utils;
mod board;
mod maxwell;

use crate::maxwell::*;
use crate::piece::*;
use crate::utils::*;
use crate::board::*;
use std::time::{Instant, Duration};
use macroquad::prelude::*;
use crate::resources::Resources;

pub const SQUARE_SIZE: f32 = 64.0;
pub const WINDOW_SIZE: f32 = SQUARE_SIZE * 8.0;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const TESTING_FEN: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const DRAWN_ENDGAME_FEN: &str = "8/8/8/3k4/R5p1/P5r1/4K3/8 w - - 0 1";
pub const MATE_IN_5_FEN: &str = "4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1";
pub const KING_AND_PAWN_ENDGAME_FEN: &str = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1";

#[derive(PartialEq)]
pub enum GameOverState {
	None,

	WhiteWins,
	Draw,
	BlackWins,
}

fn window_conf() -> Conf {
	Conf {
		window_title: "Maxwell ~ The Chess Engine v2.4".to_string(),
		window_width: WINDOW_SIZE as i32,
		window_height: WINDOW_SIZE as i32,
		window_resizable: false,
		..Default::default()
	}
}

#[macroquad::main(window_conf)]
async fn main() {
	let resources = Resources::load().await;

	let mut board_flipped = false;
	let mut game_board = Board::from_fen(STARTING_FEN);
	let mut viewing_board = game_board.clone();

	let mut piece_dragging = None;
	let mut game_over_state = GameOverState::None;

	let mut looking_back = false;


	let mut maxwell = Maxwell::new();



	// for depth in 1..=4 {
	// 	let timer = Instant::now();

	// 	let mut total_captures = 0;
	// 	let mut total_checks = 0;
	// 	println!("Total positions: {}", position_counter_test(&mut game_board, depth, &mut total_captures, &mut total_checks));
	// 	println!("Total captures: {}", total_captures);
	// 	println!("Total checks: {}", total_checks);

	// 	println!("Time in seconds: {}", timer.elapsed().as_secs_f32());

	// 	println!("\n\n\n");
	// }


	// let timer = Instant::now();
	// for _ in 0..10_000_000 {
	// 	game_board.evaluate();
	// }
	// println!("10 million evaluations: {} seconds", timer.elapsed().as_secs_f32());


	// This is suuuuuper slow, and a big bottleneck is the "calculate_attacked_squares_bitboards" function
	// let timer = Instant::now();
	// for _ in 0..1_000_000 {
	// 	let legal_moves = game_board.get_legal_moves_for_color(true, false);
	// }
	// println!("1 million legal move generations: {} seconds", timer.elapsed().as_secs_f32());


	// let timer = Instant::now();
	// for _ in 0..1_000_000 {
	// 	let legal_moves = game_board.get_legal_moves_for_color(true, true);
	// }
	// println!("1 million legal capture generations: {} seconds", timer.elapsed().as_secs_f32());


	// let legal_moves = game_board.get_legal_moves_for_color(true, false);
	// let timer = Instant::now();
	// for _ in 0..1_000_000 {
	// 	let sorted_moves = maxwell.sort_moves(&mut game_board, legal_moves.clone(), None);
	// }
	// println!("1 million move sorts: {} seconds", timer.elapsed().as_secs_f32());


	// let timer = Instant::now();
	// for i in 0..1_000_000 {
	// 	game_board.make_move(legal_moves[i % legal_moves.len()]);
	// 	game_board.undo_last_move();
	// }
	// println!("1 million moves: {} seconds", timer.elapsed().as_secs_f32());


	loop {
		// if is_key_pressed(KeyCode::Space) {
		// 	println!("{}", viewing_board.current_zobrist_key);
		// }

		let mut made_move = false;

		if game_over_state == GameOverState::None {
			if (game_board.whites_turn
			&& MAXWELL_PLAYING == MaxwellPlaying::White)
			|| (!game_board.whites_turn
			&& MAXWELL_PLAYING == MaxwellPlaying::Black)
			|| MAXWELL_PLAYING == MaxwellPlaying::Both {
				maxwell.start(&mut game_board);
				game_board.make_move(maxwell.best_move);
				made_move = true;
			} else if !looking_back {
				if is_mouse_button_pressed(MouseButton::Left) {
					let mouse_index = get_mouse_position_as_index(board_flipped);
					if game_board.board[mouse_index] != 0
					&& is_white(game_board.board[mouse_index]) == game_board.whites_turn {
						piece_dragging = Some(mouse_index);
					}
				}

				if is_mouse_button_released(MouseButton::Left) {
					if let Some(from) = piece_dragging {
						let to = get_mouse_position_as_index(board_flipped);
						if from != to {
							let promotion =
								if get_piece_type(game_board.board[from]) == PAWN
								&& (rank_of_index(to) == 1
								|| rank_of_index(to) == 8) {
									handle_promotion(&resources, &game_board, from, to, board_flipped).await
								} else {
									Some(0)
								};

							if let Some(promotion) = promotion {
								game_board.play_move(promotion, from, to);
								made_move = true;
							}
						}
						piece_dragging = None;
					}
				}
			}

			if made_move {
				viewing_board = game_board.clone();
				looking_back = false;

				if game_board.current_fifty_move_draw == 100
				|| !game_board.checkmating_material_on_board()
				|| game_board.is_threefold_repetition() {
					game_over_state = GameOverState::Draw;
				} else if game_board.get_legal_moves_for_color(game_board.whites_turn, false).is_empty() {
					if game_board.king_in_check(game_board.whites_turn) {
						if game_board.whites_turn {
							game_over_state = GameOverState::BlackWins;
						} else {
							game_over_state = GameOverState::WhiteWins;
						}
					} else {
						game_over_state = GameOverState::Draw;
					}
				}
			}
		} else if is_key_pressed(KeyCode::Enter) {
			game_board = Board::from_fen(STARTING_FEN);
			viewing_board = game_board.clone();
			looking_back = false;
			maxwell = Maxwell::new();

			game_over_state = GameOverState::None;
		}



		if looking_back
		&& is_key_pressed(KeyCode::Right) {
			viewing_board.make_move(game_board.moves[viewing_board.moves.len()]);
			if game_board.moves == viewing_board.moves {
				looking_back = false;
			}
		}

		if is_key_pressed(KeyCode::Left)
		&& !viewing_board.moves.is_empty() {
			viewing_board.undo_last_move();
			looking_back = true;
		}

		if is_key_pressed(KeyCode::F) {
			board_flipped = !board_flipped;
		}



		clear_background(macroquad::prelude::BLACK);

		render_board(&resources, &viewing_board, looking_back, piece_dragging, board_flipped);

		if let Some(piece_dragging) = piece_dragging {
			for legal_move in viewing_board.get_legal_moves_for_piece(piece_dragging, false) {
				let mut to = get_move_to(legal_move);

				if board_flipped {
					to = 63 - to;
				}

				draw_circle(
					((to % 8) as f32 + 0.5) * SQUARE_SIZE,
					((to / 8) as f32 + 0.5) * SQUARE_SIZE,
					SQUARE_SIZE * 0.2,
					resources.transparent_color,
				);
			}

			let pos = mouse_position_vec2() - vec2(SQUARE_SIZE, SQUARE_SIZE) * 0.5;
			draw_texture_ex(
				&resources.pieces_tex,
				pos.x,
				pos.y,
				macroquad::prelude::WHITE,
				DrawTextureParams {
					source: Some(Rect {
						x: (get_image_index_for_piece(viewing_board.board[piece_dragging]) - 1) as f32 * SQUARE_SIZE,
						y: 0.0,
						w: SQUARE_SIZE,
						h: SQUARE_SIZE,
					}),
					..Default::default()
				},
			);
		}

		// for i in 0..64 {
		// 	let x = (i % 8) as f32 * SQUARE_SIZE;
		// 	let y = (i / 8) as f32 * SQUARE_SIZE;

		// 	draw_rectangle(
		// 		x,
		// 		y,
		// 		SQUARE_SIZE, SQUARE_SIZE,
		// 		Color {
		// 			r: 0.0,
		// 			g: 1.0,
		// 			b: 0.0,
		// 			a: 0.5,
		// 		},
		// 	);

		// 	draw_text(
		// 		&format!("{}", KING_MIDDLEGAME_HEATMAP[flip_index(i)]),
		// 		x + 8.0,
		// 		y + 48.0,
		// 		32.0,
		// 		macroquad::prelude::WHITE,
		// 	);
		// }

		if !looking_back {
			match game_over_state {
				GameOverState::None => {}

				GameOverState::WhiteWins => {
					draw_text_ex(
						"White Wins!",
						32.0,
						240.0,
						TextParams {
							font_size: 96,
							color: GOLD,
							..Default::default()
						},
					);
				}

				GameOverState::Draw => {
					draw_text_ex(
						"Draw",
						32.0,
						240.0,
						TextParams {
							font_size: 96,
							color: GOLD,
							..Default::default()
						},
					);
				}

				GameOverState::BlackWins => {
					draw_text_ex(
						"Black Wins!",
						32.0,
						240.0,
						TextParams {
							font_size: 96,
							color: GOLD,
							..Default::default()
						},
					);
				}
			}

			if game_over_state != GameOverState::None {
				draw_text_ex(
					"Press Enter to reset...",
					32.0,
					320.0,
					TextParams {
						font_size: 48,
						color: GOLD,
						..Default::default()
					},
				);
			}
		}

		next_frame().await
	}
}

fn render_board(resources: &Resources, board: &Board, looking_back: bool, piece_dragging: Option<usize>, board_flipped: bool) {
	draw_texture(
		&resources.board_tex,
		0.0, 0.0,
		macroquad::prelude::WHITE,
	);

	if looking_back {
		draw_rectangle(
			0.0, 0.0,
			WINDOW_SIZE, WINDOW_SIZE,
			Color {
				r: 0.9,
				g: 0.9,
				b: 0.92,
				a: 0.2,
			},
		);
	}

	for y in 0..8 {
		for x in 0..8 {
			let mut index = x + y * 8;

			if board_flipped {
				index = 63 - index;
			}

			let piece = get_image_index_for_piece(board.board[index]);

			// if (board.attacked_squares_bitboards[1] >> index) & 1 == 1 {
			// 	draw_rectangle(
			// 		x as f32 * SQUARE_SIZE,
			// 		y as f32 * SQUARE_SIZE,
			// 		SQUARE_SIZE,
			// 		SQUARE_SIZE,
			// 		resources.checkmated_color,
			// 	);
			// }

			if board.board[index] == (board.whites_turn as u8) << 3 | KING
			&& board.king_in_check(board.whites_turn) {
				draw_rectangle(
					x as f32 * SQUARE_SIZE,
					y as f32 * SQUARE_SIZE,
					SQUARE_SIZE, SQUARE_SIZE,
					resources.checkmated_color,
				);
			}

			let last_move = board.get_last_move();
			if last_move != 0
			&& (get_move_from(last_move) == index
			|| get_move_to(last_move) == index) {
				draw_rectangle(
					x as f32 * SQUARE_SIZE,
					y as f32 * SQUARE_SIZE,
					SQUARE_SIZE,
					SQUARE_SIZE,
					resources.last_move_color,
				);
			}

			if piece_dragging == Some(index) {
				draw_rectangle(
					x as f32 * SQUARE_SIZE,
					y as f32 * SQUARE_SIZE,
					SQUARE_SIZE,
					SQUARE_SIZE,
					resources.last_move_color,
				);
			} else if piece > 0 {
				draw_texture_ex(
					&resources.pieces_tex,
					x as f32 * SQUARE_SIZE,
					y as f32 * SQUARE_SIZE,
					macroquad::prelude::WHITE,
					DrawTextureParams {
						source: Some(Rect {
							x: (piece - 1) as f32 * SQUARE_SIZE,
							y: 0.0,
							w: SQUARE_SIZE,
							h: SQUARE_SIZE,
						}),
						..Default::default()
					},
				);
			}
		}
	}
}




async fn handle_promotion(resources: &Resources, board: &Board, promoting_from: usize, promoting_to: usize, board_flipped: bool) -> Option<u8> {
	let render_index = if board_flipped {
		63 - promoting_to
	} else {
		promoting_to
	};

	let x = (render_index % 8) as f32 * SQUARE_SIZE;
	let mut y = (render_index / 8) as f32 * SQUARE_SIZE;
	let render_on_top = y == 0.0;
	let pawn_is_white = promoting_to / 8 == 0;

	y -= if render_on_top { 0.0 } else { 3.0 * SQUARE_SIZE };

	loop {
		if is_mouse_button_pressed(MouseButton::Left) {
			let mouse = mouse_position_vec2();
			let mouse_rect = Rect {
				x: mouse.x,
				y: mouse.y,
				w: 1.0,
				h: 1.0,
			};

			for i in 1..=4 {
				let j = if render_on_top { 4 - i } else { i - 1 };

				if mouse_rect.overlaps(&Rect {
					x: x,
					y: y + j as f32 * SQUARE_SIZE,
					w: SQUARE_SIZE,
					h: SQUARE_SIZE,
				}) {
					return Some(i + 1);
				}
			}

			return None
		}

		clear_background(macroquad::prelude::BLACK);

		render_board(resources, board, false, Some(promoting_from), board_flipped);

		draw_rectangle(
			x,
			y,
			SQUARE_SIZE,
			SQUARE_SIZE * 4.0,
			Color {
				r: 0.92,
				g: 0.92,
				b: 0.93,
				a: 1.0,
			},
		);

		for i in 1..=4 {
			let j = if render_on_top { 4 - i } else { i - 1 };

			draw_texture_ex(
				&resources.pieces_tex,
				x,
				y + j as f32 * SQUARE_SIZE,
				macroquad::prelude::WHITE,
				DrawTextureParams {
					source: Some(Rect {
						x: get_image_index_for_piece((if pawn_is_white { piece::WHITE } else { piece::BLACK }) | i as u8) as f32 * SQUARE_SIZE,
						y: 0.0,
						w: SQUARE_SIZE,
						h: SQUARE_SIZE,
					}),
					..Default::default()
				}
			);
		}

		next_frame().await
	}
}