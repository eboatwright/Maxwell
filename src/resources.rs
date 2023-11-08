use macroquad::prelude::*;

pub struct Resources {
	pub board_tex: Texture2D,
	pub piece_texs: Vec<Texture2D>,
}

impl Resources {
	pub async fn load() -> Self {
		Self {
			board_tex: load_texture("resources/board.png").await.unwrap(),

			piece_texs: vec![
				load_texture("resources/pieces/white_pawn.png").await.unwrap(),
				load_texture("resources/pieces/white_bishop.png").await.unwrap(),
				load_texture("resources/pieces/white_knight.png").await.unwrap(),
				load_texture("resources/pieces/white_rook.png").await.unwrap(),
				load_texture("resources/pieces/white_queen.png").await.unwrap(),
				load_texture("resources/pieces/white_king.png").await.unwrap(),

				load_texture("resources/pieces/black_pawn.png").await.unwrap(),
				load_texture("resources/pieces/black_bishop.png").await.unwrap(),
				load_texture("resources/pieces/black_knight.png").await.unwrap(),
				load_texture("resources/pieces/black_rook.png").await.unwrap(),
				load_texture("resources/pieces/black_queen.png").await.unwrap(),
				load_texture("resources/pieces/black_king.png").await.unwrap(),
			],
		}
	}
}