pub struct ValueHolder<T: Copy> {
	pub current: T,
	pub index: usize,
	pub history: Vec<T>,
}

impl<T: Copy> ValueHolder<T> where T: Copy {
	pub fn new(current: T) -> Self {
		Self {
			current,
			index: 0,
			history: vec![current],
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
}