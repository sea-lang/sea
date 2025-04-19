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

    #[error("unexpected token: `{}`", .0.text)]
    UnexpectedToken(token::Token),

    #[error("expected token but got `{}`", .0.text)]
    ExpectedToken(token::Token),

    #[error("expected expression but got `{}`", .0.text)]
    ExpectedExpression(token::Token),

    #[error("expected statement but got `{}`", .0.text)]
    ExpectedStatement(token::Token),

    #[error("(internal error) advance error (caused by: {0})")]
    AdvanceError(Box<ParseError>),
}
