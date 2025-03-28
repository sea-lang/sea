parser grammar Parser;

options {
	tokenVocab = Lexer;
}

// Grammar
program: top_level_stat* EOF;

comment: COMMENT | MULTILINE_COMMENT;

top_level_stat:
	use
	| fun
	| rec
	| raw_block
	| expr_var
	| expr_let
	| comment;

use: 'use' part_path;
fun: 'fun' ID part_params (':' typedesc)? expr;
raw_block: RAW_BLOCK RAW_TEXT END_RAW_BLOCK;
rec: 'rec' ID part_params;

stat:
	stat_ret
	| stat_for
	| stat_each
	| raw_block
	| comment
	| expr;

stat_ret: 'ret' expr;
stat_for:
	'for' (
		(expr ';' expr ';' expr)
		| ((ID 'in')? expr 'to' expr)
	) expr_block;
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
	| expr '.' expr
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
	| expr part_index
	| expr part_invoke
	| expr_block
	| raw_block
	| expr_if
	| expr_list
	| expr_var
	| expr_let
	| expr '=' expr
	| expr_ref
	| expr '^'
	| expr 'as' typedesc
	// Comments
	| comment;

expr_if: 'if' expr expr_block ('else' expr_block)?;
expr_block: ('{' stat* '}') | ('->' stat);
expr_list: LBRACKET (expr (',' expr)* ','?)? RBRACKET;
expr_new: 'new' ID '(' (expr (',' expr)* ','?)? ')';
expr_var: 'var' ID ':' typedesc '=' expr;
expr_let: 'let' ID ':' typedesc '=' expr;
expr_ref: 'ref' ID;

// "Parts" Allow me to break things up into smaller parts for ease-of-use
part_invoke: '(' (expr (',' expr)*)? ')';
part_params: '(' (part_param (',' part_param)*)? ')';
part_param: ID ':' typedesc;
part_path: ID ('\\' ID)*;
part_index: LBRACKET expr RBRACKET;