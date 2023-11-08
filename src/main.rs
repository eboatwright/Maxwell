mod resources;
mod point;
mod piece;
mod game;

use macroquad::prelude::*;

use crate::resources::Resources;
use crate::piece::*;
use crate::point::Point;
use crate::game::Game;

pub const SQUARE_SIZE: f32 = 64.0;
pub const WINDOW_SIZE: f32 = SQUARE_SIZE * 8.0;

fn window_conf() -> Conf {
	Conf {
		window_title: "CHESS ENGINE".to_string(),
		window_width: WINDOW_SIZE as i32,
		window_height: WINDOW_SIZE as i32,
		window_resizable: false,
		..Default::default()
	}
}

#[macroquad::main(window_conf)]
async fn main() {
	let resources = Resources::load().await;

	// let selected_color: Color = Color::from_hex(0x4B98B7);

	let mut game = Game {
		board: generate_starting_position("♜♞♝♛♚♝♞♜♟♟♟♟♟♟♟♟                                ♙♙♙♙♙♙♙♙♖♘♗♕♔♗♘♖".to_string()),
		// board: generate_starting_position("                          ♙ ♛ ♟                                 ".to_string()),
		whites_turn: true,
	};

	let mut selected_piece = false;
	let mut from = 0usize;
	let mut to: usize;

	loop {
		if is_mouse_button_pressed(MouseButton::Left) {
			let mouse_vec2 = (Vec2::from(mouse_position()) / SQUARE_SIZE).floor();
			let mouse_index = (mouse_vec2.x + (7.0 - mouse_vec2.y) * 8.0) as usize;

			if !selected_piece {
				if game.board[mouse_index].piece_type != PieceType::None
				&& game.board[mouse_index].is_white == game.whites_turn {
					selected_piece = true;
					from = mouse_index;
				}
			} else {
				selected_piece = false;
				to = mouse_index;


				let mut invalid_move = false;

				if from != to
				&& game.board[from].piece_type != PieceType::None
				&& game.board[from].is_white == game.whites_turn {
					let mut captured = false;

					if game.board[to].piece_type != PieceType::None {
						if game.board[from].is_white != game.board[to].is_white {
							captured = true;
						} else {
							invalid_move = true;
						}
					}

					match (game.board[from].piece_type, game.board[from].is_white) {
						(PieceType::Pawn, false) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							let difference = from_p.difference(to_p);

							if captured {
								if !(difference.x == -1
								&& difference.y == -1)
								&& !(difference.x == 1
								&& difference.y == -1) {
									invalid_move = true;
								}
							} else {
								let mut can_move_twice = false;
								if from_p.y == 6
								&& game.board[(from_p.x + (from_p.y - 1) * 8) as usize].piece_type == PieceType::None {
									can_move_twice = true;
								}

								if (difference.y != -1
								&& (!can_move_twice
								|| difference.y != -2))
								|| difference.x != 0 {
									invalid_move = true;
								}
							}
						}

						(PieceType::Pawn, true) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							let difference = from_p.difference(to_p);

							if captured {
								if !(difference.x == -1
								&& difference.y == 1)
								&& !(difference.x == 1
								&& difference.y == 1) {
									invalid_move = true;
								}
							} else {
								let mut can_move_twice = false;
								if from_p.y == 1
								&& game.board[(from_p.x + (from_p.y + 1) * 8) as usize].piece_type == PieceType::None {
									can_move_twice = true;
								}

								if (difference.y != 1
								&& (!can_move_twice
								|| difference.y != 2))
								|| difference.x != 0 {
									invalid_move = true;
								}
							}
						}

						(PieceType::Bishop, ..) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							if from_p.x == to_p.x
							|| from_p.y == to_p.y {
								invalid_move = true;
							} else {
								let x_dir = if to_p.x > from_p.x { 1 } else { -1 };
								let y_dir = if to_p.y > from_p.y { 1 } else { -1 };

								let mut x = from_p.x;
								let mut y = from_p.y;

								let mut found = false;

								while x >= 0 && x < 8
								&& y >= 0 && y < 8 {
									x += x_dir;
									y += y_dir;

									if to_p.x == x
									&& to_p.y == y {
										found = true;
										break;
									}

									if game.board[(x + y * 8) as usize].piece_type != PieceType::None {
										break;
									}
								}

								if !found {
									invalid_move = true;
								}
							}
						}

						(PieceType::Knight, ..) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							let difference = from_p.abs_difference(to_p);

							if !(difference.x == 2
							&& difference.y == 1)
							&& !(difference.x == 1
							&& difference.y == 2) {
								invalid_move = true;
							}
						}

						(PieceType::Rook, ..) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							if (from_p.x != to_p.x
							&& from_p.y != to_p.y)
							|| game.piece_inbetween_points(from_p, to_p) {
								invalid_move = true;
							}
						}

						(PieceType::Queen, ..) => {}

						(PieceType::King, ..) => {
							let from_p = Point::from_index(from);
							let to_p = Point::from_index(to);

							let difference = from_p.abs_difference(to_p);

							if difference.x > 1
							|| difference.y > 1 {
								invalid_move = true;
							}
						}
						_ => {}
					}
				} else {
					invalid_move = true;
				}

				if !invalid_move {
					game.board[to] = game.board[from];
					game.board[from] = Piece::none();

					game.whites_turn = !game.whites_turn;
				}
			}
		}

		clear_background(BLACK);

		draw_texture(&resources.board_tex, 0.0, 0.0, WHITE);

		for y in 0..8 {
			for x in 0..8 {
				let index = x + (7 - y) * 8;

				if selected_piece
				&& index == from {
					draw_rectangle(
						x as f32 * SQUARE_SIZE,
						y as f32 * SQUARE_SIZE,
						SQUARE_SIZE,
						SQUARE_SIZE,
						Color {
							r: 0.8,
							g: 0.8,
							b: 0.82,
							a: 0.4,
						},
					);
				}

				let piece = get_index_for_piece(game.board[index]);

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