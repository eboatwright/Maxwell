#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod resources;
mod heatmaps;
mod piece;
mod utils;
mod board;

use crate::utils::get_image_index_for_piece;
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

	let board = Board::from_fen(STARTING_FEN);

	loop {
		clear_background(macroquad::prelude::BLACK);

		draw_texture(&resources.board_tex, 0.0, 0.0, macroquad::prelude::WHITE);

		for y in 0..8 {
			for x in 0..8 {
				let index = x + y * 8;
				let piece = get_image_index_for_piece(board.board[index]);

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