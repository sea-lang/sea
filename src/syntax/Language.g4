grammar Language;

// Tokens
EQ: '=';
COMMA: ',';
SEMI: ';';
LPAREN: '(';
RPAREN: ')';
LCURLY: '{';
RCURLY: '}';

OP_DOT: '.';
OP_NOT: 'not';
OP_AND: 'and';
OP_OR: 'or';
OP_EQ: '==';
OP_NEQ: '!=';
OP_GT: '>';
OP_GTEQ: '>=';
OP_LT: '<';
OP_LTEQ: '<=';
OP_INC: '++';
OP_DEC: '--';

ADD: '+';
SUB: '-';
MUL: '*';
DIV: '/';
MOD: '%';

USE: 'use';
REC: 'rec';
FUN: 'fun';
VAR: 'var';
LET: 'let';
RET: 'ret';
RAW: 'raw';
IF: 'if';
ELSE: 'else';
FOR: 'for';
EACH: 'each';

TRUE: 'true';
FALSE: 'false';

NUMBER: [0-9]+([.][0-9]+)?;
ID: [a-zA-Z_$][a-zA-Z_$0-9]*;
WS: [ \t\n\r\f]+ -> skip;
COMMENT: '#' ~[\r\n]* -> skip;
MULTILINE_COMMENT: '#' '{' .*? '#' '}' -> skip;
// https://stackoverflow.com/a/24559773
UNTERMINATED_STRING: '"' (~["\\] | '\\' ( . | EOF))*;
STRING: UNTERMINATED_STRING '"';

// Grammar
program: (top_level_stat)* EOF;

top_level_stat:
      stat_var
    | stat_let
    | fun
    | rec
    ;

fun: 'fun' ID '(' (ID ':' typedesc ','?)* ')' expr;
raw_block: 'raw' ('{' stat* '}') | ('->' stat);
rec: 'rec' ID '(' (ID ':' typedesc ','?)* ')';

stat:
      stat_var
    | stat_let
    | stat_assign
    | stat_ret
    | stat_for
    | stat_each
    | expr
    | raw_block
    ;

stat_var: 'var' ID ':' typedesc '=' expr;
stat_let: 'let' ID ':' typedesc '=' expr;
stat_assign: ID '=' expr;
stat_ret: 'ret' expr;
stat_for: 'for' expr ';' expr ';' expr expr_block;
stat_each: 'each' expr 'of' expr expr_block;

typedesc: '^'* ID '[]'*;

expr:
    // Operators
      expr '.' ID
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
    // Literals
    | NUMBER
    | STRING
    | TRUE
    | FALSE
    | ID
    ;

expr_if: 'if' expr expr_block ('else' expr_block)?;
expr_block: ('{' stat* '}') | ('->' stat);

// "Parts"
// Allow me to break things up into smaller parts for ease-of-use
part_invoke: '(' (expr ','?)* ')';
