import random

class Matrix:
	def __init__(self, rows, cols):
		self.rows = rows
		self.cols = cols

		self.data = []
		for row in range(rows):
			new_row = []
			for col in range(cols):
				new_row.append(0.0)
			self.data.append(new_row)

	def from_2d_list(input_2d_list):
		result = Matrix(len(input_2d_list), len(input_2d_list[0]))
		result.data = input_2d_list
		return result

	def fill_zeros(self):
		for row in range(self.rows):
			for col in range(self.cols):
				self.data[row][col] = 0.0

	def flatten(self):
		result = []

		for row in range(self.rows):
			for col in range(self.cols):
				result.append(self.data[row][col])

		return result

	def transpose(a):
		result = Matrix(a.cols, a.rows)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[col][row] = a.data[row][col]

		return result

	def random(rows, cols):
		result = Matrix(rows, cols)

		for row in range(rows):
			for col in range(cols):
				result.data[row][col] = random.uniform(-0.8, 0.8)

		return result

	def add(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] + b.data[row][col]

		return result

	def subtract(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] - b.data[row][col]

		return result

	def multiply(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] * b.data[row][col]

		return result

	def divide(a, b):
		result = Matrix(a.rows, a.cols)

		for row in range(a.rows):
			for col in range(a.cols):
				result.data[row][col] = a.data[row][col] / b.data[row][col]

		return result

	def divide_by_num(mat, num):
		result = Matrix(mat.rows, mat.cols)

		for row in range(mat.rows):
			for col in range(mat.cols):
				result.data[row][col] = mat.data[row][col] / num

		return result

	def dot(a, b):
		result = Matrix(a.rows, b.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				sum_of_column = 0

				for offset in range(a.cols):
					sum_of_column += a.data[row][offset] * b.data[offset][col]

				result.data[row][col] = sum_of_column

		return result

	def scale(m, s):
		result = Matrix(m.rows, m.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				result.data[row][col] = m.data[row][col] * s

		return result

	def pow(m, e):
		result = Matrix(m.rows, m.cols)

		for row in range(result.rows):
			for col in range(result.cols):
				result.data[row][col] = m.data[row][col] ** e

		return result

	def map(m, fn):
		for row in range(m.rows):
			for col in range(m.cols):
				m.data[row][col] = fn(m.data[row][col])
		return m