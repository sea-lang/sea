# Generated from Language.g4 by ANTLR 4.13.2
from antlr4 import *
if "." in __name__:
    from .LanguageParser import LanguageParser
else:
    from LanguageParser import LanguageParser

# This class defines a complete listener for a parse tree produced by LanguageParser.
class LanguageListener(ParseTreeListener):

    # Enter a parse tree produced by LanguageParser#program.
    def enterProgram(self, ctx:LanguageParser.ProgramContext):
        pass

    # Exit a parse tree produced by LanguageParser#program.
    def exitProgram(self, ctx:LanguageParser.ProgramContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat.
    def enterStat(self, ctx:LanguageParser.StatContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat.
    def exitStat(self, ctx:LanguageParser.StatContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat_var.
    def enterStat_var(self, ctx:LanguageParser.Stat_varContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat_var.
    def exitStat_var(self, ctx:LanguageParser.Stat_varContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat_let.
    def enterStat_let(self, ctx:LanguageParser.Stat_letContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat_let.
    def exitStat_let(self, ctx:LanguageParser.Stat_letContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat_assign.
    def enterStat_assign(self, ctx:LanguageParser.Stat_assignContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat_assign.
    def exitStat_assign(self, ctx:LanguageParser.Stat_assignContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat_ret.
    def enterStat_ret(self, ctx:LanguageParser.Stat_retContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat_ret.
    def exitStat_ret(self, ctx:LanguageParser.Stat_retContext):
        pass


    # Enter a parse tree produced by LanguageParser#stat_fun.
    def enterStat_fun(self, ctx:LanguageParser.Stat_funContext):
        pass

    # Exit a parse tree produced by LanguageParser#stat_fun.
    def exitStat_fun(self, ctx:LanguageParser.Stat_funContext):
        pass


    # Enter a parse tree produced by LanguageParser#typedesc.
    def enterTypedesc(self, ctx:LanguageParser.TypedescContext):
        pass

    # Exit a parse tree produced by LanguageParser#typedesc.
    def exitTypedesc(self, ctx:LanguageParser.TypedescContext):
        pass


    # Enter a parse tree produced by LanguageParser#expr.
    def enterExpr(self, ctx:LanguageParser.ExprContext):
        pass

    # Exit a parse tree produced by LanguageParser#expr.
    def exitExpr(self, ctx:LanguageParser.ExprContext):
        pass


    # Enter a parse tree produced by LanguageParser#expr_block.
    def enterExpr_block(self, ctx:LanguageParser.Expr_blockContext):
        pass

    # Exit a parse tree produced by LanguageParser#expr_block.
    def exitExpr_block(self, ctx:LanguageParser.Expr_blockContext):
        pass


    # Enter a parse tree produced by LanguageParser#expr_if.
    def enterExpr_if(self, ctx:LanguageParser.Expr_ifContext):
        pass

    # Exit a parse tree produced by LanguageParser#expr_if.
    def exitExpr_if(self, ctx:LanguageParser.Expr_ifContext):
        pass


    # Enter a parse tree produced by LanguageParser#expr_for.
    def enterExpr_for(self, ctx:LanguageParser.Expr_forContext):
        pass

    # Exit a parse tree produced by LanguageParser#expr_for.
    def exitExpr_for(self, ctx:LanguageParser.Expr_forContext):
        pass


    # Enter a parse tree produced by LanguageParser#expr_each.
    def enterExpr_each(self, ctx:LanguageParser.Expr_eachContext):
        pass

    # Exit a parse tree produced by LanguageParser#expr_each.
    def exitExpr_each(self, ctx:LanguageParser.Expr_eachContext):
        pass


    # Enter a parse tree produced by LanguageParser#raw_block.
    def enterRaw_block(self, ctx:LanguageParser.Raw_blockContext):
        pass

    # Exit a parse tree produced by LanguageParser#raw_block.
    def exitRaw_block(self, ctx:LanguageParser.Raw_blockContext):
        pass


    # Enter a parse tree produced by LanguageParser#part_invoke.
    def enterPart_invoke(self, ctx:LanguageParser.Part_invokeContext):
        pass

    # Exit a parse tree produced by LanguageParser#part_invoke.
    def exitPart_invoke(self, ctx:LanguageParser.Part_invokeContext):
        pass



del LanguageParser