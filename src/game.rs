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

	pub fn get_legal_moves_for_color(&self, white: bool) -> Vec<PieceMove> {
		let mut result = vec![];

		for i in 0..64 {
			if self.board[i].piece_type != PieceType::None
			&& self.board[i].is_white == white {
				result = [result, self.get_legal_moves_for_piece(i)].concat();
			}
		}

		result
	}

	pub fn get_legal_moves_for_piece(&self, index: usize) -> Vec<PieceMove> {
		let mut result = vec![];

		match self.board[index].piece_type {
			PieceType::Pawn => {
				let p = Point::from_index(index);

				if self.board[index].is_white {
					// Moving forward
					if self.board[index + 8].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index + 8,
						});

						if index + 16 < 63
						&& p.y == 1
						&& self.board[index + 16].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: index + 16,
							});
						}
					}


					// Capturing
					for dir in [Point::new(-1, 1), Point::new(1, 1)] {
						let new_p = p + dir;
						let new_index = (new_p.x + new_p.y * 8) as usize;

						if self.board[new_index].piece_type != PieceType::None
						&& self.board[new_index].is_white != self.board[index].is_white {
							result.push(PieceMove {
								from: index,
								to: new_index,
							});
						}
					}
				} else {
					// Moving forward
					if self.board[index - 8].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index - 8,
						});

						if index - 16 < 63
						&& p.y == 6
						&& self.board[index - 16].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: index - 16,
							});
						}
					}


					// Capturing
					for dir in [Point::new(-1, -1), Point::new(1, -1)] {
						let new_p = p + dir;
						let new_index = (new_p.x + new_p.y * 8) as usize;

						if self.board[new_index].piece_type != PieceType::None
						&& self.board[new_index].is_white != self.board[index].is_white {
							result.push(PieceMove {
								from: index,
								to: new_index,
							});
						}
					}
				}
			}

			PieceType::Bishop => {
				for dir in [
					Point::new(-1, -1),
					Point::new(-1,  1),
					Point::new( 1, -1),
					Point::new( 1,  1),
				] {
					let mut p = Point::from_index(index);

					while p.x + dir.x >= 0
					&& p.x + dir.x < 8
					&& p.y + dir.y >= 0
					&& p.y + dir.y < 8 {
						p += dir;

						let new_index = (p.x + p.y * 8) as usize;

						if new_index == index {
							continue;
						}

						if self.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,
							});
						} else {
							if self.board[new_index].is_white != self.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,
								});
							}

							break;
						}
					}
				}
			}

			PieceType::Knight => {
				let p = Point::from_index(index);

				for dir in [
					Point::new(-1,  2),
					Point::new( 1,  2),
					Point::new(-1, -2),
					Point::new( 1, -2),
					Point::new( 2, -1),
					Point::new( 2,  1),
					Point::new(-2, -1),
					Point::new(-2,  1),
				] {
					let new_point = p + dir;

					if new_point.x < 0
					|| new_point.x > 7
					|| new_point.y < 0
					|| new_point.y > 7 {
						continue;
					}

					let new_index = (new_point.x + new_point.y * 8) as usize;

					if self.board[new_index].piece_type == PieceType::None
					|| self.board[new_index].is_white != self.board[index].is_white {
						result.push(PieceMove {
							from: index,
							to: new_index,
						});
					}
				}
			}

			PieceType::Rook => {
				for dir in [
					Point::new(-1,  0),
					Point::new( 1,  0),
					Point::new( 0, -1),
					Point::new( 0,  1),
				] {
					let mut p = Point::from_index(index);

					while p.x + dir.x >= 0
					&& p.x + dir.x < 8
					&& p.y + dir.y >= 0
					&& p.y + dir.y < 8 {
						p += dir;

						let new_index = (p.x + p.y * 8) as usize;

						if new_index == index {
							continue;
						}

						if self.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,
							});
						} else {
							if self.board[new_index].is_white != self.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,
								});
							}

							break;
						}
					}
				}
			}

			PieceType::Queen => {
				for dir in [
					Point::new(-1, -1),
					Point::new( 0, -1),
					Point::new( 1, -1),

					Point::new(-1,  0),
					Point::new( 1,  0),

					Point::new(-1,  1),
					Point::new( 0,  1),
					Point::new( 1,  1),
				] {
					let mut p = Point::from_index(index);

					while p.x + dir.x >= 0
					&& p.x + dir.x < 8
					&& p.y + dir.y >= 0
					&& p.y + dir.y < 8 {
						p += dir;

						let new_index = (p.x + p.y * 8) as usize;

						if new_index == index {
							continue;
						}

						if self.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,
							});
						} else {
							if self.board[new_index].is_white != self.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,
								});
							}

							break;
						}
					}
				}
			}

			PieceType::King => {
				let p = Point::from_index(index);

				for dir in [
					Point::new(-1, -1),
					Point::new( 0, -1),
					Point::new( 1, -1),

					Point::new(-1,  0),
					Point::new( 1,  0),

					Point::new(-1,  1),
					Point::new( 0,  1),
					Point::new( 1,  1),
				] {
					let new_point = p + dir;

					if new_point.x < 0
					|| new_point.x > 7
					|| new_point.y < 0
					|| new_point.y > 7 {
						continue;
					}

					let new_index = (new_point.x + new_point.y * 8) as usize;

					if self.board[new_index].piece_type == PieceType::None
					|| self.board[new_index].is_white != self.board[index].is_white {
						result.push(PieceMove {
							from: index,
							to: new_index,
						});
					}
				}
			}

			_ => {}
		}

		result
	}
}