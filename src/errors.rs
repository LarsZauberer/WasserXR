use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum SceneError {
    EntityNotFound,
}

impl Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for SceneError {}
