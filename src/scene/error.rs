//! Errors produced by scene operations.

use std::{error::Error, fmt};

/// An error produced by a scene operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    /// The requested entity does not exist.
    EntityNotFound,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotFound => formatter.write_str("entity not found"),
        }
    }
}

impl Error for SceneError {}
