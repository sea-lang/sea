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
HASH: '#';
AT: '@';

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
MAC: 'mac';
LIT: 'lit';
TAG: 'tag';
SWITCH: 'switch';
CASE: 'case';
FALL: 'fall';

TRUE: 'true';
FALSE: 'false';

FLOAT: '-'? [_0-9]+ '.' ([_0-9])+ [df]?;
INT: '-'? [_0-9]+ [lu]?;
HEX: '0x' [0-9a-fA-F_]*;
BINARY: '0b' [01_]*;

ID: [a-zA-Z_$][a-zA-Z_$0-9]*;
WS: [ \t\n\r\f]+ -> skip;
COMMENT: '//' ~[\r\n]* -> skip;
MULTILINE_COMMENT: '/*' .*? '*/' -> skip;
// https://stackoverflow.com/a/24559773
UNTERMINATED_STRING: '"' (~["\\] | '\\' ( . | EOF))*;
STRING: 'c'? UNTERMINATED_STRING '"';
CHAR: '`' (~[`\\] | '\\' ( . | EOF))+ '`';

RAW_BLOCK: 'raw[' -> pushMode(RAW_MODE);

// Used for `raw` blocks
mode RAW_MODE;
	RAW_TEXT: ~(']')+;
	END_RAW_BLOCK: ']' -> popMode;
