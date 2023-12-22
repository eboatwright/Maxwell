use std::io::Write;
use std::fs::File;

pub struct Log {
	file: Option<File>,
}

impl Log {
	pub fn none() -> Self {
		Self {
			file: None,
		}
	}

	pub fn new() -> Self {
		Self {
			file: Some(
				File::create(format!("logs/log_{:?}.txt", chrono::offset::Local::now()))
					.expect("Failed to open log.txt")
			),
		}
	}

	pub fn write(&mut self, text: String) {
		if let Some(file) = &mut self.file {
			writeln!(file, "{}", text).expect("Failed to write to log file");
		}
	}
}