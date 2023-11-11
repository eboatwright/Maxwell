/* TODO
detect endgame positions
change the king heatmap during endgames
encourage moving the enemy king to the edges to help with checkmate (during endgames?)
*/


#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod resources;
mod utils;
mod heatmaps;
mod point;
mod piece;
mod game;
mod maxwell;

use std::collections::HashMap;
use std::time::{Instant, Duration};
use macroquad::rand::{gen_range, srand};
use macroquad::prelude::*;

use crate::resources::Resources;
use crate::piece::*;
use crate::point::Point;
use crate::game::*;
use crate::maxwell::*;
use crate::utils::*;

pub const SQUARE_SIZE: f32 = 64.0;
pub const WINDOW_SIZE: f32 = SQUARE_SIZE * 8.0;

pub const STARTING_POSITION: &'static str = "\
♖♘♗♕♔♗♘♖\
♙♙♙♙♙♙♙♙\
________\
________\
________\
________\
♟♟♟♟♟♟♟♟\
♜♞♝♛♚♝♞♜\
";

fn window_conf() -> Conf {
	Conf {
		window_title: "MAXWELL".to_string(),
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
	let transparent_color = Color {
		r: 0.8,
		g: 0.8,
		b: 0.82,
		a: 0.5,
	};
	let checkmated_color = Color {
		r: 0.9,
		g: 0.4,
		b: 0.4,
		a: 0.8,
	};
	let last_move_color = Color {
		r: 0.8,
		g: 0.8,
		b: 0.5,
		a: 0.4,
	};

	let mut game = Game::new(
		STARTING_POSITION
// "\
// ________\
// ____♖__♔\
// _______♙\
// __♙♝_♗__\
// ___♙___♟\
// _♜____♙♕\
// __♟_♛___\
// ______♚_\
// ",
	);

	let mut selected_piece = false;
	let mut current_move = PieceMove::default();

	let mut checkmated_king: Option<usize> = None;
	let mut stalemate = false;

	let mut eval_cache: HashMap<GameData, i32> = HashMap::new();

	loop {
		if checkmated_king.is_none()
		&& !stalemate {
			let mut made_move = false;

			if game.game_data.promoting.is_none() {
				if game.game_data.whites_turn {
					if is_mouse_button_pressed(MouseButton::Left) {
						let mouse_vec2 = (Vec2::from(mouse_position()) / SQUARE_SIZE).floor();
						let mouse_index = (mouse_vec2.x + mouse_vec2.y * 8.0) as usize;

						if is_white(game.game_data.board[mouse_index]) == game.game_data.whites_turn
						&& game.game_data.board[mouse_index] != 0 {
							selected_piece = true;
							current_move.from = mouse_index;
						} else if selected_piece {
							selected_piece = false;
							current_move.to = mouse_index;

							if current_move.from != current_move.to {
								for legal_move in game.get_legal_moves_for_piece(current_move.from) {
									if legal_move == current_move {
										game.make_move(legal_move);
										made_move = true;
										break;
									}
								}
							}
						}
					}
				} else {
					let time = Instant::now();

					let (best_move, _) = search_moves(game.clone(), 4, -i32::MAX, i32::MAX, &mut eval_cache);
					if let Some(m) = best_move {
						game.make_move(m);
						made_move = true;
					} else {
						println!("got none from search function :[");
					}

					println!("{} secs", time.elapsed().as_secs_f32());
				}
			} else {
				if is_key_pressed(KeyCode::N) {
					game.promote(KNIGHT);
				} else if is_key_pressed(KeyCode::B) {
					game.promote(BISHOP);
				} else if is_key_pressed(KeyCode::R) {
					game.promote(ROOK);
				} else if is_key_pressed(KeyCode::Q) {
					game.promote(QUEEN);
				}
			}

			if made_move {
				if game.get_legal_moves_for_color(game.game_data.whites_turn).len() == 0 {
					if game.king_in_check(game.game_data.whites_turn) {
						for i in 0..64 {
							if is_white(game.game_data.board[i]) == game.game_data.whites_turn
							&& game.game_data.board[i] & 0b_0111 == KING {
								checkmated_king = Some(i);
								break;
							}
						}
					} else {
						stalemate = true;
					}
				}
			}
		} else {
			if is_key_pressed(KeyCode::Enter) {
				checkmated_king = None;
				game = Game::new(STARTING_POSITION);
			}
		}

		clear_background(macroquad::prelude::BLACK);

		draw_texture(&resources.board_tex, 0.0, 0.0, macroquad::prelude::WHITE);

		for y in 0..8 {
			for x in 0..8 {
				let index = x + y * 8;

				if selected_piece
				&& index == current_move.from {
					draw_rectangle(
						x as f32 * SQUARE_SIZE,
						y as f32 * SQUARE_SIZE,
						SQUARE_SIZE,
						SQUARE_SIZE,
						transparent_color,
					);
				}

				if index == checkmated_king.unwrap_or(69) {
					draw_rectangle(
						x as f32 * SQUARE_SIZE,
						y as f32 * SQUARE_SIZE,
						SQUARE_SIZE,
						SQUARE_SIZE,
						checkmated_color,
					);
				} else if game.game_data.last_move != PieceMove::default() {
					if index == game.game_data.last_move.from {
						draw_rectangle(
							x as f32 * SQUARE_SIZE,
							y as f32 * SQUARE_SIZE,
							SQUARE_SIZE,
							SQUARE_SIZE,
							last_move_color,
						);
					} else if index == game.game_data.last_move.to {
						draw_rectangle(
							x as f32 * SQUARE_SIZE,
							y as f32 * SQUARE_SIZE,
							SQUARE_SIZE,
							SQUARE_SIZE,
							last_move_color,
						);
					}
				}

				let piece = get_index_for_piece(game.game_data.board[index]);

				if piece > 0 {
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

		if selected_piece {
			for legal_move in game.get_legal_moves_for_piece(current_move.from) {
				let p = Point::from_index(legal_move.to);

				draw_circle(
					p.x as f32 * SQUARE_SIZE + 32.0,
					p.y as f32 * SQUARE_SIZE + 32.0,
					12.0,
					transparent_color,
				);
			}
		}

		if stalemate {
			draw_text_ex(
				"STALEMATE!",
				40.0,
				160.0,
				TextParams {
					font_size: 96,
					color: RED,
					..Default::default()
				},
			);

			draw_text_ex(
				"Draw",
				40.0,
				240.0,
				TextParams {
					font_size: 96,
					color: RED,
					..Default::default()
				},
			);
		}

		next_frame().await
	}
}