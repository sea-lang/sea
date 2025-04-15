from .syntax.Parser import Parser
from .compiler import *

def infer_type(compiler: Compiler, expr: Parser.ExprContext) -> SeaType:
	# TODO: Organize
	if expr.expr() is not None and expr.getChild(0).getText() == '(':
		return infer_type(compiler, expr.getChild(1))
	elif expr.part_invoke() is not None:
		name = expr.ID().symbol.text

		if name not in compiler.functions:
			print(f'error: cannot infer type for unknown (or unbound) function `{name}`')
			exit(1)

		return compiler.functions[name].returns
	elif expr.expr_ref() is not None:
		typ = infer_type(compiler, expr.expr_ref().expr())
		return typ.copy_with(pointers = typ.pointers + 1)
	elif expr.PTR() is not None and expr.expr() is not None:
		typ = infer_type(compiler, expr.getChild(0))
		return typ.copy_with(pointers = typ.pointers - 1)
	elif expr.AS() is not None:
		return SeaType.from_str(expr.typedesc().getText())
	elif expr.part_index() is not None:
		typ = infer_type(compiler, expr.getChild(0))
		typ.arrays.pop()
		return typ
	elif expr.OP_DOT() is not None:
		first = infer_type(compiler, expr.getChild(0))
		field = expr.getChild(2).getText()
		if first.name not in compiler.records:
			print('error: cannot infer type for non-existent struct: ' + first.name)
			exit(1)
		elif field not in compiler.records[first.name].fields:
			print(f'error: cannot infer type for non-existent field `{field}` for struct `{first.name}`')
			exit(1)
		return compiler.records[first.name].fields[field]
	elif (
		expr.OP_NOT() is not None or
		expr.OP_AND() is not None or
		expr.OP_OR() is not None or
		expr.OP_EQ() is not None or
		expr.OP_NEQ() is not None or
		expr.OP_GT() is not None or
		expr.OP_GTEQ() is not None or
		expr.OP_LT() is not None or
		expr.OP_LTEQ() is not None
	): return SEA_BOOL
	elif expr.OP_INC() is not None or expr.OP_DEC() is not None:
		return infer_type(compiler, expr.getChild(0))
	# Math
	elif (
		expr.ADD() is not None or
		expr.SUB() is not None or
		expr.MUL() is not None or
		expr.DIV() is not None or
		expr.MOD() is not None
	): return infer_type(compiler, expr.getChild(0))
	# Literals
	elif expr.number() is not None:
		n: Parser.NumberContext = expr.number()
		if n.FLOAT() is not None:
			if 'd' in n.getText():
				return SEA_DOUBLE
			return SEA_FLOAT
		elif n.INT() is not None or n.BINARY() is not None or n.HEX() is not None: # TODO: Check for size (int16, int32, int64, etc)
			# TODO: Check for character affixes
			return SEA_INT
	elif expr.STRING() is not None:
		return SEA_CSTRING if expr.STRING().getText()[0] == 'c' else SEA_STRING
	elif expr.TRUE() is not None or expr.FALSE() is not None:
		return SEA_BOOL
	elif expr.ID() is not None:
		it = expr.ID().getText()
		if it in compiler.variables:
			return compiler.variables[it].type
		elif it in compiler.functions:
			return compiler.functions[it].get_type(it)
		elif it in compiler.tag_values_to_tag_name:
			return SeaType.from_str(compiler.tag_values_to_tag_name[it])
		else:
			print(f'error: cannot infer type for identifier `{it}`')
			exit(1)
	elif expr.expr_list() is not None:
		e = expr.expr_list()
		if e.getChildCount() == 2: # Check to make sure the list isn't empty
			print(f'error: cannot infer type of empty list')
			exit(1)
		# We'll infer the type of the first child
		return infer_type(compiler, e.getChild(1)).copy_with(arrays = [-1]) # TODO: Array size
	elif expr.expr_new() is not None:
		e = expr.expr_new()
		name = e.ID().symbol.text
		return SeaType(0, name, [], [], None)
	elif expr.expr_var() is not None:
		return infer_type(compiler, expr.expr_var().expr())
	elif expr.expr_let() is not None:
		return infer_type(compiler, expr.expr_let().expr())
	elif expr.EQ() is not None:
		return infer_type(compiler, expr.EQ().getChild(2))
	else:
		print(f'internal error: cannot infer type for expression: {expr.getText()}')
		exit(1)
	return SEA_VOID
