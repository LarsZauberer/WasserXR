use std::{error::Error, fmt};

/// Recoverable system lookup, creation, and lifecycle failures.
#[derive(Debug, PartialEq, Eq)]
pub enum SystemError {
    AlreadyExists,
    NotFound,
    TypeNotFound,
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => f.write_str("system already exists"),
            Self::NotFound => f.write_str("system was not found"),
            Self::TypeNotFound => f.write_str("no loaded plugin provides this system"),
        }
    }
}

impl Error for SystemError {}
