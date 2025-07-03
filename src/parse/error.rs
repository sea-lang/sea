use thiserror::Error;

use super::token::{Token, TokenKind};

#[derive(Debug, Clone, Error)]
pub enum LexErrorKind {
    #[error("unexpected character: `{0}`")]
    UnexpectedCharacter(char),

    #[error("expected character `{0}` but got `{1}`")]
    ExpectedCharacter(char, char),

    #[error("unterminated string")]
    UnterminatedString,

    #[error("unterminated char")]
    UnterminatedChar,

    #[error("unterminated raw block")]
    UnterminatedRawBlock,
}

// A lexer error with a fake token to indicate where the error occurred at.
#[derive(Debug, Clone)]
pub struct LexError {
    pub error: LexErrorKind,
    pub token: Token,
}

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("function pointer type missing parenthesis")]
    FunPtrMissingParenthesis,

    #[error("function pointer types cannot be arrays, use an array of a type alias to a function pointer instead")]
    FunPtrWithArrays,

    #[error("unexpected token: `{}`", .0.text)]
    UnexpectedToken(Token),

    #[error("expected token but got `{}`", .0.text)]
    ExpectedToken(Token),

    #[error("expected expression but got `{}`", .0.text)]
    ExpectedExpression(Token),

    #[error("expected statement but got `{}`", .0.text)]
    ExpectedStatement(Token),

    #[error("lexing error: {0}")]
    LexError(LexErrorKind),

    #[error("expected token of kind `{0:?}`")]
    ExpectedTokenOfKind(TokenKind),

    #[error("reached EOF before closing brace")]
    ReachedEOFBeforeClosingBrace,
}
