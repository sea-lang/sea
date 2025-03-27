from typing import Callable
from .backend import Backend
from .compiler import Compiler, SeaFunction, SeaRecord, SeaType

class Backend_C(Backend):
	def __init__(self, compiler: Compiler, output_file: str):
		super().__init__(compiler, output_file)
		self.line_ending = ';\n'

	def type(self, type: SeaType) -> str:
		return type.name + ('*' * type.pointers) + ('[]' * type.arrays)

	def typed_id(self, type: SeaType, id: str) -> str:
		return type.name + ' ' + ('*' * type.pointers) + id + ('[]' * type.arrays)

	def use(self, module: str):
		pass

	def rec(self, name: str, record: SeaRecord):
		self.writeln('typedef struct {')
		self.depth += 1
		for fieldname, typedesc in record.fields.items():
			self.write(self.typed_id(typedesc, fieldname))
			self.write(';\n', False)
		self.depth -= 1
		self.writeln(f'}} {name};\n') # newline for spacing
		self.compiler.add_record(name, record)

	def fun(self, name: str, func: SeaFunction):
		self.write(f'{self.type(func.returns)} {name}(')

		end = len(func.params) - 1
		for index, (name, typedesc) in enumerate(func.params.items()):
			self.write(self.typed_id(typedesc, name))
			if index != end:
				self.write(', ', False)

		self.writeln(f')', False) # writeln for allman-esque indents

	def var(self, name: str, type: SeaType, value: Callable):
		self.write(self.typed_id(type, name))
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type)

	def let(self, name: str, type: SeaType, value: Callable):
		self.write('const ')
		self.write(self.typed_id(type, name), False)
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type)

	def assign(self, name: str, value: Callable):
		self.write(name)
		self.write(' = ', False)
		value()

	def ret(self, value: Callable):
		self.write('return ')
		value()


	def block_start(self):
		self.writeln('{')
		self.depth += 1

	def block_end(self):
		self.depth -= 1
		self.writeln('}')

	def invoke(self, it: str, args: str):
		self.write(f'{it}({args})')

	def if_(self, cond: Callable):
		self.write('if (')
		cond()
		self.write(')')

	def else_(self):
		self.writeln(f'else ')

	def for_(self, define: Callable, cond: Callable, inc: Callable):
		self.writeln(f'for ({define}; {cond}; {inc}) ')

	def each(self, var: str, of: str):
		typ = self.compiler.find_type_of(of)
		if typ == None:
			self.compiler.panic(f'no such variable: {of}')
		typ = self.type(typ)
		self.writeln(
			'for (\n'
			f'\t({typ} {var}, int i = 0, size_t length = sizeof({of}) / sizeof({typ}));\n'
			'\ti < length;\n'
			'\ti++\n'
			')'
		)


	def true(self):
		self.write('true', False)

	def false(self):
		self.write('false', False)

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
		self.write(it, False)

	def id(self, it: str):
		self.write(it, False)

	def string(self, it: str):
		self.write(f'"{it}"', False)

	def array(self, items: list[Callable]):
		print(items)
		# if len(items) > 4: # Write on multiple lines
		# 	self.writeln('{', False)
		# 	self.depth += 1
		# 	for item in items:
		# 		item()
		# 		self.writeln(',', False)
		# 	self.write('}')
		# 	self.depth -= 1
		# else: # Write on one line
		self.write('{', False)
		for item in items:
			item()
			self.write(',', False)
		self.write('}', False)

	def new(self, rec: str, items: list[Callable]):
		if len(items) > 4: # Write on multiple lines
			self.writeln(f'({rec}){{', False)
			self.depth += 1
			for item in items:
				item()
				self.writeln(',', False)
			self.write('}')
			self.depth -= 1
		else: # Write on one line
			self.write(f'({rec}){{', False)
			for item in items:
				item()
				self.write(',', False)
			self.write('}', False)


	def raw(self, code: str):
		self.write(code.strip(' '))
		if self.depth == 0:
			self.write('\n', False)


	def comment(self, text: str):
		self.writeln('//' + text)

	def multiline_comment(self, text: str):
		self.writeln('/*' + text + '*/')
