use std::{error::Error, fmt::Display};

// TODO: Inline the PluginError here
pub use crate::private::plugins::error::PluginError;

#[derive(Debug)]
pub enum SceneError {
    EntityNotFound,
    PluginError(PluginError),
    PluginCompatibilityError(PluginCompatibilityError),
    EntityError(EntityError),
    NoComponentType,
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
    ComponentAlreadyExists,
    ComponentError(ComponentError),
}

impl Display for EntityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for EntityError {}

#[derive(Debug)]
pub enum ComponentError {
    FieldNotFound,
    FieldHasNoGetter,
    FieldIsNotMutable,
}

impl Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ComponentError {}
