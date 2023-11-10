#![allow(dead_code)]

mod resources;
mod point;
mod piece;
mod game;

use macroquad::rand::{gen_range, srand};
use macroquad::prelude::*;

use crate::resources::Resources;
use crate::piece::*;
use crate::point::Point;
use crate::game::Game;

pub const SQUARE_SIZE: f32 = 64.0;
pub const WINDOW_SIZE: f32 = SQUARE_SIZE * 8.0;

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

	let mut game = Game::new(
		"♖♘♗♕♔♗♘♖♙♙♙♙♙♙♙♙                                ♟♟♟♟♟♟♟♟♜♞♝♛♚♝♞♜".to_string(),
		// "   ♖♔♖                                                  ♜   ♚  ♜".to_string(),
	);

	let mut selected_piece = false;
	let mut current_move = PieceMove {
		from: 0,
		to: 0,

		..Default::default()
	};

	let mut checkmated_king: Option<usize> = None;
	let mut stalemate = false;

	loop {
		if checkmated_king.is_none()
		&& !stalemate {
			let mut made_move = false;

			if game.game_data.promoting.is_none() {
				if game.game_data.whites_turn {
					if is_mouse_button_pressed(MouseButton::Left) {
						let mouse_vec2 = (Vec2::from(mouse_position()) / SQUARE_SIZE).floor();
						let mouse_index = (mouse_vec2.x + mouse_vec2.y * 8.0) as usize;

						if game.game_data.board[mouse_index].is_white == game.game_data.whites_turn
						&& game.game_data.board[mouse_index].piece_type != PieceType::None {
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
					let legal_moves = game.get_legal_moves_for_color(false);

					if legal_moves.len() > 0 {
						game.make_move(legal_moves[gen_range(0, legal_moves.len())]);
					}
				}
			} else {
				if is_key_pressed(KeyCode::B) {
					game.promote(PieceType::Bishop);
				} else if is_key_pressed(KeyCode::N) {
					game.promote(PieceType::Knight);
				} else if is_key_pressed(KeyCode::R) {
					game.promote(PieceType::Rook);
				} else if is_key_pressed(KeyCode::Q) {
					game.promote(PieceType::Queen);
				}
			}

			if made_move
			&& game.get_legal_moves_for_color(game.game_data.whites_turn).len() == 0 {
				if game.king_in_check(game.game_data.whites_turn) {
					for i in 0..64 {
						if game.game_data.board[i].is_white == game.game_data.whites_turn
						&& game.game_data.board[i].piece_type == PieceType::King {
							checkmated_king = Some(i);
							break;
						}
					}
				} else {
					stalemate = true;
				}
			}
		} else {
			if is_key_pressed(KeyCode::Enter) {
				checkmated_king = None;
				game = Game::new(
					"♖♘♗♕♔♗♘♖♙♙♙♙♙♙♙♙                                ♟♟♟♟♟♟♟♟♜♞♝♛♚♝♞♜".to_string(),
				);
			}
		}

		clear_background(BLACK);

		draw_texture(&resources.board_tex, 0.0, 0.0, WHITE);

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
				}

				let piece = get_index_for_piece(game.game_data.board[index]);

				if piece > 0 {
					draw_texture(
						&resources.piece_texs[piece - 1],
						x as f32 * SQUARE_SIZE,
						y as f32 * SQUARE_SIZE,
						WHITE,
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

pub fn generate_starting_position(string: String) -> [Piece; 64] {
	let mut board: [Piece; 64] = [Piece::new(PieceType::None, false); 64];

	for i in 0..64 {
		board[i] = match string.chars().collect::<Vec<char>>()[i] {
			'♟' => Piece::new(PieceType::Pawn, true),
			'♝' => Piece::new(PieceType::Bishop, true),
			'♞' => Piece::new(PieceType::Knight, true),
			'♜' => Piece::new(PieceType::Rook, true),
			'♛' => Piece::new(PieceType::Queen, true),
			'♚' => Piece::new(PieceType::King, true),

			'♙' => Piece::new(PieceType::Pawn, false),
			'♗' => Piece::new(PieceType::Bishop, false),
			'♘' => Piece::new(PieceType::Knight, false),
			'♖' => Piece::new(PieceType::Rook, false),
			'♕' => Piece::new(PieceType::Queen, false),
			'♔' => Piece::new(PieceType::King, false),

			_ => Piece::new(PieceType::None, false),
		};
	}

	board
}

fn get_index_for_piece(piece: Piece) -> usize {
	match (piece.piece_type, piece.is_white) {
		(PieceType::Pawn, true) => 1,
		(PieceType::Bishop, true) => 2,
		(PieceType::Knight, true) => 3,
		(PieceType::Rook, true) => 4,
		(PieceType::Queen, true) => 5,
		(PieceType::King, true) => 6,

		(PieceType::Pawn, false) => 7,
		(PieceType::Bishop, false) => 8,
		(PieceType::Knight, false) => 9,
		(PieceType::Rook, false) => 10,
		(PieceType::Queen, false) => 11,
		(PieceType::King, false) => 12,

		(PieceType::None, ..) => 0,
	}
}

// fn get_char_for_piece(piece: Piece) -> char {
// 	match (piece.piece_type, piece.is_white) {
// 		(PieceType::Pawn, true) => '♟',
// 		(PieceType::Bishop, true) => '♝',
// 		(PieceType::Knight, true) => '♞',
// 		(PieceType::Rook, true) => '♜',
// 		(PieceType::Queen, true) => '♛',
// 		(PieceType::King, true) => '♚',

// 		(PieceType::Pawn, false) => '♙',
// 		(PieceType::Bishop, false) => '♗',
// 		(PieceType::Knight, false) => '♘',
// 		(PieceType::Rook, false) => '♖',
// 		(PieceType::Queen, false) => '♕',
// 		(PieceType::King, false) => '♔',

// 		(PieceType::None, ..) => ' ',
// 	}
// }

// fn get_index_from_coordinate(coordinate: String) -> Option<usize> {
// 	let mut invalid_column = false;

// 	let mut result = match coordinate.chars().nth(0).expect("Invalid coordinate") {
// 		'a' => 0,
// 		'b' => 1,
// 		'c' => 2,
// 		'd' => 3,
// 		'e' => 4,
// 		'f' => 5,
// 		'g' => 6,
// 		'h' => 7,
// 		_ => {
// 			invalid_column = true;
// 			0
// 		},
// 	};

// 	let number = coordinate.chars().nth(1)?.to_string().parse::<usize>().ok()? as usize;
// 	result += (number - 1) * 8;

// 	if invalid_column
// 	|| !(1..=8).contains(&number) {
// 		None
// 	} else {
// 		Some(result)
// 	}
// }