use std::path::PathBuf;

use thiserror::Error;

use crate::parse::error::ParseError;

#[derive(Debug, Clone, Error)]
pub enum SeaError {
    #[error("parsing error")]
    Parse {
        path: PathBuf,
        #[source]
        why: ParseError,
    },
}
