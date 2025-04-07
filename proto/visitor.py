import os
from pathlib import Path
from typing import Optional
import antlr4
from antlr4.tree.Tree import TerminalNodeImpl
from antlr4.error.ErrorListener import ErrorListener

from .infer import infer_type
from .backend import Backend
from .compiler import SEA_VOID, AnyTemplate, HashTags, SeaFunction, SeaFunctionTemplate, SeaRecord, SeaRecordTemplate, SeaTag, SeaTagRec, SeaTagRecTemplate, SeaTemplate, SeaTemplateGroup, SeaTemplateParams, SeaType
from .syntax.Lexer import Lexer
from .syntax.Parser import Parser
from .syntax.ParserListener import ParserListener

class Visitor(ParserListener):
	ALLOWED_TEMPLATE_TYPES = ['type', 'int', 'char', 'bool', 'float', 'double']

	def __init__(self, backend: Backend, nostd: bool = False):
		self.backend = backend
		self.nostd = nostd
		self.skipping = False
		self.template: Optional[SeaTemplateParams] = None
		self.add_implicit_break_statement = False

	def _writer(self, expr, index: Optional[int] = None):
		if index is None:
			return lambda: self.write_expr(expr)
		else:
			return lambda: self.write_expr(expr.getChild(index))

	def _should_skip(self) -> bool:
		return self.skipping

	def _get_all(self, to_find) -> list:
		ret = []
		i = 0
		it = to_find(i)
		while it is not None:
			ret.append(it)
			i += 1
			it = to_find(i)
		return ret

	def _get_fun(self, ctx: Parser.FunContext) -> SeaFunction:
		hashtags = [] if ctx.hashtag() is None else [HashTags.Fun[id.symbol.text.upper()] for id in self._get_all(ctx.hashtag().ID)]

		params = {}
		for param in self._get_all(ctx.part_params().part_param):
			params[param.ID().symbol.text] = SeaType.from_str(param.typedesc().getText())

		if ctx.COLON() is not None:
			returns = SeaType.from_str(ctx.typedesc().getText())
		else:
			returns = SEA_VOID

		return SeaFunction(returns, params, hashtags)

	def _get_rec(self, ctx: Parser.RecContext) -> SeaRecord:
		hashtags = [] if ctx.hashtag() is None else [HashTags.Rec[id.symbol.text.upper()] for id in self._get_all(ctx.hashtag().ID)]

		fields = {}
		for param in self._get_all(ctx.part_params().part_param):
			fields[param.ID().symbol.text] = SeaType.from_str(param.typedesc().getText())

		return SeaRecord(fields, hashtags)

	def _get_tagrec_fields(self, ctx: Parser.TagrecContext) -> dict[str, SeaRecord]:
		fields: dict[str, SeaRecord] = {}
		entries = self._get_all(ctx.tagrec_entry)
		for entry in entries:
			params = {}
			part = entry.part_params()
			for param in self._get_all(part.part_param):
				params[param.ID().symbol.text] = SeaType.from_str(param.typedesc().getText())
			fields[entry.ID().symbol.text] = SeaRecord(params, [])

		return fields

	def _use_if_exists(self, module: str, path: str) -> bool:
		if module in self.backend.using:
			return True # Already imported
		if not os.path.exists(path):
			return False
		self.backend.using.append(module)
		self.backend.use(module)
		visit(path, backend = self.backend)
		return True

	def _write_applied_template(self, id: str, applied, original_template: AnyTemplate, params: list[str]):
		if isinstance(applied, SeaFunction):
			name = id + ''.join([f'_{p}' for p in params])
			self.backend.fun_begin(name, applied)
			self.backend.block_begin()

			assert isinstance(original_template, SeaFunctionTemplate)
			end_offset = 1 if original_template.code.LCURLY() is not None else 0 # Check whether we are a {} or a ->
			for i in range(1, original_template.code.getChildCount() - end_offset):
				it = original_template.code.getChild(i)
				walker = antlr4.ParseTreeWalker()
				walker.walk(self, it)

			self.backend.block_end()
			self.backend.fun_end()
		elif isinstance(applied, SeaRecord):
			name = id + ''.join([f'_{p}' for p in params])
			self.backend.rec(name, applied)
		elif isinstance(applied, SeaTagRec):
			name = id + ''.join([f'_{p}' for p in params])
			self.backend.tagrec_rec(name, applied.fields, applied.kind_id)
		else:
			print(f'internal error: unhandled template application for `{type(applied)}`\n\tthis error should never happen; please report it!')
			exit(1)

	def enterProgram(self, ctx: Parser.ProgramContext):
		self.backend.file_begin()
		if not self.nostd:
			searched = []
			for path in self.backend.libpaths:
				p = path + '/std/lib.sea'
				searched.append(p)
				if self._use_if_exists('std', p):
					break
			else:
				print('error: failed to locate stdlib.')
				for path in searched:
					# Replace home dir with ~ to censor username
					print('  - ' + path.replace(str(Path.home()), '~'))
				exit(1)

	def exitProgram(self, ctx: Parser.ProgramContext):
		self.backend.file_end()

	def enterUse(self, ctx: Parser.UseContext):
		if self._should_skip(): return

		module = ctx.part_path().getText()
		# We can assume that if module is already imported, module_lib is also already imported, so we won't even check for it here.
		if module in self.backend.using:
			return

		searched = []
		for path in self.backend.libpaths:
			# Recursively import for lib.sea files
			for mod in reversed(module.split('/')):
				self._use_if_exists(mod, path + '/' + mod + '/lib.sea')

			module_path = path + '/' + module

			searched.append(module_path + '.sea')
			if self._use_if_exists(module, module_path + '.sea'):
				return

			searched.append(module_path + '/lib.sea')
			if self._use_if_exists(module, module_path + '/lib.sea'):
				return

		print(f'error: module `{module}` does not exists, searched for:')
		for path in searched:
			# Replace home dir with ~ to censor username
			print('  - ' + path.replace(str(Path.home()), '~'))
		exit(1)

	def write_expr(self, expr: Parser.ExprContext):
		if self._should_skip(): return

		def _writer(index: int):
			return lambda: self.write_expr(expr.getChild(index))

		# TODO: Organize
		if expr.expr() is not None and expr.getChild(0).getText() == '(':
			self.backend.group_expr(_writer(1))
		elif expr.part_invoke() is not None:
			e = expr.part_invoke()
			items = []
			for i in range(1, e.getChildCount() - 1):
				item = e.children[i]
				if isinstance(item, TerminalNodeImpl):
					continue
				items.append(self._writer(e, i))

			name = expr.ID().symbol.text
			if expr.template_descriptor() is not None:
				tem = expr.template_descriptor()
				children = tem.getChildren(predicate = lambda it: isinstance(it, Parser.Template_descriptor_valueContext))
				for child in children:
					name += '_' + child.getText()
			self.backend.invoke(name, items)
		elif expr.expr_ref() is not None:
			self.backend.ref(lambda: self.write_expr(expr.expr_ref().expr()))
		elif expr.PTR() is not None and expr.expr() is not None:
			self.backend.deref(lambda: self.write_expr(expr.getChild(0)))
		elif expr.AS() is not None:
			self.backend.cast(SeaType.from_str(expr.typedesc().getText(), template = self.template), lambda: self.write_expr(expr.getChild(0)))
		elif expr.part_index() is not None:
			e = expr.part_index()
			self.backend.index(self._writer(expr, 0), self._writer(e, 1))
		# Operators
		elif expr.OP_DOT() is not None: self.backend.dot(_writer(0), _writer(2))
		elif expr.OP_NOT() is not None: self.backend.not_(_writer(1))
		elif expr.OP_AND() is not None: self.backend.and_(_writer(0), _writer(2))
		elif expr.OP_OR() is not None: self.backend.or_(_writer(0), _writer(2))
		elif expr.OP_EQ() is not None: self.backend.eq(_writer(0), _writer(2))
		elif expr.OP_NEQ() is not None: self.backend.neq(_writer(0), _writer(2))
		elif expr.OP_GT() is not None: self.backend.gt(_writer(0), _writer(2))
		elif expr.OP_GTEQ() is not None: self.backend.gteq(_writer(0), _writer(2))
		elif expr.OP_LT() is not None: self.backend.lt(_writer(0), _writer(2))
		elif expr.OP_LTEQ() is not None: self.backend.lteq(_writer(0), _writer(2))
		elif expr.OP_INC() is not None: self.backend.inc(_writer(0))
		elif expr.OP_DEC() is not None: self.backend.dec(_writer(0))
		# Math
		elif expr.ADD() is not None: self.backend.add(_writer(0), _writer(2))
		elif expr.SUB() is not None: self.backend.sub(_writer(0), _writer(2))
		elif expr.MUL() is not None: self.backend.mul(_writer(0), _writer(2))
		elif expr.DIV() is not None: self.backend.div(_writer(0), _writer(2))
		elif expr.MOD() is not None: self.backend.mod(_writer(0), _writer(2))
		# Literals
		elif expr.number() is not None:
			self.backend.number(expr.number().getText())
		elif expr.STRING() is not None:
			text = expr.STRING().getText()
			if text[0] == 'c':
				self.backend.string(text[2:-1], True)
			else:
				self.backend.string(text[1:-1], False)
		elif expr.CHAR() is not None:
			self.backend.char(expr.CHAR().getText()[1:-1])
		elif expr.TRUE() is not None:
			self.backend.true()
		elif expr.FALSE() is not None:
			self.backend.false()
		elif expr.ID() is not None:
			self.backend.id(expr.ID().getText())
		elif expr.expr_list() is not None:
			e = expr.expr_list()
			items = []
			for i in range(1, e.getChildCount() - 1):
				item = e.children[i]
				if isinstance(item, TerminalNodeImpl):
					continue
				items.append(self._writer(e, i))

			type = None if len(items) == 0 else infer_type(self.backend.compiler, expr, self.template)

			self.backend.array(type, items)
		elif expr.expr_new() is not None:
			e = expr.expr_new()
			name = e.ID().symbol.text

			start = 1
			if e.template_descriptor() is not None:
				tem = e.template_descriptor()
				i = 0
				it = tem.template_descriptor_value(i)
				params = []
				while it is not None:
					params.append(it.getText())
					i += 1
					it = tem.template_descriptor_value(i)
				# Ready to see a cursed one-liner?
				name += ''.join('_' + (p if self.template is None or not self.template.has_field(p) else self.template.get(p)) for p in params)
				start = 3

			items = []
			for i in range(start, e.getChildCount() - 1):
				item = e.children[i]
				if isinstance(item, TerminalNodeImpl):
					continue
				items.append(self._writer(e, i))

			self.backend.new(name, items)
		elif expr.expr_var() is not None:
			e = expr.expr_var()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr(), self.template)
			else:
				typ = SeaType.from_str(e.typedesc().getText(), template = self.template)
			self.backend.var(name, typ, self._writer(e.expr()))
		elif expr.expr_let() is not None:
			e = expr.expr_let()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr(), self.template)
			else:
				typ = SeaType.from_str(e.typedesc().getText(), template = self.template)
			self.backend.let(name, typ, self._writer(e.expr()))
		elif expr.EQ() is not None:
			self.backend.assign(self._writer(expr, 0), self._writer(expr, 2))

	def enterTop_level_stat(self, ctx: Parser.Top_level_statContext):
		if self._should_skip(): return

		if ctx.expr_var() is not None:
			e = ctx.expr_var()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr(), self.template)
			else:
				typ = SeaType.from_str(e.typedesc().getText(), template = self.template)
			self.backend.var(name, typ, lambda: self.write_expr(e.expr()), is_top_level = True)
			self.backend.write(self.backend.line_ending, False)
		elif ctx.expr_let() is not None:
			e = ctx.expr_let()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr(), self.template)
			else:
				typ = SeaType.from_str(e.typedesc().getText(), template = self.template)
			self.backend.let(name, typ, lambda: self.write_expr(e.expr()), is_top_level = True)
			self.backend.write(self.backend.line_ending, False)

	def enterStat(self, ctx: Parser.StatContext):
		if self._should_skip(): return

		if ctx.expr() is not None:
			self.backend.force_indent_next = True
			self.write_expr(ctx.expr())

	def exitStat(self, ctx: Parser.StatContext):
		if self._should_skip(): return

		if self.backend.needs_line_ending(ctx):
			self.backend.write(self.backend.line_ending, False)
		else:
			self.backend.writeln('', False)

	def enterStat_ret(self, ctx: Parser.Stat_retContext):
		if self._should_skip(): return
		self.backend.ret(None if ctx.expr() is None else self._writer(ctx.expr()))

	def enterFun(self, ctx: Parser.FunContext):
		if self._should_skip(): return
		self.backend.fun_begin(ctx.ID().symbol.text, self._get_fun(ctx))

	def exitFun(self, ctx: Parser.FunContext):
		if self._should_skip(): return
		self.backend.fun_end()
		if ctx.expr_block() is None:
			self.backend.write(self.backend.line_ending)

	def enterRaw_block(self, ctx: Parser.Raw_blockContext):
		if self._should_skip(): return
		self.backend.raw(ctx.getChild(1).getText())

	def enterRec(self, ctx: Parser.RecContext):
		if self._should_skip(): return
		self.backend.rec(ctx.ID().symbol.text, self._get_rec(ctx))

	def enterDef(self, ctx: Parser.DefContext):
		if self._should_skip(): return
		self.backend.def_(ctx.ID().symbol.text, SeaType.from_str(ctx.typedesc().getText()))

	def enterTem(self, ctx: Parser.TemContext):
		if self._should_skip(): return

		if self.backend.compiler.current_template is not None:
			print('error: templates cannot be nested')
			exit(1)

		self.skipping = True

		fields = {}
		i = 0
		it = ctx.template_def_param(i)
		while it is not None:
			if not it.ID(1).symbol.text in self.ALLOWED_TEMPLATE_TYPES:
				print(f'error: templates may only accept parameters of types: {self.ALLOWED_TEMPLATE_TYPES}')
				exit(1)
			fields[it.ID(0).symbol.text] = it.ID(1).symbol.text
			i += 1
			it = ctx.template_def_param(i)

		template = SeaTemplate(fields)
		self.backend.compiler.current_template = template

		is_group = ctx.top_level_stat(1) is not None # Check if there are at least 2 things in teh template, if so then we are a group
		templates: dict[str, AnyTemplate] = {}

		# Add templates
		i = 0
		it = ctx.top_level_stat(i)
		while it is not None:
			if it.fun() is not None:
				fun = it.fun()
				block = fun.expr_block()
				if block is None:
					print('error: cannot forward-declare functions in a template')
					exit(1)
				tem = SeaFunctionTemplate(template, self._get_fun(fun), block)
				if is_group:
					templates[fun.ID().symbol.text] = tem
				else:
					self.backend.compiler.add_function_template(fun.ID().symbol.text, tem)
			elif it.rec() is not None:
				rec = it.rec()
				tem = SeaRecordTemplate(template, self._get_rec(rec))
				if is_group:
					templates[rec.ID().symbol.text] = tem
				else:
					self.backend.compiler.add_record_template(rec.ID().symbol.text, tem)
			elif it.tagrec() is not None:
				tagrec = it.tagrec()
				fields = self._get_tagrec_fields(tagrec)
				hashtags = [] if tagrec.hashtag() is None else [HashTags.TagRec[id.symbol.text.upper()] for id in self._get_all(tagrec.hashtag().ID)]
				tem = SeaTagRecTemplate(template, SeaTagRec(tagrec.ID().symbol.text + '$Kind', fields, hashtags))

				# We'll write the enum (since those aren't template-able) now and save the records as a template
				tag_name = tagrec.ID().symbol.text
				self.backend.tagrec_tag(tag_name, list(fields.keys()))

				if is_group:
					templates[tagrec.ID().symbol.text] = tem
				else:
					self.backend.compiler.add_tagrec_template(tagrec.ID().symbol.text, tem)
			else:
				print('error: templates can only contain functions and records, got: ' + str(type(it).__name__))
				exit(1)
			i += 1
			it = ctx.top_level_stat(i)

		if is_group:
			self.backend.compiler.add_template_group(templates, template)

		self.backend.compiler.current_template = None

	def exitTem(self, ctx: Parser.TemContext):
		self.skipping = False

	def enterGen(self, ctx: Parser.GenContext):
		if self._should_skip(): return

		id = ctx.ID().symbol.text
		template = self.backend.compiler.find_template_by_id(id)

		if template is None:
			print(f'error: no such template: {id}')
			exit(1)

		t = ctx.template_descriptor()
		params: list[str] = []
		i = 0
		it = t.template_descriptor_value(i)
		while it is not None:
			params.append(it.getText())
			i += 1
			it = t.template_descriptor_value(i)

		tem_params = SeaTemplateParams(template.template, params)
		self.template = tem_params

		if isinstance(template, SeaTemplateGroup):
			for id, t in template.templates.items():
				self._write_applied_template(id, t.apply(tem_params), t, params)
		else:
			self._write_applied_template(id, template.apply(tem_params), template, params)

		self.template = None

	def enterTag(self, ctx: Parser.TagContext):
		if self._should_skip(): return

		hashtags = [] if ctx.hashtag() is None else [HashTags.Tag[id.symbol.text.upper()] for id in self._get_all(ctx.hashtag().ID)]

		fields = {}
		i = 0
		it: Parser.Tag_entryContext = ctx.tag_entry(i)
		while it is not None:
			value = None if it.EQ() is None else it.number().getText()
			if value is not None:
				if '.' in value:
					print('error: tag value cannot be a float')
					exit(1)
				value = int(value)
			fields[it.ID().symbol.text] = value
			i += 1
			it = ctx.tag_entry(i)

		self.backend.tag(ctx.ID().symbol.text, SeaTag(fields, hashtags))

	def enterTagrec(self, ctx: Parser.TagrecContext):
		if self._should_skip(): return
		self.backend.tagrec(ctx.ID().symbol.text, self._get_tagrec_fields(ctx))


	def enterExpr_block(self, ctx: Parser.Expr_blockContext):
		if self._should_skip(): return
		self.backend.block_begin()

	def exitExpr_block(self, ctx: Parser.Expr_blockContext):
		if self._should_skip(): return
		if self.add_implicit_break_statement:
			self.backend.case_break()
			self.add_implicit_break_statement = False
		self.backend.block_end()

	def enterStat_if(self, ctx: Parser.Stat_ifContext):
		if self._should_skip(): return
		self.backend.if_(self._writer(ctx, 1))

	def enterStat_else(self, ctx: Parser.Stat_elseContext):
		if self._should_skip(): return
		self.backend.else_()

	def enterStat_for(self, ctx: Parser.Stat_forContext):
		if self._should_skip(): return

		if ctx.TO() is not None: # for/in?/to
			self.backend.for_to(
				None if ctx.IN() is None else ctx.ID(),
				lambda: self.write_expr(ctx.expr(0)),
				lambda: self.write_expr(ctx.expr(1)),
			)
		elif ctx.expr(1) is None: # Single-expr for loop
			self.backend.for_single_expr(lambda: self.write_expr(ctx.expr(0)))
		else: # C-style for loops
			self.backend.for_c_style(
				lambda: self.write_expr(ctx.expr(0)),
				lambda: self.write_expr(ctx.expr(1)),
				lambda: self.write_expr(ctx.expr(2))
			)

	def enterStat_each(self, ctx: Parser.Stat_eachContext):
		if self._should_skip(): return
		self.backend.each_begin(ctx.ID(0).symbol.text, ctx.ID(1).symbol.text)

	def exitStat_each(self, ctx: Parser.Stat_eachContext):
		if self._should_skip(): return
		self.backend.each_end()

	def enterStat_switch(self, ctx: Parser.Stat_switchContext):
		if self._should_skip(): return
		self.backend.switch_begin(lambda: self.write_expr(ctx.expr()))

	def exitStat_switch(self, ctx: Parser.Stat_switchContext):
		if self._should_skip(): return
		self.backend.switch_end()

	def enterCase(self, ctx: Parser.CaseContext):
		if self._should_skip(): return
		if ctx.ELSE() is not None:
			self.backend.case_else()
		else:
			self.add_implicit_break_statement = ctx.FALL() is None
			self.backend.case(lambda: self.write_expr(ctx.expr()))

class SeaErrorListener(ErrorListener):
	def __init__(self, file_path):
		self.file_path = file_path
		self._symbol = ''

	def syntaxError(self, recognizer, offendingSymbol, line, column, msg, e) -> None:
		self._symbol = offendingSymbol
		print(f'\033[31m{self.file_path}:{line}:{column}\033[0m: {msg}')
		exit(1)

	@property
	def symbol(self):
		return self._symbol


def visit(file_path: str, backend: Backend, nostd: bool = False):
	lexer = Lexer(antlr4.FileStream(file_path))
	stream = antlr4.CommonTokenStream(lexer)
	parser = Parser(stream)
	parser.removeErrorListeners()
	error_listener = SeaErrorListener(file_path)
	parser.addErrorListener(error_listener)

	tree = parser.program()
	# print(tree.toStringTree(recog = parser))

	walker = antlr4.ParseTreeWalker()
	walker.walk(Visitor(backend, nostd), tree)
