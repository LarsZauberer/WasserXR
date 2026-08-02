use std::{error::Error, fmt};

use crate::scene::plugin::PluginError;

/// Recoverable system lookup, creation, and lifecycle failures.
#[derive(Debug, PartialEq, Eq)]
pub enum SystemError {
    AlreadyExists,
    NotFound,
    TypeNotFound,
    NoRunner(PluginError),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => f.write_str("system already exists"),
            Self::NotFound => f.write_str("system was not found"),
            Self::TypeNotFound => f.write_str("no loaded plugin provides this system"),
            Self::NoRunner(error) => write!(f, "system runner could not be resolved: {error}"),
        }
    }
}

impl Error for SystemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoRunner(error) => Some(error),
            _ => None,
        }
    }
}
