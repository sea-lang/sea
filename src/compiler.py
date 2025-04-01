import sys
from typing import NamedTuple, NoReturn, Optional
from .syntax.Parser import Parser

class SeaType(NamedTuple):
	pointers: int # Amount of pointers
	name: str
	params: list[str]
	arrays: list[str|int]

	def copy(self) -> 'SeaType':
		return SeaType(self.pointers, self.name, self.params, self.arrays)

	# TODO: Apply templated array sizes
	def apply(self, template: 'SeaTemplateParams') -> 'SeaType':
		n = self.name
		if template.has_field(n):
			n = template.get(n)
		p = [it if not template.has_field(it) else template.get(it) for it in self.params]
		a = [it if not type(it) is str or not template.has_field(it) else template.get(it) for it in self.arrays]
		ret = SeaType(self.pointers, n, p, a)
		return ret

	@staticmethod
	def from_str(s: str, template: Optional['SeaTemplateParams'] = None) -> 'SeaType':
		if '{' not in s:
			name = s.split('[')[0].strip('^[]')
			if template is not None and template.has_field(name):
				name = template.get(name)

			array_strs = [p.strip('[]') for p in s.split('[')[1:]]
			arrays = []
			for i, a in enumerate(array_strs):
				if a == '': # Dynamically-sized arrays
					arrays.append(-1)
				elif a.isdigit(): # Fixed-size arrays
					arrays.append(int(a))
				elif a.isalnum(): # IDs, used for variable-sized arrays and templates
					arrays.append(a)

			return SeaType(s.count('^'), name, [], arrays)
		else:
			name = s[:s.find('{')].strip('^[]')

			if template is not None and template.has_field(name):
				name = template.get(name)
			tem = [it.strip() for it in s[s.find('{')+1:s.find('}')].split(',')]
			if template is not None:
				for i, t in enumerate(tem):
					if template.has_field(t):
						tem[i] = template.get(t)

			array_strs = [p.strip('[]') for p in s.split('[')[1:]]
			arrays = []
			for i, a in enumerate(array_strs):
				if a == '': # Dynamically-sized arrays
					arrays.append(-1)
				elif a.isdigit(): # Fixed-size arrays
					arrays.append(int(a))
				elif a.isalnum(): # IDs, used for variable-sized arrays and templates
					arrays.append(a)

			return SeaType(s.count('^'), name, tem, arrays)

SEA_VOID = SeaType(0, 'void', [], [])

class SeaFunction(NamedTuple):
	returns: SeaType
	params: dict[str, SeaType]

class SeaRecord(NamedTuple):
	fields: dict[str, SeaType]

class SeaVariable(NamedTuple):
	type: SeaType
	constant: bool
	depth: int

class SeaTemplate(NamedTuple):
	fields: dict[str, str]

class SeaTemplateParams(NamedTuple):
	template: SeaTemplate
	params: list[str]

	def has_field(self, name: str) -> bool:
		return name in self.template.fields.keys()

	def get(self, name: str) -> str:
		if self.has_field(name):
			return self.params[list(self.template.fields.keys()).index(name)]
		print(f'SeaTemplateParams#get: undefined field: {name}')
		exit(1)

class SeaRecordTemplate(NamedTuple):
	template: SeaTemplate
	record: SeaRecord

	def apply(self, params: SeaTemplateParams) -> SeaRecord:
		fields = self.record.fields.copy()
		for name in self.record.fields.keys():
			fields[name] = fields[name].apply(params)
		return SeaRecord(fields)

class SeaFunctionTemplate(NamedTuple):
	template: SeaTemplate
	function: SeaFunction
	code: Parser.Expr_blockContext

	def apply(self, params: SeaTemplateParams) -> SeaFunction:
		fun_returns = self.function.returns.apply(params)
		fun_params = self.function.params.copy()
		for name, type in fun_params.items():
			fun_params[name] = type.apply(params)
		print(fun_params)
		return SeaFunction(fun_returns, fun_params)

class Compiler:
	def __init__(self):
		self.functions: dict[str, SeaFunction] = {}
		self.records: dict[str, SeaRecord] = {}
		self.variables: dict[str, SeaVariable] = {}
		self.scope_depth: int = 0
		self.current_template: Optional[SeaTemplate] = None
		self.template_recs: dict[str, SeaRecordTemplate] = {}
		self.template_funs: dict[str, SeaFunctionTemplate] = {}

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

	def add_variable(self, name: str, type: SeaType, constant: bool = False):
		self.variables[name] = SeaVariable(type, constant, self.scope_depth)

	def add_record_template(self, name: str, record: SeaRecord, template: SeaTemplate):
		self.template_recs[name] = SeaRecordTemplate(template, record)

	def add_function_template(self, name: str, function: SeaFunction, code: Parser.Expr_blockContext, template: SeaTemplate):
		self.template_funs[name] = SeaFunctionTemplate(template, function, code)

	def find_type_of(self, variable: str) -> Optional[SeaType]:
		keys = list(self.variables.keys())
		# Iterate backwards so that we look at the most recent scope first
		for i in range(len(self.variables) - 1, -1, -1):
			if keys[i] == variable:
				return self.variables[keys[i]].type
		return None

	def find_template_by_id(self, id: str) -> Optional[SeaFunctionTemplate | SeaRecordTemplate]:
		if id in self.template_funs:
			return self.template_funs[id]
		elif id in self.template_recs:
			return self.template_recs[id]
		return None
