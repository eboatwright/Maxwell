import sys
import random

import chess
import chess.polyglot

if __name__ == "__main__":
	board = chess.Board()
	# board.set_fen(sys.argv[1])

	# opening_line = ""

	# This file path is because this python script gets called from a separate directory
	with chess.polyglot.open_reader("src/nnue_trainer/opening_book/Perfect2021.bin") as reader:
		number_of_book_moves = random.randint(2, 10)
		for i in range(number_of_book_moves):
			move = reader.choice(board).move
			# opening_line = f"{opening_line}{move} "

			board.push(move)

	# print(opening_line[:-1])
	print(board.fen())