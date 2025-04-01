import os
from typing import Optional
import antlr4
from antlr4.tree.Tree import TerminalNodeImpl
from antlr4.error.ErrorListener import ErrorListener
from .backend import Backend
from .backend_c import Backend_C
from .compiler import SEA_VOID, Compiler, SeaFunction, SeaFunctionTemplate, SeaRecord, SeaTemplate, SeaTemplateParams, SeaType
from .syntax.Lexer import Lexer
from .syntax.Parser import Parser
from .syntax.ParserListener import ParserListener

class Visitor(ParserListener):
	ALLOWED_TEMPLATE_TYPES = ['type', 'int', 'char', 'bool', 'float', 'double']

	def __init__(self, backend: Backend):
		self.backend = backend
		self.skipping = False
		# Used to apply template parameters in expressions and statements
		self.template: Optional[SeaTemplateParams] = None

	def _writer(self, expr, index: Optional[int] = None):
		if index is None:
			return lambda: self.write_expr(expr)
		else:
			return lambda: self.write_expr(expr.getChild(index))

	def _simple_writer(self, expr: Parser.ExprContext):
		return lambda: self.write_expr(expr)

	def _should_skip(self) -> bool:
		return self.skipping

	def _get_fun(self, ctx: Parser.FunContext) -> SeaFunction:
		params = {}
		part = ctx.part_params()
		for i in range(1, part.getChildCount() - 1):
			param = part.children[i]
			if isinstance(param, TerminalNodeImpl):
				continue
			params[param.ID().symbol.text] = SeaType.from_str(param.typedesc().getText())
		del part

		if ctx.COLON() is not None:
			returns = SeaType.from_str(ctx.getChild(4).getText())
		else:
			returns = SEA_VOID

		return SeaFunction(returns, params)

	def _get_rec(self, ctx: Parser.RecContext) -> SeaRecord:
		fields = {}
		part = ctx.part_params()
		for i in range(1, part.getChildCount() - 1):
			param = part.children[i]
			if isinstance(param, TerminalNodeImpl):
				continue
			fields[param.ID().symbol.text] = SeaType.from_str(param.typedesc().getText())
		del part

		return SeaRecord(fields)

	def enterProgram(self, ctx: Parser.ProgramContext):
		self.backend.file_begin()

	def exitProgram(self, ctx: Parser.ProgramContext):
		self.backend.file_end()

	def enterComment(self, ctx: Parser.CommentContext):
		if self._should_skip(): return

		if ctx.COMMENT() is not None:
			self.backend.comment(ctx.COMMENT().getText().removeprefix('//'))
		else:
			self.backend.multiline_comment(ctx.MULTILINE_COMMENT().getText().removeprefix('/*').removesuffix('*/'))

	def enterUse(self, ctx: Parser.UseContext):
		if self._should_skip(): return

		module = ctx.part_path().getText()
		# We can assume that if module is already imported, module_lib is also already imported, so we won't even check for it here.
		if module in self.backend.using:
			return

		# Use the automagically imported `lib.sea` module, if it exists
		module_lib = module[:module.rfind('\\')] + '\\lib'
		if not module_lib in self.backend.using:
			path = module_lib.replace('\\', '/') + '.sea'
			# We won't throw an error if `lib.sea`` doesn't exist, since it's optional
			if os.path.exists(path):
				self.backend.using.append(module_lib)
				self.backend.use(module_lib)
				self.backend.module_stack.append(module_lib)
				visit(path, backend = self.backend)
				self.backend.module_stack.pop()

		# Use the module
		possible_paths = [
			module.replace('\\', '/') + '.sea',
			module.replace('\\', '/') + '/lib.sea'
		]
		for path in possible_paths:
			if os.path.exists(path):
				self.backend.using.append(module)
				self.backend.use(module)
				self.backend.module_stack.append(module)
				visit(path, backend = self.backend)
				self.backend.module_stack.pop()
				return
		print(f'error: module `{module}` does not exists (searched for `{possible_paths}`)')
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
			self.backend.invoke(expr.ID().symbol.text, items)
		elif expr.expr_ref() is not None:
			self.backend.ref(expr.expr_ref().ID().symbol.text)
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
		elif expr.NUMBER() is not None:
			self.backend.number(expr.NUMBER().getText())
		elif expr.STRING() is not None:
			text = expr.STRING().getText()
			if text[0] == 'c':
				self.backend.string(text[2:-1], True)
			else:
				self.backend.string(text[1:-1], False)
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
			self.backend.array(items)
		elif expr.expr_new() is not None:
			e = expr.expr_new()
			name = e.ID().symbol.text

			start = 1
			if e.template_descriptor() is not None:
				print('hai')
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

			print(start)
			print(name)

			items = []
			for i in range(start, e.getChildCount() - 1):
				item = e.children[i]
				if isinstance(item, TerminalNodeImpl):
					continue
				items.append(self._writer(e, i))

			self.backend.new(name, items)
		elif expr.expr_var() is not None:
			e = expr.expr_var()
			self.backend.var(
				e.ID().symbol.text,
				SeaType.from_str(e.typedesc().getText(), template = self.template),
				self._writer(e.expr())
			)
		elif expr.expr_let() is not None:
			e = expr.expr_let()
			self.backend.let(
				e.ID().symbol.text,
				SeaType.from_str(e.typedesc().getText(), template = self.template),
				self._writer(e.expr())
			)
		elif expr.EQ() is not None:
			self.backend.assign(self._writer(expr, 0), self._writer(expr, 2))

	def enterTop_level_stat(self, ctx: Parser.Top_level_statContext):
		if self._should_skip(): return

		if ctx.expr_var() is not None:
			e = ctx.expr_var()
			self.backend.var(
				e.ID().symbol.text,
				SeaType.from_str(e.typedesc().getText()),
				self._writer(e.expr())
			)
			self.backend.write(self.backend.line_ending, False)
		elif ctx.expr_let() is not None:
			e = ctx.expr_let()
			self.backend.let(
				e.ID().symbol.text,
				SeaType.from_str(e.typedesc().getText()),
				self._writer(e.expr())
			)
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
		self.backend.ret(self._writer(ctx.expr()))

	def enterFun(self, ctx: Parser.FunContext):
		if self._should_skip(): return
		self.backend.fun_begin(ctx.ID().symbol.text, self._get_fun(ctx))

	def exitFun(self, ctx: Parser.FunContext):
		if self._should_skip(): return
		self.backend.fun_end()

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

		# Add templates
		i = 0
		it = ctx.top_level_stat(i)
		while it is not None:
			if it.fun() is not None:
				fun = it.fun()
				self.backend.compiler.add_function_template(fun.ID().symbol.text, self._get_fun(fun), fun.expr_block(), template)
			elif it.rec() is not None:
				rec = it.rec()
				self.backend.compiler.add_record_template(rec.ID().symbol.text, self._get_rec(rec), template)
			elif isinstance(it, Parser.CommentContext):
				pass
			else:
				print('error: templates can only contain functions and records, got: ' + str(type(it).__name__))
				exit(1)
			i += 1
			it = ctx.top_level_stat(i)

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
		params = []
		i = 0
		it = t.template_descriptor_value(i)
		while it is not None:
			params.append(it.getText())
			i += 1
			it = t.template_descriptor_value(i)

		tem_params = SeaTemplateParams(template.template, params)
		self.template = tem_params
		applied = template.apply(tem_params)

		if isinstance(applied, SeaFunction):
			name = id + ''.join([f'_{p}' for p in params])
			self.backend.fun_begin(name, applied)
			self.backend.block_begin()

			assert isinstance(template, SeaFunctionTemplate)
			end_offset = 1 if template.code.LCURLY() is not None else 0 # Check if we are a {} or a ->
			for i in range(1, template.code.getChildCount() - end_offset):
				it = template.code.getChild(i)
				print(it.getText())
				walker = antlr4.ParseTreeWalker()
				walker.walk(self, it)

			self.backend.block_end()
			self.backend.fun_end()
		elif isinstance(applied, SeaRecord):
			name = id + ''.join([f'_{p}' for p in params])
			self.backend.rec(name, applied)
		else:
			print(f'internal error: unhandled template application for `{applied}`\n\tthis error should never happen; please report it!')
			exit(1)

		self.template = None

	def enterExpr_block(self, ctx: Parser.Expr_blockContext):
		if self._should_skip(): return
		self.backend.block_begin()

	def exitExpr_block(self, ctx: Parser.Expr_blockContext):
		if self._should_skip(): return
		self.backend.block_end()

	def enterStat_if(self, ctx: Parser.Stat_ifContext):
		if self._should_skip(): return
		self.backend.if_(self._writer(ctx, 1))

	def enterStat_else(self, ctx: Parser.Stat_elseContext):
		if self._should_skip(): return
		self.backend.else_()

	def enterStat_for(self, ctx: Parser.Stat_forContext):
		if self._should_skip(): return

		if ctx.TO() is not None:
			self.backend.for_to(
				None if ctx.IN() is None else ctx.ID(),
				lambda: self.write_expr(ctx.expr(0)),
				lambda: self.write_expr(ctx.expr(1)),
			)
		else:
			self.backend.for_(
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


def visit(file_path: str, output_path: Optional[str] = None, backend: Optional[Backend] = None):
	lexer = Lexer(antlr4.FileStream(file_path))
	stream = antlr4.CommonTokenStream(lexer)
	parser = Parser(stream)
	parser.removeErrorListeners()
	error_listener = SeaErrorListener(file_path)
	parser.addErrorListener(error_listener)

	tree = parser.program()
	# print(tree.toStringTree(recog = parser))

	walker = antlr4.ParseTreeWalker()
	if backend is None:
		if output_path is None:
			print('error: visit() requires an output path when no backend is specified')
			exit(1)
		with Backend_C(Compiler(), output_path) as backend:
			walker.walk(Visitor(backend), tree)
	else:
		walker.walk(Visitor(backend), tree)
