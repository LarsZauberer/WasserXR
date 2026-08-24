use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum SceneError {
    EntityNotFound,
}

impl Error for SceneError {}

impl Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityNotFound => write!(f, "entity not found"),
        }
    }
}
