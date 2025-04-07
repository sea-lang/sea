parser grammar Parser;

options {
	tokenVocab = Lexer;
}

program: top_level_stat* EOF;

hashtag: '#' (('(' ID (',' ID)* ')') | ID);
number: INT | FLOAT | HEX | BINARY;

top_level_stat:
	use
	| fun
	| rec
	| def
	| tem
	| gen
	| tag
	| tagrec
	| raw_block
	| expr_var
	| expr_let;

use: 'use' part_path;
fun: hashtag? 'fun' ID part_params (':' typedesc)? expr_block?;
raw_block: RAW_BLOCK RAW_TEXT END_RAW_BLOCK;
rec: hashtag? 'rec' ID part_params;
def: 'def' ID '=' typedesc;
tem: 'tem' '(' template_def_param (',' template_def_param)* ')' (('{' top_level_stat* '}') | ('->' top_level_stat));
gen: 'gen' ID template_descriptor;
tag: hashtag? 'tag' ID '(' tag_entry (','? tag_entry)* ')';
tagrec: hashtag? 'tag' 'rec' ID '(' tagrec_entry (','? tagrec_entry)* ')';

tag_entry: ID ('=' number)?;
tagrec_entry: ID part_params;

stat:
	stat_ret
	| stat_if
	| stat_else
	| stat_for
	| stat_each
	| stat_switch
	| raw_block
	| expr_block
	| expr;

stat_if: 'if' expr stat;
stat_else: 'else' stat;
stat_ret: 'ret' expr?;
stat_for:
	'for' (
		(expr ';' expr ';' expr)
		| expr
		| ((ID 'in')? expr 'to' expr)
	) stat;
stat_each: 'each' ID 'of' ID stat;
stat_switch: 'switch' expr '{' case* '}';
case: (('fall'? 'case' expr) | 'else') expr_block;

template_descriptor_value: typedesc | number | ID | TRUE | FALSE;
template_descriptor: '{' (template_descriptor_value (',' template_descriptor_value)*)? '}';
typedesc: '^'* (ID | ('fun' '(' (typedesc (',' typedesc)*)? ')' (':' typedesc)?)) template_descriptor? (LBRACKET (INT | ID)? RBRACKET)*;

expr:
	'(' expr ')'
	// Literals
	| number
	| STRING
	| CHAR
	| TRUE
	| FALSE
	| expr_new
	| expr '.' expr
	// Pointers
	| expr_ref
	| expr '^'
	// Assignment
	| expr '=' expr
	// Operators
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
	| ID template_descriptor? part_invoke
	| raw_block
	| expr_list
	| expr_var
	| expr_let
	| expr 'as' typedesc
	// IDs
	| ID;

expr_block: ('{' stat* '}') | ('->' stat);
expr_list: LBRACKET (expr (',' expr)* ','?)? RBRACKET;
expr_new: 'new' ID template_descriptor? '(' (expr (',' expr)* ','?)? ')';
expr_var: 'var' ID (':' typedesc)? '=' expr;
expr_let: 'let' ID (':' typedesc)? '=' expr;
expr_ref: 'ref' expr;

// "Parts" Allow me to break things up into smaller parts for ease-of-use
part_invoke: '(' (expr (',' expr)*)? ')';
part_params: '(' (part_param (',' part_param)*)? ')';
part_param: ID ':' typedesc;
part_path: ID ('/' ID)*;
part_index: LBRACKET expr RBRACKET;
template_def_param: ID ':' ID;
