use crate::piece::*;
use crate::Point;

pub struct Game {
	pub board: [Piece; 64],
	pub whites_turn: bool,
}

impl Game {
	// fn flush(&self) {
	// 	std::process::Command::new("clear").status().unwrap();

	// 	for y in (0..8).rev() {
	// 		let mut line = format!("{}", format!("{} ", y + 1).bold().black().on_white());
	// 		for x in 0..8 {
	// 			let piece = self.board[x + y * 8];

	// 			line.push(get_char_for_piece(piece));
	// 			line.push(' ');
	// 		}
	// 		println!("{}", line);
	// 	}
	// 	println!("{}", "  a b c d e f g h".bold().black().on_white());

	// 	stdout().flush().unwrap();
	// }

	pub fn piece_inbetween_points(&mut self, mut a: Point, mut b: Point) -> bool {
		if a.x > b.x {
			let buffer = a.x;
			a.x = b.x;
			b.x = buffer;
		}

		if a.y > b.y {
			let buffer = a.y;
			a.y = b.y;
			b.y = buffer;
		}

		for x in a.x..=b.x {
			for y in a.y..=b.y {
				if (x == a.x
				&& y == a.y)
				|| (x == b.x
				&& y == b.y) {
					continue;
				}


				if self.board[(x + y * 8) as usize].piece_type != PieceType::None {
					return true;
				}
			}
		}
		false
	}
}