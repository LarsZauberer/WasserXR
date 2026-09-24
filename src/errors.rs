use std::{error::Error, fmt::Display};

use crate::definitions::error::PluginDefinitionError;

/// Error returned when text cannot be parsed as a field's declared type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldParseError {
    InvalidInput,
}

impl Display for FieldParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("input does not match the field type")
    }
}

impl Error for FieldParseError {}

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
        match self {
            Self::IOError(error) => write!(f, "I/O error: {error}"),
            Self::FailedToOpenPlugin => f.write_str("failed to open plugin"),
            Self::FailedToFindPluginDefinition => f.write_str("failed to find plugin definition"),
            Self::DefinitionValidationError(error) => {
                write!(f, "plugin definition validation error: {error}")
            }
        }
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IOError(error) => Some(error),
            Self::DefinitionValidationError(error) => Some(error),
            Self::FailedToOpenPlugin | Self::FailedToFindPluginDefinition => None,
        }
    }
}

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

#[derive(Debug)]
pub enum SceneError {
    EntityNotFound,
    PluginError(PluginError),
    PluginCompatibilityError(PluginCompatibilityError),
    EntityError(EntityError),
    ComponentError(ComponentError),
    NoComponentType,
    AssetNotFound,
    AssetError(AssetError),
    SystemNotFound,
    FunctionNotFound,
    RequestedTypeIDNotFound,
    SystemError(SystemError),
}

impl Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityNotFound => f.write_str("entity not found"),
            Self::PluginError(error) => write!(f, "plugin error: {error}"),
            Self::PluginCompatibilityError(error) => {
                write!(f, "plugin compatibility error: {error}")
            }
            Self::EntityError(error) => write!(f, "entity error: {error}"),
            Self::ComponentError(error) => write!(f, "component error: {error}"),
            Self::NoComponentType => f.write_str("component type not found"),
            Self::AssetNotFound => f.write_str("asset not found"),
            Self::AssetError(error) => write!(f, "asset error: {error}"),
            Self::SystemNotFound => f.write_str("system not found"),
            Self::FunctionNotFound => f.write_str("function not found"),
            Self::RequestedTypeIDNotFound => f.write_str("requested type ID not found"),
            Self::SystemError(error) => write!(f, "system error: {error}"),
        }
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
        match self {
            Self::PluginWithSameNameExists => {
                f.write_str("a plugin with the same name already exists")
            }
        }
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
        match self {
            Self::ComponentNotFound => f.write_str("component not found"),
            Self::ComponentAlreadyExists => f.write_str("component already exists"),
            Self::ComponentError(error) => write!(f, "component error: {error}"),
        }
    }
}

impl Error for EntityError {}

impl From<EntityError> for SceneError {
    fn from(value: EntityError) -> Self {
        SceneError::EntityError(value)
    }
}

#[derive(Debug)]
pub enum ComponentError {
    FieldNotFound,
    FieldError(FieldError),
}

impl Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldNotFound => f.write_str("field not found"),
            Self::FieldError(error) => write!(f, "field error: {error}"),
        }
    }
}

impl Error for ComponentError {}

impl From<ComponentError> for EntityError {
    fn from(value: ComponentError) -> Self {
        EntityError::ComponentError(value)
    }
}

impl From<ComponentError> for SceneError {
    fn from(value: ComponentError) -> Self {
        Self::ComponentError(value)
    }
}

#[derive(Debug)]
pub enum FieldError {
    NoGetter,
    NotMutable,
}

impl Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoGetter => f.write_str("field has no getter"),
            Self::NotMutable => f.write_str("field is not mutable"),
        }
    }
}

impl Error for FieldError {}

impl From<FieldError> for ComponentError {
    fn from(value: FieldError) -> Self {
        ComponentError::FieldError(value)
    }
}

#[derive(Debug)]
pub enum AssetError {
    CreationFailure,
    FieldNotFound,
}

impl Error for AssetError {}

impl Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreationFailure => f.write_str("asset creation failed"),
            Self::FieldNotFound => f.write_str("asset field not found"),
        }
    }
}

impl From<AssetError> for SceneError {
    fn from(value: AssetError) -> Self {
        SceneError::AssetError(value)
    }
}

#[derive(Debug)]
pub enum SystemError {
    AlreadyExists,
    DependencyNotFound(String),
    DependencyInUse,
    DependencyCycle,
}

impl Error for SystemError {}

impl Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists => f.write_str("system already exists"),
            Self::DependencyNotFound(name) => {
                write!(f, "system dependency '{name}' could not be resolved")
            }
            Self::DependencyInUse => f.write_str("system is required by another system"),
            Self::DependencyCycle => f.write_str("system dependencies contain a cycle"),
        }
    }
}

impl From<SystemError> for SceneError {
    fn from(value: SystemError) -> Self {
        SceneError::SystemError(value)
    }
}
