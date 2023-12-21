pub const PAWN:   usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK:   usize = 3;
pub const QUEEN:  usize = 4;
pub const KING:   usize = 5;

pub const BLACK_PAWN:   usize = PAWN;
pub const BLACK_KNIGHT: usize = KNIGHT;
pub const BLACK_BISHOP: usize = BISHOP;
pub const BLACK_ROOK:   usize = ROOK;
pub const BLACK_QUEEN:  usize = QUEEN;
pub const BLACK_KING:   usize = KING;

pub const WHITE_PAWN:   usize = PAWN   + 6;
pub const WHITE_KNIGHT: usize = KNIGHT + 6;
pub const WHITE_BISHOP: usize = BISHOP + 6;
pub const WHITE_ROOK:   usize = ROOK   + 6;
pub const WHITE_QUEEN:  usize = QUEEN  + 6;
pub const WHITE_KING:   usize = KING   + 6;

pub const NO_PIECE: usize = 12;
pub const PIECE_COUNT: usize = 12;

pub const PROMOTABLE: [u8; 4] = [
	KNIGHT as u8,
	BISHOP as u8,
	ROOK as u8,
	QUEEN as u8,
];

pub fn is_piece_white(piece: usize) -> bool {
	piece > BLACK_KING
}

pub fn get_piece_type(piece: usize) -> usize {
	piece % 6
}

pub fn build_piece(is_white: bool, piece_type: usize) -> usize {
	piece_type + (if is_white { 6 } else { 0 })
}

pub fn char_to_piece(piece: char) -> usize {
	match piece {
		'p' => BLACK_PAWN,
		'n' => BLACK_KNIGHT,
		'b' => BLACK_BISHOP,
		'r' => BLACK_ROOK,
		'q' => BLACK_QUEEN,
		'k' => BLACK_KING,

		'P' => WHITE_PAWN,
		'N' => WHITE_KNIGHT,
		'B' => WHITE_BISHOP,
		'R' => WHITE_ROOK,
		'Q' => WHITE_QUEEN,
		'K' => WHITE_KING,

		_   => NO_PIECE,
	}
}

pub fn piece_to_char(piece: usize) -> char {
	match piece {
		BLACK_PAWN   => 'p',
		BLACK_KNIGHT => 'n',
		BLACK_BISHOP => 'b',
		BLACK_ROOK   => 'r',
		BLACK_QUEEN  => 'q',
		BLACK_KING   => 'k',

		WHITE_PAWN   => 'P',
		WHITE_KNIGHT => 'N',
		WHITE_BISHOP => 'B',
		WHITE_ROOK   => 'R',
		WHITE_QUEEN  => 'Q',
		WHITE_KING   => 'K',

		_ => ' ',
	}
}