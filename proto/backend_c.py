from typing import Callable, Optional
from .backend import Backend
from .compiler import SEA_VOID, Compiler, HashTags, SeaFunction, SeaRecord, SeaTag, SeaType

class Backend_C(Backend):
	def __init__(self, compiler: Compiler, output_file: str):
		super().__init__(compiler, output_file)
		self.line_ending = ';\n'

	def type(self, type: SeaType) -> str:
		name = type.name
		if name == 'fun':
			print('error: function pointers must be named')
			exit(1)
		else:
			for param in type.params:
				name += '_' + param.replace('{', '_').replace('}', '')
			return name + ('*' * type.pointers) + ''.join(['[]' if it == -1 else f'[{it}]' for it in type.arrays])

	def typed_id(self, type: SeaType, id: str) -> str:
		name = type.name
		if name == 'fun':
			name = id
			for param in type.params:
				name += '_' + param.replace('{', '_').replace('}', '')
			return (
				self.type(type.funptr_rets or SEA_VOID) +
				'(*' + ('*' * type.pointers) + name + ') (' +
				', '.join([self.type(it) for it in type.funptr_args]) + ')' +
				''.join(['[]' if it == -1 else f'[{it}]' for it in type.arrays])
			)
		else:
			for param in type.params:
				name += '_' + param.replace('{', '_').replace('}', '')
			return name + ' ' + ('*' * type.pointers) + id + ''.join(['[]' if it == -1 else f'[{it}]' for it in type.arrays])


	def file_begin(self):
		if len(self.module_stack) > 0:
			self.writeln(f'#pragma region "mod: {self.module_stack[-1]}"')

	def file_end(self):
		if len(self.module_stack) > 0:
			self.writeln(f'#pragma endregion "mod: {self.module_stack[-1]}"')


	def rec(self, name: str, record: SeaRecord):
		self.compiler.add_record(name, record)

		if HashTags.Rec.UNION in record.hashtags:
			self.writeln('typedef union {')
		else:
			self.writeln(f'struct {name};')
			self.writeln(f'typedef struct {name} {name};')
			self.writeln(f'struct {name} {{')

		self.depth += 1
		for fieldname, typedesc in record.fields.items():
			self.write(self.typed_id(typedesc, fieldname))
			self.write(';\n', False)
		self.depth -= 1

		self.writeln('};\n') # newline for spacing

	def fun_begin(self, name: str, func: SeaFunction):
		for hashtag in func.hashtags:
			match hashtag:
				case HashTags.Fun.NORET:
					self.write('noreturn ', False)
				case HashTags.Fun.EXTERN: # TODO: anti-name-mangling
					print('extern unimplemented')
					exit(1)
				case HashTags.Fun.INLINE:
					self.write('inline ', False)
				case HashTags.Fun.STATIC:
					self.write('static ', False)

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
		self.writeln('', False) # writeln for spacing
		self.compiler.pop_scope()

	def def_(self, name: str, type: SeaType):
		self.writeln(f'typedef {self.typed_id(type, name)};\n')

	def tag(self, name: str, tag: SeaTag):
		prefix = ''

		for hashtag in tag.hashtags:
			match hashtag:
				case HashTags.Tag.STATIC:
					prefix += 'static '

		self.writeln(f'{prefix}enum {name};')
		self.writeln(f'{prefix}typedef enum {name} {name};')
		self.writeln(f'{prefix}enum {name} {{')
		self.depth += 1
		for field, maybe_value in tag.fields.items():
			if maybe_value is not None:
				self.writeln(f'{field} = {maybe_value},')
			else:
				self.writeln(field + ',')
		self.depth -= 1
		self.writeln('};')

		self.compiler.add_tag(name, tag)

	def tagrec_tag(self, name: str, fields: list[str]):
		self.tag(name + '$Kind', SeaTag({key:None for key in fields}, []))
		self.writeln('', False)

	def tagrec_rec(self, name: str, fields: dict[str, SeaRecord], kind_name: Optional[str] = None, hashtags: Optional[list[HashTags.Rec]] = None):
		rec = SeaRecord({name:field.get_type(name) for name, field in fields.items()}, hashtags or [])
		self.compiler.add_record(name, rec)

		prefix = ''

		for hashtag in rec.hashtags:
			match hashtag:
				case HashTags.Rec.STATIC:
					prefix += 'static '

		self.writeln(f'{prefix}struct {name};')
		self.writeln(f'{prefix}typedef struct {name} {name};')
		self.writeln(f'{prefix}struct {name} {{')
		self.depth += 1
		kind_name = kind_name if kind_name is not None else (name + '$Kind')
		self.writeln(f'{kind_name} kind;')
		self.writeln('union {')
		self.depth += 1
		for rec_name, rec in fields.items():
			if len(rec.fields) == 0:
				continue # Skip empty fields

			self.write('struct { ')
			for fieldname, typedesc in rec.fields.items():
				self.write(self.typed_id(typedesc, fieldname) + '; ', False)
			self.writeln(f'}} {rec_name};', False)
		self.depth -= 1
		self.writeln('};')
		self.depth -= 1
		self.writeln('};\n')

		# Write helper functions
		for rec_name, rec in fields.items():
			self.write(f'{prefix}{name} {name}_new{rec_name}(')
			self.write(', '.join([self.typed_id(typedesc, fieldname) for fieldname, typedesc in rec.fields.items()]), False)
			self.writeln(')', False)
			self.writeln('{')
			self.depth += 1
			if len(rec.fields) > 0:
				self.write(f'return ({name}){{{rec_name}, .{rec_name}={{')
				self.write(', '.join([fieldname for fieldname in rec.fields.keys()]), False)
				self.writeln('}};', False)
			else:
				self.writeln(f'return ({name}){{{rec_name}}};')
			self.depth -= 1
			self.writeln('}\n')

	def tagrec(self, name: str, fields: dict[str, SeaRecord]):
		self.tagrec_tag(name, list(fields.keys()))
		self.tagrec_rec(name, fields)


	def var(self, name: str, type: SeaType, value: Callable, is_top_level: bool = False):
		self.write(self.typed_id(type, name), False)
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type)

	def let(self, name: str, type: SeaType, value: Callable, is_top_level: bool = False):
		self.write('const ', False)
		self.write(self.typed_id(type, name), False)
		self.write(' = ', False)
		value()
		self.compiler.add_variable(name, type, True)

	def assign(self, name: Callable, value: Callable):
		name()
		self.write(' = ', False)
		value()

	def ret(self, value: Optional[Callable]):
		if value is None:
			self.write('return')
		else:
			self.write('return ')
			value()


	def block_begin(self):
		self.writeln('{', False)
		self.depth += 1

	def block_end(self):
		self.depth -= 1
		self.writeln('}')

	def invoke(self, id: str, args: list[Callable]):
		self.write(id + '(', False)
		for arg in args:
			arg()
			if arg != args[-1]:
				self.write(', ', False)
		self.write(')', False)

	def if_(self, cond: Callable):
		self.write('if (')
		cond()
		self.write(') ', False)

	def else_(self):
		self.write('else ')

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
		self.write(f'for (int _to = ')
		to()
		self.write(f', {var} = ', False)
		from_()
		self.write(f'; {var} < _to; {var}++) ', False)

	# TODO: Use a trait for this
	def each_begin(self, var: str, of: str):
		typ = self.compiler.find_type_of(of)
		if typ == None:
			self.compiler.panic(f'no such variable: {of}')
		typ = SeaType(typ.pointers, typ.name, typ.params, typ.arrays, typ.funptr_args, typ.funptr_rets)

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

	def switch_begin(self, of: Callable):
		self.write('switch (')
		of()
		self.writeln(') {', False)
		self.depth += 1

	def switch_end(self):
		self.depth -= 1
		self.writeln('}')

	def case(self, match: Callable):
		self.write('case ')
		match()
		self.write(': ', False)

	def case_else(self):
		self.write('default: ')

	def case_break(self):
		self.writeln('break;')


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
		value()
		self.write(op, False)

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
		self.write(it.replace('_', ''), False)

	def id(self, it: str):
		self.write(it, False)

	def string(self, it: str, c: bool):
		if c:
			self.write(f'"{it}"', False)
		else:
			self.write(f'(String){{false, {len(it)}, "{it}"}}', False)

	def char(self, it: str):
		# \` is an invalid escape sequence in C, so we'll need some special handling for when `\`` is used:
		if it == '\\`':
			self.write(f'\'`\'', False)
		else:
			self.write(f'\'{it}\'', False)

	def array(self, type: Optional[SeaType], items: list[Callable]):
		if len(items) > 4: # Write on multiple lines
			if type is None:
				self.writeln('{', False)
			else:
				self.writeln(f'({self.type(type)}){{', False)

			self.depth += 1
			for item in items:
				self.force_indent_next = True
				item()
				self.writeln(',', False)
			self.depth -= 1
			self.write('}')
		else: # Write on one line
			if type is None:
				self.write('{', False)
			else:
				self.write(f'({self.type(type)}){{', False)

			for item in items:
				item()
				self.write(',', False)
			self.write('}', False)

	def new(self, rec: str, items: list[Callable]):
		if len(items) > 4: # Write on multiple lines
			self.writeln(f'({rec}){{', False)
			self.depth += 1
			for item in items:
				self.force_indent_next = True
				item()
				self.writeln(',', False)
			self.depth -= 1
			self.write('}')
		else: # Write on one line
			self.write(f'({rec}){{', False)
			for item in items:
				item()
				self.write(', ', False)
			self.write('}', False)

	def ref(self, value: Callable):
		self.write(f'(&', False)
		value()
		self.write(')', False)

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
