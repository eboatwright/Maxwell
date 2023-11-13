#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod resources;
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

	let mut board = Board::from_fen(STARTING_FEN);

	let mut piece_dragging = None;
	let mut last_move = 0u16;

	loop {
		if is_mouse_button_pressed(MouseButton::Left) {
			let mouse_index = get_mouse_position_as_index();
			if board.board[mouse_index] != 0 {
				piece_dragging = Some(mouse_index);
			}
		}

		if is_mouse_button_released(MouseButton::Left) {
			if let Some(from) = piece_dragging {
				let to = get_mouse_position_as_index();
				if from != to {
					board.board[to] = board.board[from];
					board.board[from] = 0;

					last_move = ((from as u16) << 6) | to as u16;
				}
				piece_dragging = None;
			}
		}

		clear_background(macroquad::prelude::BLACK);

		draw_texture(&resources.board_tex, 0.0, 0.0, macroquad::prelude::WHITE);

		for y in 0..8 {
			for x in 0..8 {
				let index = x + y * 8;
				let piece = get_image_index_for_piece(board.board[index]);

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

		if let Some(piece_dragging) = piece_dragging {
			let pos = mouse_position_vec2() - vec2(SQUARE_SIZE, SQUARE_SIZE) * 0.5;
			draw_texture_ex(
				&resources.pieces_tex,
				pos.x,
				pos.y,
				macroquad::prelude::WHITE,
				DrawTextureParams {
					source: Some(Rect {
						x: (get_image_index_for_piece(board.board[piece_dragging]) - 1) as f32 * SQUARE_SIZE,
						y: 0.0,
						w: SQUARE_SIZE,
						h: SQUARE_SIZE,
					}),
					..Default::default()
				},
			);
		}

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