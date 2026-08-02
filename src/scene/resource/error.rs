use std::{error::Error, fmt};

/// Recoverable resource-operation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    AlreadyExists,
    NotFound,
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => f.write_str("resource already exists"),
            Self::NotFound => f.write_str("resource was not found"),
        }
    }
}

impl Error for ResourceError {}
