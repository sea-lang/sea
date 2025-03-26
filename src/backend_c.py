from typing import Callable
from backend import Backend
from compiler import Compiler, SeaFunction, SeaRecord, SeaType

class Backend_C(Backend):
	def __init__(self, compiler: Compiler, output_file: str):
		super().__init__(compiler, output_file)
		self.line_ending = ';\n'

	def type(self, type: SeaType) -> str:
		return ('*' * type.pointers) + type.name + ('[]' * type.arrays)

	def use(self, module: str):
		pass

	def rec(self, name: str, record: SeaRecord):
		self.writeln('typedef struct {\n')
		for name, typedesc in record.fields.items():
			self.writeln(f'\t{self.type(typedesc)} {name};')
		self.writeln(f'}} {name};\n')
		self.compiler.add_record(name, record)

	def fun(self, name: str, func: SeaFunction):
		self.write(f'{self.type(func.returns)} {name}(')

		end = len(func.params) - 1
		for index, (name, typedesc) in enumerate(func.params.items()):
			self.write(f'{self.type(typedesc)} {name}')
			if index != end:
				self.write(', ')

		self.writeln(f')')

	def var(self, name: str, type: SeaType, value: Callable):
		self.writeln(f'{self.type(type)} {name} = ')
		value()
		self.compiler.add_variable(name, type)

	def let(self, name: str, type: SeaType, value: Callable):
		self.writeln(f'const {self.type(type)} {name} = ')
		value()
		self.compiler.add_variable(name, type)

	def ret(self, value: Callable):
		self.writeln(f'return ')
		value()


	def block_start(self):
		self.writeln('{')

	def block_end(self):
		self.writeln('}')

	def invoke(self, it: str, args: str):
		self.writeln(f'{it}({args})')

	def if_(self, cond: str):
		self.writeln(f'if ({cond}) ')

	def else_(self):
		self.writeln(f'else ')

	def for_(self, define: str, cond: str, inc: str):
		self.writeln(f'for ({define}; {cond}; {inc}) ')

	def each(self, var: str, of: str):
		typ = self.compiler.find_type_of(var)
		if typ == None:
			self.compiler.panic(f'no such variable: {var}')
		typ = self.type(typ)
		self.writeln(
			'for (\n'
			f'\t({typ} {var}, int i = 0, size_t length = sizeof({of}) / sizeof({typ}));\n'
			'\ti < length;\n'
			'\ti++\n'
			')'
		)


	def true(self):
		self.write('true')
	def false(self):
		self.write('false')

	def _op(self, op: str, left: Callable, right: Callable):
		left()
		self.write(op)
		right()

	def _prefix_unary_op(self, op: str, value: Callable):
		self.write(op)
		value()

	def _postfix_unary_op(self, op: str, value: Callable):
		self.write(op)
		value()

	def dot(self, left: Callable, right: Callable): self._op('.', left, right)
	def not_(self, value: Callable): self._prefix_unary_op('!', value)
	def and_(self, left: Callable, right: Callable): self._op('&&', left, right)
	def or_(self, left: Callable, right: Callable): self._op('||', left, right)
	def eq(self, left: Callable, right: Callable): self._op('==', left, right)
	def neq(self, left: Callable, right: Callable): self._op('!=', left, right)
	def gt(self, left: Callable, right: Callable): self._op('>', left, right)
	def gteq(self, left: Callable, right: Callable): self._op('>=', left, right)
	def lt(self, left: Callable, right: Callable): self._op('<', left, right)
	def lteq(self, left: Callable, right: Callable): self._op('<=', left, right)
	def inc(self, value: Callable): self._postfix_unary_op('++', value)
	def dec(self, value: Callable): self._postfix_unary_op('--', value)


	def add(self, left: Callable, right: Callable): self._op('+', left, right)
	def sub(self, left: Callable, right: Callable): self._op('-', left, right)
	def mul(self, left: Callable, right: Callable): self._op('*', left, right)
	def div(self, left: Callable, right: Callable): self._op('/', left, right)
	def mod(self, left: Callable, right: Callable): self._op('%', left, right)


	def number(self, it: str):
		self.write(it)

	def id(self, it: str):
		self.write(it)

	def string(self, it: str):
		self.write(f'"{it}"')


	def raw(self, code: Callable):
		code()
