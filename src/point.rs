#[derive(Copy, Clone, PartialEq)]
pub struct Point {
	pub x: i8,
	pub y: i8,
}

impl Point {
	pub fn new(x: i8, y: i8) -> Self {
		Self {
			x,
			y,
		}
	}

	pub fn from_index(index: usize) -> Point {
		let x = (index % 8) as i8;
		let y = (index as f32 / 8.0).floor() as i8;

		Point::new(x, y)
	}

	pub fn difference(&self, other: Point) -> Point {
		Point::new(
			other.x - self.x,
			other.y - self.y,
		)
	}

	pub fn abs_difference(&self, other: Point) -> Point {
		Point::new(
			(other.x - self.x).abs(),
			(other.y - self.y).abs(),
		)
	}

	pub fn point_inbetween(a: Point, b: Point, c: Point) -> bool {
		for x in a.x..=c.x {
			for y in a.y..=c.y {
				if b.x == x
				&& b.y == y {
					return true;
				}
			}
		}

		false
	}
}