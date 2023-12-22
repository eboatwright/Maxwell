pub const WHITE_CASTLE_LONG: u8  = 0b1000;
pub const WHITE_CASTLE_SHORT: u8 = 0b0100;
pub const BLACK_CASTLE_SHORT: u8 = 0b0010;
pub const BLACK_CASTLE_LONG: u8  = 0b0001;

pub const SHORT_CASTLING_RIGHTS: [u8; 2] = [BLACK_CASTLE_SHORT, WHITE_CASTLE_SHORT];
pub const LONG_CASTLING_RIGHTS: [u8; 2] = [BLACK_CASTLE_LONG, WHITE_CASTLE_LONG];

pub const ALL_WHITE_CASTLING_RIGHTS: u8 = 0b1100;
pub const ALL_BLACK_CASTLING_RIGHTS: u8 = 0b0011;

pub const SHORT_CASTLE_MASK: [u64; 2] = [
	(1 << 5) | (1 << 6),
	(1 << 61) | (1 << 62),
];

pub const LONG_CASTLE_MASK: [u64; 2] = [
	(1 << 2) | (1 << 3),
	(1 << 58) | (1 << 59),
];

// Although the logic behind long castling makes sense, in practice it's kinda wonky,
// Because of this one situation: 4k3/8/8/5b2/8/8/8/R3K3 w Q - 0 1
// Where white can still long castle
pub const EXTRA_LONG_CASTLE_SQUARE_CHECK: [u64; 2] = [
	1 << 1,
	1 << 57,
];

pub struct CastlingRights {
	pub current: u8,
	pub history: Vec<u8>,
	pub index: usize,
}

impl CastlingRights {
	pub fn new(castling_rights: u8) -> Self {
		Self {
			current: castling_rights,
			history: vec![castling_rights],
			index: 0,
		}
	}

	pub fn push(&mut self) {
		self.index += 1;
		if self.index >= self.history.len() {
			self.history.push(self.current);
		} else {
			self.history[self.index] = self.current;
		}
	}

	pub fn pop(&mut self) {
		self.index -= 1;
		self.current = self.history[self.index];
	}

	pub fn clear(&mut self) {
		self.index = 0;
		self.current = self.history[0];
		self.history.clear();
		self.push();
	}

	pub fn print(&self) {
		if self.current & WHITE_CASTLE_LONG != 0 {
			println!("White can castle long");
		}

		if self.current & WHITE_CASTLE_SHORT != 0 {
			println!("White can castle short");
		}

		if self.current & BLACK_CASTLE_LONG != 0 {
			println!("Black can castle long");
		}

		if self.current & BLACK_CASTLE_SHORT != 0 {
			println!("Black can castle short");
		}
	}
}