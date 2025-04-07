# Reef is a schemafull configuration format made for Sea
# It's intentionally kept very simplistic, only supporting key/value pairs.
# For more complex data structures, you should use another config format.

import re
from typing import Callable, NamedTuple, Optional, Union


class ReefParseResult(NamedTuple):
	success: bool
	message: str
	fields: dict[str, str]

class ReefSchemaValidator(NamedTuple):
	predicate: Callable[[str], bool]
	required: bool
	default: Optional[str]


class ReefSchema:
	def __init__(self):
		self.fields: dict[str, ReefSchemaValidator] = {}

	# Using Regex for this felt overkill, and I also didn't really want to deal with writing a RE.
	def _split_line(self, line: str) -> tuple[str, str]:
		key = None
		quote = None
		i = 0 # Define `i` here so that its value can be used outside the for loop
		for i, char in enumerate(line):
			if char == '"' or char == '\'':
				quote = None if quote == char else char
			elif char == '=' and quote is None and key is None:
				key = line[:i].strip()
				break
		else:
			return '', ''

		return key, line[i + 1:].strip()

	def parse(self, code: str) -> ReefParseResult:
		fields: dict[str, str] = {}

		for i, line in enumerate(code.splitlines()):
			s = line.strip()

			if s[0] == '#': # Comments
				continue

			key, value = self._split_line(s)
			if key == '': # Syntax error
				return ReefParseResult(False, f'syntax error [line {i+1}], unexpected `=`', {})

			validator = self.fields[key]
			if not validator.predicate(value):
				return ReefParseResult(False, f'validation failed: value for field `{key}` is invalid', {})

			fields[key] = value

		# Check for required fields and add defaults
		for field, validator in self.fields.items():
			if field not in fields:
				if validator.required:
					return ReefParseResult(False, f'validation failed: required field `{field}` was not present', {})
				elif validator.default is not None:
					fields[field] = validator.default

		return ReefParseResult(True, '', fields)

	def parse_file(self, file_path: str) -> ReefParseResult:
		with open(file_path, 'r') as f:
			return self.parse(f.read())

class ReefSchemaBuilder:
	def __init__(self):
		self.schema = ReefSchema()

	def field_string(self, key: str, required: bool = False, default: Optional[str] = None) -> 'ReefSchemaBuilder':
		self.schema.fields[key] = ReefSchemaValidator(
			lambda _: True,
			required,
			default
		)
		return self

	def field_int(self, key: str, required: bool = False, default: Optional[str] = None) -> 'ReefSchemaBuilder':
		self.schema.fields[key] = ReefSchemaValidator(
			lambda it: re.fullmatch('\\-?[0-9]+', it) is not None,
			required,
			default
		)
		return self

	def field_float(self, key: str, required: bool = False, default: Optional[str] = None) -> 'ReefSchemaBuilder':
		self.schema.fields[key] = ReefSchemaValidator(
			lambda it: re.fullmatch('\\-?[0-9]+(\\.[0-9]+)?', it) is not None,
			required,
			default
		)
		return self

	def field_enum(self, key: str, values: list[str], required: bool = False, default: Optional[str] = None) -> 'ReefSchemaBuilder':
		self.schema.fields[key] = ReefSchemaValidator(
			lambda it: it in values,
			required,
			default
		)
		return self

	def field_bool(self, key: str, required: bool = False, default: Optional[str] = None) -> 'ReefSchemaBuilder':
		self.schema.fields[key] = ReefSchemaValidator(
			lambda it: it in {'true', 'false'},
			required,
			default
		)
		return self

	def build(self) -> ReefSchema:
		return self.schema


def parse(code: str, schema: ReefSchema) -> ReefParseResult:
	return schema.parse(code)

def parse_file(file_path: str, schema: ReefSchema) -> ReefParseResult:
	with open(file_path, 'r') as f:
		return schema.parse(f.read())
