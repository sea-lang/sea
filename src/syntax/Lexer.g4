lexer grammar Lexer;

// Tokens
EQ: '=';
COMMA: ',';
COLON: ':';
SEMI: ';';
PTR: '^';
LPAREN: '(';
RPAREN: ')';
LBRACKET: '[';
RBRACKET: ']';
LCURLY: '{';
RCURLY: '}';
BACKSLASH: '\\';

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
OP_PIPE: '->';

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
OF: 'of';
NEW: 'new';
REF: 'ref';
AS: 'as';
TO: 'to';
IN: 'in';
DEF: 'def';

TRUE: 'true';
FALSE: 'false';

NUMBER: '-'? [0-9]+ ('.' [0-9]+)?;
ID: [a-zA-Z_$][a-zA-Z_$0-9]*;
WS: [ \t\n\r\f]+ -> skip;
// Comments are intentionally left unskipped so they can be preserved in transpiles
COMMENT: '//' ~[\r\n]*;
MULTILINE_COMMENT: '/*' .*? '*/';
// https://stackoverflow.com/a/24559773
UNTERMINATED_STRING: '"' (~["\\] | '\\' ( . | EOF))*;
STRING: 'c'? UNTERMINATED_STRING '"';

RAW_BLOCK: 'raw[' -> pushMode(RAW_MODE);

// Used for `raw` blocks
mode RAW_MODE;
RAW_TEXT: ~(']')+;
END_RAW_BLOCK: ']' -> popMode;
