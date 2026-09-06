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
        match self {
            Self::EntityNotFound => f.write_str("entity not found"),
            Self::PluginError(error) => write!(f, "plugin error: {error}"),
            Self::PluginCompatibilityError(error) => {
                write!(f, "plugin compatibility error: {error}")
            }
            Self::EntityError(error) => write!(f, "entity error: {error}"),
            Self::NoComponentType => f.write_str("component type not found"),
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
