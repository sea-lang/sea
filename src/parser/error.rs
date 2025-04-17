use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("unexpected character: `{0}`")]
    UnexpectedCharacter(char),

    #[error("unterminated string")]
    UnterminatedString,
}
