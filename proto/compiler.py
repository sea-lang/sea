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

class SeaType(NamedTuple):
	pointers: int # Amount of pointers
	name: str
	params: list[str]
	arrays: list[str|int]
	funptr_args: list['SeaType']
	funptr_rets: Optional['SeaType']

	def copy(self) -> 'SeaType':
		return SeaType(self.pointers, self.name, self.params, self.arrays, self.funptr_args, self.funptr_rets)

	def copy_with(
		self,
		pointers: Optional[int] = None,
		name: Optional[str] = None,
		params: Optional[list[str]] = None,
		arrays: Optional[list[str|int]] = None,
		funptr_args: Optional[list['SeaType']] = None,
		funptr_rets: Optional[Optional['SeaType']] = None, # this one may need special handling, idk
	) -> 'SeaType':
		return SeaType(
			self.pointers if pointers is None else pointers,
			self.name if name is None else name,
			self.params if params is None else params,
			self.arrays if arrays is None else arrays,
			self.funptr_args if funptr_args is None else funptr_args,
			self.funptr_rets if funptr_rets is None else funptr_rets
		)

	# TODO: Apply templated array sizes
	def apply(self, template: 'SeaTemplateParams') -> 'SeaType':
		return SeaType(
			self.pointers,
			self.name if not template.has_field(self.name) else template.get(self.name),
			[it if not template.has_field(it) else template.get(it) for it in self.params],
			[it if not type(it) is str or not template.has_field(it) else template.get(it) for it in self.arrays],
			[it.apply(template) for it in self.funptr_args],
			None if self.funptr_rets is None else self.funptr_rets.apply(template)
		)

	# Converts the type to a string for name-mangling
	def to_mangling_str(self) -> str:
		return ('p' * self.pointers) + self.name + (''.join(f'_{it}' for it in self.params)) + (''.join([(f'a{count}' if count != -1 else 'a') for count in self.arrays]))

	@staticmethod
	def from_str(s: str, template: Optional['SeaTemplateParams'] = None) -> 'SeaType':
		if '{' not in s:
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
			elif template is not None and template.has_field(name):
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

			return SeaType(s.count('^'), name, [], arrays, funptr_args, funptr_rets)
		else:
			name = s[:s.find('{')].strip('^[]')
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
			elif template is not None and template.has_field(name):
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

			return SeaType(s.count('^'), name, tem, arrays, funptr_args, funptr_rets)

SEA_VOID = SeaType(0, 'void', [], [], [], None)
SEA_INT = SeaType(0, 'int', [], [], [], None)
SEA_FLOAT = SeaType(0, 'float', [], [], [], None)
SEA_DOUBLE = SeaType(0, 'double', [], [], [], None)
SEA_BOOL = SeaType(0, 'bool', [], [], [], None)
SEA_CSTRING = SeaType(1, 'char', [], [], [], None)
SEA_STRING = SeaType(0, 'String', [], [], [], None)

class SeaFunction(NamedTuple):
	returns: SeaType
	params: dict[str, SeaType]
	hashtags: list[HashTags.Fun]

	def get_type(self, name: str) -> SeaType:
		return SeaType(1, name, [], [], list(self.params.values()), self.returns)

class SeaRecord(NamedTuple):
	fields: dict[str, SeaType]
	hashtags: list[HashTags.Rec]

	def get_type(self, name: str) -> SeaType:
		return SeaType(0, name, [], [], [], None)

class SeaTag(NamedTuple):
	fields: dict[str, Optional[int]]
	hashtags: list[HashTags.Tag]

class SeaTagRec(NamedTuple):
	kind_id: str
	fields: dict[str, SeaRecord]
	hashtags: list[HashTags.TagRec]

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
		return SeaRecord(fields, self.record.hashtags)

class SeaFunctionTemplate(NamedTuple):
	template: SeaTemplate
	function: SeaFunction
	code: Parser.Expr_blockContext

	def apply(self, params: SeaTemplateParams) -> SeaFunction:
		fun_returns = self.function.returns.apply(params)
		fun_params = self.function.params.copy()
		for name, type in fun_params.items():
			fun_params[name] = type.apply(params)
		return SeaFunction(fun_returns, fun_params, self.function.hashtags)

class SeaTagRecTemplate(NamedTuple):
	template: SeaTemplate
	tagrec: SeaTagRec

	def apply(self, params: SeaTemplateParams) -> SeaTagRec:
		fields: dict[str, SeaRecord] = {}
		for rec_name, rec in self.tagrec.fields.items():
			rec_fields: dict[str, SeaType] = {}
			for field_name, rec_field in rec.fields.items():
				rec_fields[field_name] = rec_field.apply(params)
			fields[rec_name] = SeaRecord(rec_fields, rec.hashtags)
		return SeaTagRec(self.tagrec.kind_id, fields, self.tagrec.hashtags)

AnyTemplate = SeaRecordTemplate | SeaFunctionTemplate | SeaTagRecTemplate

class SeaTemplateGroup(NamedTuple):
	template: SeaTemplate
	templates: dict[str, AnyTemplate]

class Compiler:
	def __init__(self):
		self.functions: dict[str, SeaFunction] = {}
		self.records: dict[str, SeaRecord] = {}
		self.variables: dict[str, SeaVariable] = {}
		self.tags: dict[str, SeaTag] = {}
		self.tag_values_to_tag_name: dict[str, str] = {}
		self.scope_depth: int = 0
		self.current_template: Optional[SeaTemplate] = None
		self.template_recs: dict[str, SeaRecordTemplate] = {}
		self.template_funs: dict[str, SeaFunctionTemplate] = {}
		self.template_tagrecs: dict[str, SeaTagRecTemplate] = {}
		self.template_groups: list[tuple[set[str], SeaTemplateGroup]] = []

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

	def add_record_template(self, name: str, template: SeaRecordTemplate):
		self.template_recs[name] = template

	def add_function_template(self, name: str, template: SeaFunctionTemplate):
		self.template_funs[name] = template

	def add_tagrec_template(self, name: str, template: SeaTagRecTemplate):
		self.template_tagrecs[name] = template

	def add_template_group(self, templates: dict[str, AnyTemplate], template: SeaTemplate):
		self.template_groups.append((set(templates.keys()), SeaTemplateGroup(template, templates)))

	def find_type_of(self, variable: str) -> Optional[SeaType]:
		keys = list(self.variables.keys())
		# Iterate backwards so that we look at the most recent scope first
		for i in range(len(self.variables) - 1, -1, -1):
			if keys[i] == variable:
				return self.variables[keys[i]].type
		return None

	def find_template_by_id(self, id: str) -> Optional[AnyTemplate | SeaTemplateGroup]:
		if id in self.template_funs:
			return self.template_funs[id]
		elif id in self.template_recs:
			return self.template_recs[id]
		elif id in self.template_tagrecs:
			return self.template_tagrecs[id]
		else:
			for ids, group in self.template_groups:
				if id in ids:
					return group
		return None
