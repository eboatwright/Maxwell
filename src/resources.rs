use macroquad::prelude::*;

pub struct Resources {
	pub board_tex: Texture2D,
	pub pieces_tex: Texture2D,
}

impl Resources {
	pub async fn load() -> Self {
		Self {
			board_tex: load_texture("resources/board.png").await.unwrap(),
			pieces_tex: load_texture("resources/pieces.png").await.unwrap(),
		}
	}
}