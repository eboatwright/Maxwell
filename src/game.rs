use std::collections::HashMap;
use crate::utils::*;
use crate::piece::*;
use crate::Point;

#[derive(Clone, Debug, PartialEq)]
#[derive(Eq, Hash)]
pub struct GameData {
	pub board: [u8; 64],
	pub whites_turn: bool,

	pub last_move: PieceMove,

	pub promoting: Option<usize>,

	pub white_can_short_castle: bool,
	pub white_can_long_castle: bool,

	pub black_can_short_castle: bool,
	pub black_can_long_castle: bool,
}

#[derive(Clone)]
pub struct Game {
	pub game_data: GameData,
	pub last_game_data: GameData,
}

impl Game {
	pub fn new(board: &'static str) -> Self {
		let game_data = GameData {
			board: generate_starting_position(board),
			whites_turn: true,

			last_move: PieceMove::default(),

			promoting: None,

			white_can_short_castle: true,
			white_can_long_castle: true,

			black_can_short_castle: true,
			black_can_long_castle: true,
		};

		Self {
			game_data: game_data.clone(),
			last_game_data: game_data,
		}
	}

	pub fn eval(&mut self) -> i32 {
		let mut eval = 0;

		for i in 0..64 {
			if self.game_data.board[i] != 0 {
				let piece_value = get_worth_for_piece(self.game_data.board[i], i);
				if is_white(self.game_data.board[i]) {
					eval += piece_value;
				} else {
					eval -= piece_value;
				}
			}
		}

		eval += self.get_legal_moves_for_color(true).len() as i32;
		eval -= self.get_legal_moves_for_color(false).len() as i32;

		eval
	}

	pub fn make_move(&mut self, m: PieceMove) {
		self.last_game_data = self.game_data.clone();
		self.game_data.last_move = m;

		self.game_data.board[m.to] = self.game_data.board[m.from];
		self.game_data.board[m.from] = 0;

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

		match self.game_data.board[m.to] & 0b_0111 {
			PAWN => {
				if let Some(i) = m.en_passant_capture {
					self.game_data.board[i] = 0;
				} else if p.y == 0
				|| p.y == 7 {
					if self.game_data.whites_turn {
						self.game_data.promoting = Some(m.to);
					} else {
						self.game_data.board[m.to] = (self.game_data.board[m.to] & 0b_1000) | m.promotion_type;
					}
				}
			}

			KING => {
				if m.short_castle {
					self.game_data.board[m.to - 1] = self.game_data.board[m.to + 1];
					self.game_data.board[m.to + 1] = 0;
				} else if m.long_castle {
					self.game_data.board[m.to + 1] = self.game_data.board[m.to - 2];
					self.game_data.board[m.to - 2] = 0;
				}

				if is_white(self.game_data.board[m.to]) {
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
		self.game_data = self.last_game_data.clone();
	}

	pub fn promote(&mut self, piece: u8) {
		let p = self.game_data.promoting.unwrap();
		self.game_data.board[p] = (self.game_data.board[p] & 0b_1000) | piece;
		self.game_data.promoting = None;
	}

	pub fn get_legal_moves_for_color(&mut self, white: bool) -> Vec<PieceMove> {
		let mut result = vec![];

		for i in 0..64 {
			if self.game_data.board[i] != 0
			&& is_white(self.game_data.board[i]) == white {
				result = [result, self.get_legal_moves_for_piece(i)].concat();
			}
		}

		result
	}

	pub fn king_in_check(&mut self, white: bool) -> bool {
		for i in 0..64 {
			if self.game_data.board[i] != 0
			&& is_white(self.game_data.board[i]) == !white {
				for legal_move in self.get_moves_for_piece(i) {
					if self.game_data.board[legal_move.to] & 0b_0111 == KING
					&& is_white(self.game_data.board[legal_move.to]) == white {
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
				if self.king_in_check(self.game_data.whites_turn) {
					result.remove(i);
					continue;
				}

				self.make_move(
					PieceMove {
						to: (result[i].from as i8 + if result[i].short_castle { 1 } else { -1 }) as usize,
						short_castle: false,
						long_castle: false,
						..result[i]
					}
				);

				if self.king_in_check(!self.game_data.whites_turn) {
					result.remove(i);
					already_removed = true;
				}

				self.undo_last_move();
			}

			if already_removed {
				continue;
			}

			self.make_move(result[i]);

			if self.king_in_check(!self.game_data.whites_turn) {
				result.remove(i);
			}

			self.undo_last_move();
		}

		result
	}

	fn get_moves_for_piece(&mut self, index: usize) -> Vec<PieceMove> {
		let mut result = vec![];

		match self.game_data.board[index] & 0b_0111 {
			PAWN => {
				let p = Point::from_index(index);

				if is_white(self.game_data.board[index]) {







					if p.y > 0 {
						let will_promote = p.y - 1 == 0;


						// Moving forward
						if self.game_data.board[index - 8] == 0 {
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
							&& self.game_data.board[index - 16] == 0 {
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

							if self.game_data.board[new_index] != 0
							&& is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
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
						if self.game_data.board[index + 8] == 0 {
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
							&& self.game_data.board[index + 16] == 0 {
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

							if self.game_data.board[new_index] != 0
							&& is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
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

			KNIGHT => {
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

					if self.game_data.board[new_index] == 0
					|| is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
						result.push(PieceMove {
							from: index,
							to: new_index,

							..Default::default()
						});
					}
				}
			}

			BISHOP => {
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

						if self.game_data.board[new_index] == 0 {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
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

			ROOK => {
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

						if self.game_data.board[new_index] == 0 {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
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

			QUEEN => {
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

						if self.game_data.board[new_index] == 0 {
							result.push(PieceMove {
								from: index,
								to: new_index,

								..Default::default()
							});
						} else {
							if is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
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

			KING => {
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

					if self.game_data.board[new_index] == 0
					|| is_white(self.game_data.board[new_index]) != is_white(self.game_data.board[index]) {
						result.push(PieceMove {
							from: index,
							to: new_index,

							..Default::default()
						});
					}
				}

				if is_white(self.game_data.board[index]) {
					if self.game_data.white_can_short_castle
					&& self.game_data.board[index + 1] == 0
					&& self.game_data.board[index + 2] == 0 {
						result.push(PieceMove {
							from: index,
							to: index + 2,

							short_castle: true,

							..Default::default()
						});
					}

					if self.game_data.white_can_long_castle
					&& self.game_data.board[index - 1] == 0
					&& self.game_data.board[index - 2] == 0
					&& self.game_data.board[index - 3] == 0 {
						result.push(PieceMove {
							from: index,
							to: index - 2,

							long_castle: true,

							..Default::default()
						});
					}
				} else {
					if self.game_data.black_can_short_castle
					&& self.game_data.board[index + 1] == 0
					&& self.game_data.board[index + 2] == 0 {
						result.push(PieceMove {
							from: index,
							to: index + 2,

							short_castle: true,

							..Default::default()
						});
					}

					if self.game_data.black_can_long_castle
					&& self.game_data.board[index - 1] == 0
					&& self.game_data.board[index - 2] == 0
					&& self.game_data.board[index - 3] == 0 {
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