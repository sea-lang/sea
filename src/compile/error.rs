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
}
