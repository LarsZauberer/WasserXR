use std::{error::Error, fmt::Display};

pub use crate::private::plugins::error::PluginError;

#[derive(Debug)]
pub enum SceneError {
    EntityNotFound,
    PluginError(PluginError),
    PluginCompatibilityError(PluginCompatibilityError),
}

impl Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for SceneError {}

impl From<PluginError> for SceneError {
    fn from(value: PluginError) -> Self {
        Self::PluginError(value)
    }
}

#[derive(Debug)]
pub enum PluginCompatibilityError {
    PluginWithSameNameExists,
}

impl Display for PluginCompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PluginCompatibilityError {}

impl From<PluginCompatibilityError> for SceneError {
    fn from(value: PluginCompatibilityError) -> Self {
        Self::PluginCompatibilityError(value)
    }
}

#[derive(Debug)]
pub enum EntityError {
    ComponentNotFound,
}

impl Display for EntityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for EntityError {}
