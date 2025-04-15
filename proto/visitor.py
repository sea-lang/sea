import os
from pathlib import Path
from typing import NamedTuple, Optional
import antlr4
from antlr4.tree.Tree import TerminalNodeImpl
from antlr4.error.ErrorListener import ErrorListener

from .infer import infer_type
from .backend import Backend
from .compiler import SEA_VOID, HashTags, SeaFunction, SeaRecord, SeaTag, SeaTagRec, SeaType
from .syntax.Lexer import Lexer
from .syntax.Parser import Parser
from .syntax.ParserListener import ParserListener

# This visitor looks for functions, records, etc to generate forward declarations for
class PreVisitor(ParserListener):
	def __init__(self, backend: Backend):
		self.backend = backend

	def _get_all(self, to_find) -> list:
		ret = []
		i = 0
		it = to_find(i)
		while it is not None:
			ret.append(it)
			i += 1
			it = to_find(i)
		return ret

class Visitor(ParserListener):
	def __init__(self, previsitor: PreVisitor, backend: Backend, nostd: bool = False):
		self.previsitor = previsitor
		self.backend = backend
		self.nostd = nostd
		self.skipping = False
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

	def enterProgram(self, ctx: Parser.ProgramContext):
		self.backend.file_begin()

		# Implicit `use std`
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

		# Write forward declarations

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

			self.backend.invoke(expr.ID().symbol.text, items)
		elif expr.expr_ref() is not None:
			self.backend.ref(lambda: self.write_expr(expr.expr_ref().expr()))
		elif expr.PTR() is not None and expr.expr() is not None:
			self.backend.deref(lambda: self.write_expr(expr.getChild(0)))
		elif expr.AS() is not None:
			self.backend.cast(SeaType.from_str(expr.typedesc().getText()), lambda: self.write_expr(expr.getChild(0)))
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

			type = None if len(items) == 0 else infer_type(self.backend.compiler, expr)

			self.backend.array(type, items)
		elif expr.expr_new() is not None:
			e = expr.expr_new()
			name = e.ID().symbol.text

			start = 1

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
				typ = infer_type(self.backend.compiler, e.expr())
			else:
				typ = SeaType.from_str(e.typedesc().getText())

			if e.expr().expr_list() is not None:
				l = e.expr().expr_list()
				size = len(self._get_all(l.expr))
				if typ.arrays[0] == -1:
					typ = typ.copy_with(arrays = [size, *typ.arrays[1:]])

			self.backend.var(name, typ, self._writer(e.expr()))
		elif expr.expr_let() is not None:
			e = expr.expr_let()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr())
			else:
				typ = SeaType.from_str(e.typedesc().getText())

			if e.expr().expr_list() is not None:
				l = e.expr().expr_list()
				size = len(self._get_all(l.expr))
				if typ.arrays[0] == -1:
					typ = typ.copy_with(arrays = [size, *typ.arrays[1:]])

			self.backend.let(name, typ, self._writer(e.expr()))
		elif expr.invoke_mac() is not None:
			e = expr.invoke_mac()
			name = e.ID().symbol.text
			params = self._get_all(e.expr)
			self._expand_macro(name, params)
		elif expr.EQ() is not None:
			self.backend.assign(self._writer(expr, 0), self._writer(expr, 2))

	def enterTop_level_stat(self, ctx: Parser.Top_level_statContext):
		if self._should_skip(): return

		if ctx.expr_var() is not None:
			e = ctx.expr_var()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr())
			else:
				typ = SeaType.from_str(e.typedesc().getText())
			self.backend.var(name, typ, lambda: self.write_expr(e.expr()), is_top_level = True)
			self.backend.write(self.backend.line_ending, False)
		elif ctx.expr_let() is not None:
			e = ctx.expr_let()
			name = e.ID().symbol.text
			if e.typedesc() is None:
				typ = infer_type(self.backend.compiler, e.expr())
			else:
				typ = SeaType.from_str(e.typedesc().getText())
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

	# Previsit
	previsitor = PreVisitor(backend)
	walker.walk(previsitor, tree)

	# Compile
	walker.walk(Visitor(previsitor, backend, nostd), tree)
