use std::{error::Error, fmt::Display};

use crate::definitions::error::PluginDefinitionError;

/// Errors that a plugin might throw
#[derive(Debug)]
pub enum PluginError {
    IOError(std::io::Error),
    FailedToOpenPlugin,
    FailedToFindPluginDefinition,
    DefinitionValidationError(PluginDefinitionError),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<PluginDefinitionError> for PluginError {
    fn from(value: PluginDefinitionError) -> Self {
        Self::DefinitionValidationError(value)
    }
}
