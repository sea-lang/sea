import sys
import random
from typing import List, Optional
import antlr4

from backend import Backend
from backend_c import Backend_C
from compiler import Compiler
from syntax.LanguageLexer import LanguageLexer
from syntax.LanguageParser import LanguageParser
from syntax.LanguageListener import LanguageListener

class Visitor(LanguageListener):
	def __init__(self, backend: Backend):
		self.backend = backend

	def _writer(self, expr: LanguageParser.ExprContext, index: Optional[int] = None):
		if index is None:
			return lambda: self.write_expr(expr)
		else:
			return lambda: self.write_expr(expr.getChild(index))

	def _simple_writer(self, expr: LanguageParser.ExprContext):
		return lambda: self.write_expr(expr)

	def write_expr(self, expr: LanguageParser.ExprContext):
		def _writer(index: int):
			def it():
				self.write_expr(expr.getChild(index))
			return it

		# Literals
		if expr.NUMBER() is not None:
			self.backend.number(expr.NUMBER().getText())
		elif expr.STRING() is not None:
			self.backend.number(expr.STRING().getText())
		elif expr.TRUE() is not None:
			self.backend.true()
		elif expr.FALSE() is not None:
			self.backend.false()
		elif expr.ID() is not None:
			self.backend.id(expr.ID().getText())
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
		elif expr.OP_INC() is not None: self.backend.inc(_writer(1))
		elif expr.OP_DEC() is not None: self.backend.dec(_writer(1))
		# Math
		elif expr.ADD() is not None: self.backend.add(_writer(0), _writer(1))
		elif expr.SUB() is not None: self.backend.sub(_writer(0), _writer(1))
		elif expr.MUL() is not None: self.backend.mul(_writer(0), _writer(1))
		elif expr.DIV() is not None: self.backend.div(_writer(0), _writer(1))
		elif expr.MOD() is not None: self.backend.mod(_writer(0), _writer(1))

	def exitStat(self, ctx:LanguageParser.StatContext):
		self.backend.write(self.backend.line_ending)

	def enterStat_var(self, ctx:LanguageParser.Stat_varContext):
		self.backend.var(ctx.ID().symbol.text, ctx.typedesc(), self._writer(ctx.expr()))

	def enterStat_let(self, ctx:LanguageParser.Stat_letContext):
		self.backend.let(ctx.ID().symbol.text, ctx.typedesc(), self._writer(ctx.expr()))

	def enterStat_assign(self, ctx:LanguageParser.Stat_assignContext):
		# self.backend.assign(ctx.ID().symbol.text, self._writer(ctx.expr()))
		pass

	def enterStat_ret(self, ctx:LanguageParser.Stat_retContext):
		self.backend.ret(self._writer(ctx.expr()))

	def enterStat_fun(self, ctx:LanguageParser.Stat_funContext):
		pass

	def exitStat_fun(self, ctx:LanguageParser.Stat_funContext):
	pass


	def enterTypedesc(self, ctx:LanguageParser.TypedescContext):
	pass

	def exitTypedesc(self, ctx:LanguageParser.TypedescContext):
	pass


	def enterExpr(self, ctx:LanguageParser.ExprContext):
	pass

	def exitExpr(self, ctx:LanguageParser.ExprContext):
	pass


	def enterExpr_block(self, ctx:LanguageParser.Expr_blockContext):
	pass

	def exitExpr_block(self, ctx:LanguageParser.Expr_blockContext):
	pass


	def enterExpr_if(self, ctx:LanguageParser.Expr_ifContext):
	pass

	def exitExpr_if(self, ctx:LanguageParser.Expr_ifContext):
	pass


	def enterExpr_for(self, ctx:LanguageParser.Expr_forContext):
	pass

	def exitExpr_for(self, ctx:LanguageParser.Expr_forContext):
	pass


	def enterExpr_each(self, ctx:LanguageParser.Expr_eachContext):
	pass

	def exitExpr_each(self, ctx:LanguageParser.Expr_eachContext):
	pass


	def enterPart_invoke(self, ctx:LanguageParser.Part_invokeContext):
	pass

	def exitPart_invoke(self, ctx:LanguageParser.Part_invokeContext):
	pass


def visit(file_path: str):
	input_stream = antlr4.FileStream(file_path)
	lexer = LanguageLexer(input_stream)
	stream = antlr4.CommonTokenStream(lexer)
	parser = LanguageParser(stream)

	tree = parser.program()
	walker = antlr4.ParseTreeWalker()
	visitor = Visitor()
	walker.walk(visitor, tree)
