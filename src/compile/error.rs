use thiserror::Error;

use super::symbol::Symbol;

#[derive(Debug, Clone, Error)]
pub enum CompilerError {
    #[error("undefined or unbound symbol: `{0}`")]
    UnknownSymbol(String),

    #[error("cannot instantiate `{0}`")]
    Uninstantiatable(String, Symbol),

    #[error("tag rec instantiation requires a kind")]
    TagRecInstantiateWithoutKind,

    #[error("import error: {0}")]
    ImportError(String),

    #[error("no such pragma: {0}")]
    NoSuchPragma(String),

    #[error("pragma expected {0} argument(s) but got {1}")]
    NotEnoughOrTooManyPragmaArguments(usize, usize),

    #[error("invalid pragma arguments: expected `{0}` at index `{1}`")]
    InvalidPragmaArguments(String, usize),
}
