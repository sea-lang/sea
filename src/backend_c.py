from typing import Callable, Optional
from .backend import Backend
from .compiler import Compiler, SeaFunction, SeaRecord, SeaType

class Backend_C(Backend):
	def __init__(self, compiler: Compiler, output_file: str):
		super().__init__(compiler, output_file)
		self.line_ending = ';\n'

	def type(self, type: SeaType) -> str:
		name = type.name
		for param in type.params:
			name += f'_{param}'
		return name + ('*' * type.pointers) + ''.join([f'[{it}]' for it in type.arrays])

	def typed_id(self, type: SeaType, id: str) -> str:
		name = type.name
		for param in type.params:
			name += f'_{param}'
		return name + ' ' + ('*' * type.pointers) + id + ''.join([f'[{it}]' for it in type.arrays])


	def file_begin(self):
		self.writeln(f'#pragma region "mod: {self.module_stack[-1]}"')

	def file_end(self):
		self.writeln(f'#pragma endregion "mod: {self.module_stack[-1]}"')


	def rec(self, name: str, record: SeaRecord):
		self.writeln(f'struct {name};')
		self.writeln(f'typedef struct {name} {name};')
		self.writeln(f'struct {name} {{')
		self.depth += 1
		for fieldname, typedesc in record.fields.items():
			self.write(self.typed_id(typedesc, fieldname))
			self.write(';\n', False)
		self.depth -= 1
		# self.writeln(f'}} {name};\n') # newline for spacing
		self.writeln('};\n') # newline for spacing
		self.compiler.add_record(name, record)

	def fun_begin(self, name: str, func: SeaFunction):
		self.write(f'{self.type(func.returns)} {name}(')
		self.compiler.push_scope()
		self.compiler.add_function(name, func)

		end = len(func.params) - 1
		for index, (name, typedesc) in enumerate(func.params.items()):
			self.write(self.typed_id(typedesc, name))
			if index != end:
				self.write(', ', False)
			self.compiler.add_variable(name, typedesc)

		self.writeln(f')', False) # writeln for allman-esque braces

	def fun_end(self):
		if self.compiler.current_template is not None:
			return

		self.writeln('', False) # writeln for spacing
		self.compiler.pop_scope()

	def def_(self, name: str, type: SeaType):
		self.writeln(f'typedef {self.typed_id(type, name)};\n')

	def var(self, name: str, type: SeaType, value: Callable):
		self.write(self.typed_id(type, name), False)
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type)

	def let(self, name: str, type: SeaType, value: Callable):
		self.write('const ', False)
		self.write(self.typed_id(type, name), False)
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type, True)

	def assign(self, name: Callable, value: Callable):
		name()
		self.write(' = ', False)
		value()

	def ret(self, value: Callable):
		self.write('return ')
		value()


	def block_begin(self):
		self.writeln('{', False)
		self.depth += 1

	def block_end(self):
		self.depth -= 1
		self.writeln('}')

	def invoke(self, id: str, args: list[Callable]):
		self.write(id, False)
		self.write('(', False)
		for arg in args:
			arg()
			if arg != args[-1]:
				self.write(', ', False)
		self.write(')', False)

	def if_(self, cond: Callable):
		self.write('if (')
		cond()
		self.write(')', False)

	def else_(self):
		self.write('else ', False)

	def for_c_style(self, define: Callable, cond: Callable, inc: Callable):
		self.write('for (')
		define()
		self.write('; ', False)
		cond()
		self.write('; ', False)
		inc()
		self.write(') ', False)

	def for_single_expr(self, cond: Callable):
		self.write('while (')
		cond()
		self.write(') ', False)

	def for_to(self, var: Optional[str], from_: Callable, to: Callable):
		if var is None:
			var = '_i'
		self.write(f'for (int _to = 1+')
		to()
		self.write(f', {var} = ', False)
		from_()
		self.write(f'; {var} < _to; {var}++) ', False)

	# TODO: Use a trait for this
	def each_begin(self, var: str, of: str):
		typ = self.compiler.find_type_of(of)
		if typ == None:
			self.compiler.panic(f'no such variable: {of}')
		typ = SeaType(typ.pointers, typ.name, typ.params, typ.arrays)

		self.writeln('{')
		self.depth += 1

		self.write(self.typed_id(typ, var))
		self.writeln(';', False)
		self.writeln(f'const size_t _l = sizeof({of}) / sizeof({self.type(typ)});')
		self.writeln('for (int _i = 0; _i < _l; _i++) {')
		self.depth += 1
		self.writeln(f'{var} = {of}[_i];')
		self.force_indent_next = True

	def each_end(self):
		self.depth -= 1
		self.writeln('}')
		self.depth -= 1
		self.writeln('}')


	def true(self):
		self.write('true', False)

	def false(self):
		self.write('false', False)

	def _op(self, op: str, left: Callable, right: Callable):
		left()
		self.write(op, False)
		right()

	def _prefix_unary_op(self, op: str, value: Callable):
		self.write(op, False)
		value()

	def _postfix_unary_op(self, op: str, value: Callable):
		self.write(op, False)
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


	def group_expr(self, it: Callable):
		self.write('(', False)
		it()
		self.write(')', False)

	def number(self, it: str):
		self.write(it, False)

	def id(self, it: str):
		self.write(it, False)

	def string(self, it: str, c: bool):
		if c:
			self.write(f'"{it}"', False)
		else:
			self.write(f'stringView({len(it)}, "{it}")', False)

	def array(self, items: list[Callable]):
		if len(items) > 4: # Write on multiple lines
			self.writeln('{', False)
			self.depth += 1
			for item in items:
				item()
				self.writeln(',', False)
			self.write('}')
			self.depth -= 1
		else: # Write on one line
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

	def ref(self, id: str):
		self.write(f'(&{id})', False)

	def deref(self, value: Callable):
		self.write(f'(*', False)
		value()
		self.write(')', False)

	def cast(self, type: SeaType, value: Callable):
		self.write(f'(({self.type(type)})', False)
		value()
		self.write(')', False)

	def index(self, expr: Callable, index: Callable):
		expr()
		self.write('[', False)
		index()
		self.write(']', False)


	def raw(self, code: str):
		self.write(code.strip(' '))
		if self.depth == 0:
			self.write('\n', False)


	def comment(self, text: str):
		self.writeln('//' + text)

	def multiline_comment(self, text: str):
		self.writeln('/*' + text + '*/')
