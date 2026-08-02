use std::{error::Error, fmt};

/// Recoverable entity-operation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityError {
    NotFound,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("entity was not found"),
        }
    }
}

impl Error for EntityError {}
