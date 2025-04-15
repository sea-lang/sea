from enum import Enum
import sys
from typing import NamedTuple, NoReturn, Optional
from .syntax.Parser import Parser


class HashTags:
	class Fun(Enum):
		NORET = 0
		INLINE = 1
		EXTERN = 2
		STATIC = 3

	class Rec(Enum):
		UNION = 0
		STATIC = 1

	class Tag(Enum):
		STATIC = 0

	class TagRec(Enum):
		STATIC = 0

	class Mac(Enum):
		STATIC = 0

class SeaType(NamedTuple):
	pointers: int # Amount of pointers
	name: str
	arrays: list[str|int]
	funptr_args: list['SeaType']
	funptr_rets: Optional['SeaType']

	def copy(self) -> 'SeaType':
		return SeaType(self.pointers, self.name, self.arrays, self.funptr_args, self.funptr_rets)

	def copy_with(
		self,
		pointers: Optional[int] = None,
		name: Optional[str] = None,
		arrays: Optional[list[str|int]] = None,
		funptr_args: Optional[list['SeaType']] = None,
		funptr_rets: Optional[Optional['SeaType']] = None, # this one may need special handling, idk
	) -> 'SeaType':
		return SeaType(
			self.pointers if pointers is None else pointers,
			self.name if name is None else name,
			self.arrays if arrays is None else arrays,
			self.funptr_args if funptr_args is None else funptr_args,
			self.funptr_rets if funptr_rets is None else funptr_rets
		)

	# Converts the type to a string for name-mangling
	def to_mangling_str(self) -> str:
		return ('p' * self.pointers) + self.name + (''.join([(f'a{count}' if count != -1 else 'a') for count in self.arrays]))

	@staticmethod
	def from_str(s: str) -> 'SeaType':
		name = s.split('[')[0].strip('^[]')
		funptr_args = []
		funptr_rets = None

		if '(' in s and s.split('(')[0].strip('^') == 'fun':
			args = [it.strip() for it in s[s.find('(')+1:s.rfind(')')].split(',') if it.strip() != '']
			if len(args) > 0:
				funptr_args = [SeaType.from_str(it.strip()) for it in args]

			if ':' in s.split(')')[-1]:
				funptr_rets = SeaType.from_str(s.split(':')[-1].strip())
			else:
				funptr_rets = SEA_VOID

			name = 'fun'

		array_strs = [p.strip('[]') for p in s.split('[')[1:]]
		arrays = []
		for i, a in enumerate(array_strs):
			if a == '': # Dynamically-sized arrays
				arrays.append(-1)
			elif a.isdigit(): # Fixed-size arrays
				arrays.append(int(a))
			elif a.isalnum(): # IDs, used for variable-sized arrays and templates
				arrays.append(a)

		return SeaType(s.count('^'), name, arrays, funptr_args, funptr_rets)

SEA_VOID = SeaType(0, 'void', [], [], None)
SEA_INT = SeaType(0, 'int', [], [], None)
SEA_FLOAT = SeaType(0, 'float', [], [], None)
SEA_DOUBLE = SeaType(0, 'double', [], [], None)
SEA_BOOL = SeaType(0, 'bool', [], [], None)
SEA_CSTRING = SeaType(1, 'char', [], [], None)
SEA_STRING = SeaType(0, 'String', [], [], None)

class SeaFunction(NamedTuple):
	returns: SeaType
	params: dict[str, SeaType]
	hashtags: list[HashTags.Fun]

	def get_type(self, name: str) -> SeaType:
		return SeaType(1, name, [], list(self.params.values()), self.returns)

class SeaRecord(NamedTuple):
	fields: dict[str, SeaType]
	hashtags: list[HashTags.Rec]

	def get_type(self, name: str) -> SeaType:
		return SeaType(0, name, [], [], None)

class SeaTag(NamedTuple):
	fields: dict[str, Optional[int]]
	hashtags: list[HashTags.Tag]

class SeaTagRec(NamedTuple):
	kind_id: str
	fields: dict[str, SeaRecord]
	hashtags: list[HashTags.TagRec]

class SeaMacro(NamedTuple):
	params: list[str]
	hashtags: list[HashTags.Mac]

class SeaVariable(NamedTuple):
	type: SeaType
	constant: bool
	depth: int

class Compiler:
	def __init__(self):
		self.functions: dict[str, SeaFunction] = {}
		self.records: dict[str, SeaRecord] = {}
		self.variables: dict[str, SeaVariable] = {}
		self.tags: dict[str, SeaTag] = {}
		self.tag_values_to_tag_name: dict[str, str] = {}
		self.macros: dict[str, SeaMacro] = {}

		self.scope_depth: int = 0

	def panic(self, message: str) -> NoReturn:
		print('compiler error: ' + message, file = sys.stderr)
		exit(1)

	def push_scope(self):
		self.scope_depth += 1

	def pop_scope(self):
		self.scope_depth -= 1
		self.variables = {k:v for k,v in self.variables.items() if v.depth < self.scope_depth}

	def add_function(self, name: str, func: SeaFunction):
		if name in self.functions:
			self.panic(f'function {name} already exists.')
		self.functions[name] = func

	def add_record(self, name: str, record: SeaRecord):
		if name in self.records:
			self.panic(f'record {name} already exists.')
		self.records[name] = record

	def add_tag(self, name: str, tag: SeaTag):
		if name in self.tags:
			self.panic(f'tag {name} already exists.')
		self.tags[name] = tag
		for key in tag.fields.keys():
			if key in self.tag_values_to_tag_name:
				self.panic(f'duplicate tag entry: {name}')
			self.tag_values_to_tag_name[key] = name

	def add_variable(self, name: str, type: SeaType, constant: bool = False):
		self.variables[name] = SeaVariable(type, constant, self.scope_depth)

	def find_type_of(self, variable: str) -> Optional[SeaType]:
		keys = list(self.variables.keys())
		# Iterate backwards so that we look at the most recent scope first
		for i in range(len(self.variables) - 1, -1, -1):
			if keys[i] == variable:
				return self.variables[keys[i]].type
		return None
