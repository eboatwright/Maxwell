use macroquad::prelude::*;

pub struct Resources {
	pub board_tex: Texture2D,
	pub pieces_tex: Texture2D,

	pub transparent_color: Color,
	pub checkmated_color: Color,
	pub last_move_color: Color,
}

impl Resources {
	pub async fn load() -> Self {
		Self {
			board_tex: load_texture("resources/board.png").await.unwrap(),
			pieces_tex: load_texture("resources/pieces.png").await.unwrap(),

			transparent_color: Color {
				r: 0.92,
				g: 0.94,
				b: 0.96,
				a: 0.5,
			},

			checkmated_color: Color {
				r: 0.8,
				g: 0.16,
				b: 0.16,
				a: 0.5,
			},

			last_move_color: Color {
				r: 0.82,
				g: 0.70,
				b: 0.24,
				a: 0.5,
			},
		}
	}
}