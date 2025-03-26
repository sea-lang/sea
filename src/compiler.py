import sys
from typing import NamedTuple, NoReturn, Optional

class SeaType(NamedTuple):
	pointers: int # Amount of pointers
	name: str
	arrays: int # Amount of arrays

class SeaFunction(NamedTuple):
	returns: SeaType
	params: dict[str, SeaType]

class SeaRecord(NamedTuple):
	fields: dict[str, SeaType]

class SeaVariable(NamedTuple):
	type: SeaType
	depth: int

class Compiler:
	def __init__(self):
		self.functions: dict[str, SeaFunction] = {}
		self.records: dict[str, SeaRecord] = {}
		self.variables: dict[str, SeaVariable] = {}
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

	def add_variable(self, name: str, type: SeaType):
		self.variables[name] = SeaVariable(type, self.scope_depth)

	def find_type_of(self, variable: str) -> Optional[SeaType]:
		keys = list(self.variables.keys())
		for i in range(len(self.variables) - 1, 0, -1):
			if keys[i] == variable:
				return self.variables[keys[i]].type
		return None
