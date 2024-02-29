use super::selfplay::play_games;
use super::network::Network;
use crate::nnue_trainer::config;
use rand::thread_rng;
use rand::prelude::SliceRandom;

pub struct DataPoint {
	pub fen: String,
	pub outcome: f32,
}

pub fn nnue_train() {
	println!("### MAXWELL NNUE TRAINER 2.0 ###\n");

	let mut network = Network::new();
	let mut rng = thread_rng();

	let mut training_cycle = 0;
	let mut total_games = 0;
	let mut total_positions = 0;

	network.save_weights(total_positions);

	loop {
		println!("Training cycle {}:", training_cycle + 1);

		let mut data_points = play_games(network.clone());

		total_games += config::GAMES_PER_MATCH;
		total_positions += data_points.len();

		println!("Training network...");

		for epoch in 0..config::EPOCHS {
			println!("Epoch {}...", epoch + 1);

			data_points.shuffle(&mut rng);

			let mut data_point_index = 0;
			while data_point_index < data_points.len() {
				let next_index = usize::min(data_point_index + config::MINIBATCH_SIZE, data_points.len());

				network.back_prop(data_point_index, &data_points[data_point_index..next_index]);

				data_point_index = next_index;
			}
		}

		println!("\nDone training!\n");

		network.save_weights(total_positions);
		training_cycle += 1;

		println!("Total games: {}", total_games);
		println!("Total positions: {}\n", total_positions);
	}
}