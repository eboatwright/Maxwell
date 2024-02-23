# The developer of the Weiawaga engine's NNUE trainer: Mimir, was very very helpful in making this!
# https://github.com/Heiaha/Mimir/

import random

import chess
import chess.engine
import chess.polyglot
import asyncio

import config
from neural_network import NeuralNetwork


class TrainingResults:
	def __init__(self):
		self.games = 0
		self.positions = 0


class DataPoint:
	def __init__(self, fen, outcome):
		self.fen = fen
		self.outcome = outcome


training_results = TrainingResults()
nn = NeuralNetwork()
data_points = []


async def play_game():
	transport, maxwell_engine = await chess.engine.popen_uci(["./../target/release/maxwell", "debug_output=false"])
	board = chess.Board()

	with chess.polyglot.open_reader("Perfect2021.bin") as reader:
		number_of_book_moves = random.randint(1, 10)

		for i in range(number_of_book_moves):
			board.push(reader.choice(board).move)

	fen_strings = []

	while not board.is_game_over(claim_draw=True):
		if random.randint(0, 100) < config.PERC_CHANCE_FOR_RANDOM_MOVE:
			board.push(random.choice(list(board.legal_moves)))
		else:
			result = await maxwell_engine.play(board, chess.engine.Limit(depth=config.DEPTH_PER_MOVE))
			board.push(result.move)

		fen_strings.append(board.fen())

		if board.fullmove_number >= config.MAX_MOVES:
			break

	game_outcome = 0.0

	# For some reason when it detects a threefold-repetition or a 50 move draw, it returns None instead of a draw :P
	if outcome := board.outcome(): 
		if outcome.winner == chess.WHITE:
			game_outcome = 1.0
		elif outcome.winner == chess.BLACK:
			game_outcome = -1.0

	for fen in fen_strings:
		data_points.append(DataPoint(fen, game_outcome))

	await maxwell_engine.quit()


async def play_games():
	games_completed = 0

	pending = {asyncio.create_task(play_game()) for _ in range(config.CONCURRENT_GAMES)}

	while len(pending) > 0:
		print(f"Playing self-play games... {games_completed}/{config.GAMES_PER_MATCH}", end="\r", flush=True)

		completed, pending = await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)

		for completed_task in completed:
			games_completed += 1

			if games_completed + len(pending) < config.GAMES_PER_MATCH:
				pending.add(asyncio.create_task(play_game()))


if __name__ == "__main__":
	print("### MAXWELL NNUE TRAINER ###\n")

	training_cycle = 0
	nn.save_weights() # Save the initial randomized weights so that the program has the same weights as the trainer

	while True:
		data_points = []

		print(f"Training cycle {training_cycle + 1}:")

		asyncio.run(play_games())

		training_results.games += config.GAMES_PER_MATCH
		training_results.positions += len(data_points)

		print("\nSelf-play done!\n")
		print("Training network...")

		for epoch in range(config.EPOCHS_PER_TRAIN):
			print(f"Epoch {epoch + 1}...")

			random.shuffle(data_points)

			data_point_index = 0

			while data_point_index < len(data_points):
				next_index = min(data_point_index + config.MINIBATCH_SIZE, len(data_points))

				nn.back_prop(data_point_index, next_index, data_points[data_point_index:next_index])

				data_point_index = next_index

		print("Done training!")
		# print("Calculating total error...")
		# print(f"Total error on data set: {nn.get_total_error(data_points)}")
		# print("Done!\n")

		print(f"Total games played: {training_results.games}")
		print(f"Total positions trained on: {training_results.positions}\n")

		training_cycle += 1
		nn.save_weights()