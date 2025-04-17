use thiserror::Error;

use super::token;

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("unexpected character: `{0}`")]
    UnexpectedCharacter(char),

    #[error("unterminated string")]
    UnterminatedString,

    #[error("function pointer type missing parenthesis")]
    FunPtrMissingParenthesis,

    #[error("function pointer types cannot be arrays, use an array of a type alias to a function pointer instead")]
    FunPtrWithArrays,

    #[error("unexpected token: {0}")]
    UnexpectedToken(token::Token),

    #[error("expected expression")]
    ExpectedExpression,

    #[error("expected statement")]
    ExpectedStatement,
}
