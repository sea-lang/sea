parser grammar Parser;

options {
	tokenVocab = Lexer;
}

// Grammar
program: top_level_stat* EOF;

comment: COMMENT | MULTILINE_COMMENT;

top_level_stat:
	stat_var
	| stat_let
	| fun
	| rec
	| raw_block
	| comment;

fun: 'fun' ID part_params (':' typedesc)? expr;
raw_block: RAW_BLOCK RAW_TEXT END_RAW_BLOCK;
rec: 'rec' ID part_params;

stat:
	stat_var
	| stat_let
	| stat_assign
	| stat_ret
	| stat_for
	| stat_each
	| raw_block
	| comment
	| expr;

stat_var: 'var' ID ':' typedesc '=' expr;
stat_let: 'let' ID ':' typedesc '=' expr;
stat_assign: ID '=' expr;
stat_ret: 'ret' expr;
stat_for: 'for' expr ';' expr ';' expr expr_block;
stat_each: 'each' ID 'of' ID expr_block;

typedesc: '^'* ID (LBRACKET NUMBER? RBRACKET)*;

expr:
	// Literals
	NUMBER
	| STRING
	| TRUE
	| FALSE
	| ID
	| expr_new
	// Operators
	| expr '.' ID
	| 'not' expr
	| expr 'and' expr
	| expr 'or' expr
	| expr '==' expr
	| expr '!=' expr
	| expr '>' expr
	| expr '>=' expr
	| expr '<' expr
	| expr '<=' expr
	// Math
	| expr '*' expr
	| expr '/' expr
	| expr '%' expr
	| expr '+' expr
	| expr '-' expr
	| expr '++'
	| expr '--'
	// Control flow and friends
	| expr part_invoke
	| expr_block
	| raw_block
	| expr_if
	| expr_list
	// Comments
	| comment;

expr_if: 'if' expr expr_block ('else' expr_block)?;
expr_block: ('{' stat* '}') | ('->' stat);
expr_list: LBRACKET (expr ','?)* RBRACKET;
expr_new: 'new' ID '(' (expr ','?)* ')';

// "Parts" Allow me to break things up into smaller parts for ease-of-use
part_invoke: '(' (expr ','?)* ')';
part_params: '(' (part_param ','?)* ')';
part_param: ID ':' typedesc;