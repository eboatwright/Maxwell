#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod resources;
mod precomputed_data;
mod heatmaps;
mod piece;
mod utils;
mod board;

use crate::piece::*;
use crate::utils::*;
use crate::board::Board;
use std::time::{Instant, Duration};
use macroquad::rand::{gen_range, srand};
use macroquad::prelude::*;

use crate::resources::Resources;

pub const SQUARE_SIZE: f32 = 64.0;
pub const WINDOW_SIZE: f32 = SQUARE_SIZE * 8.0;

pub const STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn window_conf() -> Conf {
	Conf {
		window_title: "Maxwell ~ The Chess Engine v2.0".to_string(),
		window_width: WINDOW_SIZE as i32,
		window_height: WINDOW_SIZE as i32,
		window_resizable: false,
		..Default::default()
	}
}

#[macroquad::main(window_conf)]
async fn main() {
	srand(miniquad::date::now() as u64);

	let resources = Resources::load().await;

	let mut game_board = Board::from_fen(STARTING_FEN);
	let mut viewing_board = Board::from_fen(STARTING_FEN);

	let mut piece_dragging = None;

	loop {
		let looking_back = game_board.moves != viewing_board.moves;
		if !looking_back {
			if is_mouse_button_pressed(MouseButton::Left) {
				let mouse_index = get_mouse_position_as_index();
				if game_board.board[mouse_index] != 0
				&& is_white(game_board.board[mouse_index]) == game_board.whites_turn {
					piece_dragging = Some(mouse_index);
				}
			}

			if is_mouse_button_released(MouseButton::Left) {
				if let Some(from) = piece_dragging {
					let to = get_mouse_position_as_index();
					if from != to {
						let promotion =
							if get_piece_type(game_board.board[from]) == PAWN
							&& (rank_of_index(to) == 1
							|| rank_of_index(to) == 8) {
								handle_promotion(&resources, &game_board, from, to).await
							} else {
								Some(0)
							};

						if let Some(promotion) = promotion {
							game_board.play_move(promotion, from, to);
						}
					}
					piece_dragging = None;
				}
			}

			viewing_board = game_board.clone();
		} else if is_key_pressed(KeyCode::Right) {
			viewing_board.make_move(game_board.moves[viewing_board.moves.len()]);
		}

		if is_key_pressed(KeyCode::Left) {
			viewing_board.undo_last_move();
		}

		clear_background(macroquad::prelude::BLACK);

		render_board(&resources, &viewing_board, looking_back, piece_dragging);

		if let Some(piece_dragging) = piece_dragging {
			for legal_move in viewing_board.get_legal_moves_for_piece(piece_dragging) {
				let to = get_move_to(legal_move);
				draw_circle(
					((to % 8) as f32 + 0.5) * SQUARE_SIZE,
					((to as f32 / 8.0).floor() + 0.5) * SQUARE_SIZE,
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

		// let bitboard = viewing_board.piece_bitboards[1][0];
		// for i in 0..64 {
		// 	if (1 << i) & bitboard != 0 {
		// 		draw_rectangle(
		// 			(i % 8) as f32 * SQUARE_SIZE,
		// 			(i as f32 / 8.0).floor() * SQUARE_SIZE,
		// 			SQUARE_SIZE, SQUARE_SIZE,
		// 			Color {
		// 				r: 1.0,
		// 				g: 0.0,
		// 				b: 0.0,
		// 				a: 0.5,
		// 			},
		// 		);
		// 	}
		// }

		// let bitboard = viewing_board.piece_bitboards[1][1];
		// for i in 0..64 {
		// 	if (1 << i) & bitboard != 0 {
		// 		draw_rectangle(
		// 			(i % 8) as f32 * SQUARE_SIZE,
		// 			(i as f32 / 8.0).floor() * SQUARE_SIZE,
		// 			SQUARE_SIZE, SQUARE_SIZE,
		// 			Color {
		// 				r: 0.0,
		// 				g: 0.0,
		// 				b: 1.0,
		// 				a: 0.5,
		// 			},
		// 		);
		// 	}
		// }

		// draw_text_ex(
		// 	"STALEMATE!",
		// 	40.0,
		// 	160.0,
		// 	TextParams {
		// 		font_size: 96,
		// 		color: RED,
		// 		..Default::default()
		// 	},
		// );

		// draw_text_ex(
		// 	"Draw",
		// 	40.0,
		// 	240.0,
		// 	TextParams {
		// 		font_size: 96,
		// 		color: RED,
		// 		..Default::default()
		// 	},
		// );

		next_frame().await
	}
}

fn render_board(resources: &Resources, board: &Board, looking_back: bool, piece_dragging: Option<usize>) {
	draw_texture(&resources.board_tex, 0.0, 0.0, macroquad::prelude::WHITE);
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
			let index = x + y * 8;
			let piece = get_image_index_for_piece(board.board[index]);

			// if (board.all_piece_bitboards[1] >> index) & 1 == 1 {
			// 	draw_rectangle(
			// 		x as f32 * SQUARE_SIZE,
			// 		y as f32 * SQUARE_SIZE,
			// 		SQUARE_SIZE,
			// 		SQUARE_SIZE,
			// 		resources.checkmated_color,
			// 	);
			// }

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

async fn handle_promotion(resources: &Resources, board: &Board, promoting_from: usize, promoting_to: usize) -> Option<u8> {
	let x = (promoting_to % 8) as f32 * SQUARE_SIZE;
	let y = (promoting_to as f32 / 8.0).floor();

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
				if mouse_rect.overlaps(&Rect {
					x: x,
					y: y + (4 - i) as f32 * SQUARE_SIZE,
					w: SQUARE_SIZE,
					h: SQUARE_SIZE,
				}) {
					return Some(i + 1);
				}
			}

			return None
		}

		clear_background(macroquad::prelude::BLACK);

		render_board(resources, board, false, Some(promoting_from));

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
			draw_texture_ex(
				&resources.pieces_tex,
				x,
				(y + (4 - i) as f32) * SQUARE_SIZE,
				macroquad::prelude::WHITE,
				DrawTextureParams {
					source: Some(Rect {
						x: i as f32 * SQUARE_SIZE,
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