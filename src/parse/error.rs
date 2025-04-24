use thiserror::Error;

use super::token::{Token, TokenKind};

#[derive(Debug, Clone, Error)]
pub enum ParseError {
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

    #[error("(internal error) advance error (caused by: {0})")]
    AdvanceError(Box<ParseError>),

    #[error("expected token of kind `{0:?}`")]
    ExpectedTokenOfKind(TokenKind),
}
