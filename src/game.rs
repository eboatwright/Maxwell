use crate::generate_starting_position;
use crate::piece::*;
use crate::Point;

#[derive(Copy, Clone, Debug)]
pub struct GameData {
	pub board: [Piece; 64],
	pub whites_turn: bool,

	pub promoting: Option<usize>,

	pub last_move: PieceMove,

	pub white_can_short_castle: bool,
	pub white_can_long_castle: bool,

	pub black_can_short_castle: bool,
	pub black_can_long_castle: bool,
}

pub struct Game {
	pub game_data: GameData,
	pub last_game_data: GameData,
}

impl Game {
	pub fn new(board: String) -> Self {
		let game_data = GameData {
			board: generate_starting_position(board),
			whites_turn: true,

			promoting: None,

			last_move: PieceMove::default(),

			white_can_short_castle: true,
			white_can_long_castle: true,

			black_can_short_castle: true,
			black_can_long_castle: true,
		};

		Self {
			game_data,
			last_game_data: game_data,
		}
	}

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


				if self.game_data.board[(x + y * 8) as usize].piece_type != PieceType::None {
					return true;
				}
			}
		}
		false
	}

	pub fn make_move(&mut self, m: PieceMove) {
		self.last_game_data = self.game_data.clone();
		self.game_data.last_move = m;

		self.game_data.board[m.to] = self.game_data.board[m.from];
		self.game_data.board[m.from] = Piece::none();

		let p = Point::from_index(m.to);

		if m.from == 0
		|| m.to == 0 {
			self.game_data.black_can_long_castle = false;
		} else if m.from == 7
		|| m.to == 7 {
			self.game_data.black_can_short_castle = false;
		} else if m.from == 56
		|| m.to == 56 {
			self.game_data.white_can_long_castle = false;
		} else if m.from == 63
		|| m.to == 63 {
			self.game_data.white_can_short_castle = false;
		}

		match self.game_data.board[m.to].piece_type {
			PieceType::Pawn => {
				if let Some(i) = m.en_passant_capture {
					self.game_data.board[i].piece_type = PieceType::None;
				} else if p.y == 0
				|| p.y == 7 {
					// if self.game_data.whites_turn {
						self.game_data.promoting = Some(m.to);
					// } else {
						// self.game_data.board[m.to].piece_type = m.promotion_type;
					// }
				}
			}

			PieceType::King => {
				if m.short_castle {
					self.game_data.board[m.to - 1] = self.game_data.board[m.to + 1];
					self.game_data.board[m.to + 1].piece_type = PieceType::None;
				} else if m.long_castle {
					self.game_data.board[m.to + 1] = self.game_data.board[m.to - 2];
					self.game_data.board[m.to - 2].piece_type = PieceType::None;
				}

				if self.game_data.board[m.to].is_white {
					self.game_data.white_can_short_castle = false;
					self.game_data.white_can_long_castle = false;
				} else {
					self.game_data.black_can_short_castle = false;
					self.game_data.black_can_long_castle = false;
				}
			}

			_ => {}
		}

		self.game_data.whites_turn = !self.game_data.whites_turn;
	}

	pub fn undo_last_move(&mut self) {
		self.game_data = self.last_game_data;
	}

	pub fn promote(&mut self, piece: PieceType) {
		self.game_data.board[self.game_data.promoting.unwrap()].piece_type = piece;
		self.game_data.promoting = None;
	}

	pub fn get_legal_moves_for_color(&mut self, white: bool) -> Vec<PieceMove> {
		let mut result = vec![];

		for i in 0..64 {
			if self.game_data.board[i].piece_type != PieceType::None
			&& self.game_data.board[i].is_white == white {
				result = [result, self.get_legal_moves_for_piece(i)].concat();
			}
		}

		result
	}

	pub fn king_in_check(&mut self) -> bool {
		for i in 0..64 {
			if self.game_data.board[i].piece_type != PieceType::None
			&& self.game_data.board[i].is_white == self.game_data.whites_turn {
				for legal_move in self.get_moves_for_piece(i) {
					if self.game_data.board[legal_move.to].piece_type == PieceType::King
					&& self.game_data.board[legal_move.to].is_white != self.game_data.board[legal_move.from].is_white {
						return true;
					}
				}
			}
		}

		false
	}

	pub fn get_legal_moves_for_piece(&mut self, index: usize) -> Vec<PieceMove> {
		let mut result = self.get_moves_for_piece(index);

		for i in (0..result.len()).rev() {
			let mut already_removed = false;

			if result[i].short_castle
			|| result[i].long_castle {
				self.make_move(
					PieceMove {
						to: (result[i].from as i8 + if result[i].short_castle { 1 } else { -1 }) as usize,
						short_castle: false,
						long_castle: false,
						..result[i]
					}
				);

				if self.king_in_check() {
					result.remove(i);
					already_removed = true;
				}

				self.undo_last_move();
			}

			if already_removed {
				continue;
			}

			self.make_move(result[i]);

			if self.king_in_check() {
				result.remove(i);
			}

			self.undo_last_move();
		}

		result
	}

	fn get_moves_for_piece(&mut self, index: usize) -> Vec<PieceMove> {
		let mut result = vec![];

		match self.game_data.board[index].piece_type {
			PieceType::Pawn => {
				let p = Point::from_index(index);

				if self.game_data.board[index].is_white {







					if p.y > 0 {
						let will_promote = p.y - 1 == 0;


						// Moving forward
						if self.game_data.board[index - 8].piece_type == PieceType::None {
							if will_promote {
								for t in PROMOTABLE_PIECES {
									result.push(PieceMove {
										from: index,
										to: index - 8,

										promotion_type: t,

										..Default::default()
									});
								}
							} else {
								result.push(PieceMove {
									from: index,
									to: index - 8,

									..Default::default()
								});
							}

							if index - 16 < 63
							&& p.y == 6
							&& self.game_data.board[index - 16].piece_type == PieceType::None {
								result.push(PieceMove {
									from: index,
									to: index - 16,

									pawn_moving_twice: true,

									..Default::default()
								});
							}
						}


						// Normal capturing
						for dir in [Point::new(-1, -1), Point::new(1, -1)] {
							let new_p = p + dir;

							if new_p.x < 0
							|| new_p.x > 7 {
								continue;
							}

							let new_index = (new_p.x + new_p.y * 8) as usize;

							if self.game_data.board[new_index].piece_type != PieceType::None
							&& self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
								if will_promote {
									for t in PROMOTABLE_PIECES {
										result.push(PieceMove {
											from: index,
											to: new_index,

											promotion_type: t,

											..Default::default()
										});
									}
								} else {
									result.push(PieceMove {
										from: index,
										to: new_index,

										..Default::default()
									});
								}
							}
						}



						// En Passant
						if self.game_data.last_move.pawn_moving_twice {
							if self.game_data.last_move.to == index - 1 {
								result.push(PieceMove {
									from: index,
									to: index - 9,

									en_passant_capture: Some(index - 1),

									..Default::default()
								});
							} else if self.game_data.last_move.to == index + 1 {
								result.push(PieceMove {
									from: index,
									to: index - 7,

									en_passant_capture: Some(index + 1),

									..Default::default()
								});
							}
						}
					}














				} else {











					if p.y < 7 {
						let will_promote = p.y + 1 == 7;


						// Moving forward
						if self.game_data.board[index + 8].piece_type == PieceType::None {
							if will_promote {
								for t in PROMOTABLE_PIECES {
									result.push(PieceMove {
										from: index,
										to: index + 8,

										promotion_type: t,

										..Default::default()
									});
								}
							} else {
								result.push(PieceMove {
									from: index,
									to: index + 8,

									..Default::default()
								});
							}

							if index + 16 < 63
							&& p.y == 1
							&& self.game_data.board[index + 16].piece_type == PieceType::None {
								result.push(PieceMove {
									from: index,
									to: index + 16,

									pawn_moving_twice: true,

									..Default::default()
								});
							}
						}


						// Normal capturing
						for dir in [Point::new(-1, 1), Point::new(1, 1)] {
							let new_p = p + dir;

							if new_p.x < 0
							|| new_p.x > 7 {
								continue;
							}

							let new_index = (new_p.x + new_p.y * 8) as usize;

							if self.game_data.board[new_index].piece_type != PieceType::None
							&& self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
								if will_promote {
									for t in PROMOTABLE_PIECES {
										result.push(PieceMove {
											from: index,
											to: new_index,

											promotion_type: t,

											..Default::default()
										});
									}
								} else {
									result.push(PieceMove {
										from: index,
										to: new_index,

										..Default::default()
									});
								}
							}
						}



						// En Passant
						if self.game_data.last_move.pawn_moving_twice {
							if self.game_data.last_move.to == index - 1 {
								result.push(PieceMove {
									from: index,
									to: index + 7,

									en_passant_capture: Some(index - 1),

									..Default::default()
								});
							} else if self.game_data.last_move.to == index + 1 {
								result.push(PieceMove {
									from: index,
									to: index + 9,

									en_passant_capture: Some(index + 1),

									..Default::default()
								});
							}
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

						if self.game_data.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,

									..Default::default()
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

					if self.game_data.board[new_index].piece_type == PieceType::None
					|| self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
						result.push(PieceMove {
							from: index,
							to: new_index,

							..Default::default()
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

						if self.game_data.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,

									..Default::default()
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

						if self.game_data.board[new_index].piece_type == PieceType::None {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
								result.push(PieceMove {
									from: index,
									to: new_index,

									..Default::default()
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

					if self.game_data.board[new_index].piece_type == PieceType::None
					|| self.game_data.board[new_index].is_white != self.game_data.board[index].is_white {
						result.push(PieceMove {
							from: index,
							to: new_index,

							..Default::default()
						});
					}
				}

				if self.game_data.board[index].is_white {
					if self.game_data.white_can_short_castle
					&& self.game_data.board[index + 1].piece_type == PieceType::None
					&& self.game_data.board[index + 2].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index + 2,

							short_castle: true,

							..Default::default()
						});
					}

					if self.game_data.white_can_long_castle
					&& self.game_data.board[index - 1].piece_type == PieceType::None
					&& self.game_data.board[index - 2].piece_type == PieceType::None
					&& self.game_data.board[index - 3].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index - 2,

							long_castle: true,

							..Default::default()
						});
					}
				} else {
					if self.game_data.black_can_short_castle
					&& self.game_data.board[index + 1].piece_type == PieceType::None
					&& self.game_data.board[index + 2].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index + 2,

							short_castle: true,

							..Default::default()
						});
					}

					if self.game_data.black_can_long_castle
					&& self.game_data.board[index - 1].piece_type == PieceType::None
					&& self.game_data.board[index - 2].piece_type == PieceType::None
					&& self.game_data.board[index - 3].piece_type == PieceType::None {
						result.push(PieceMove {
							from: index,
							to: index - 2,

							long_castle: true,

							..Default::default()
						});
					}
				}
			}

			_ => {}
		}

		result
	}
}