use std::{error::Error, fmt::Display};

pub use crate::private::plugins::error::PluginError;

#[derive(Debug)]
pub enum SceneError {
    EntityNotFound,
    PluginError(PluginError),
}

impl Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for SceneError {}
