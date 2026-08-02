use std::{error::Error, fmt};

use crate::scene::plugin::PluginError;

/// Recoverable component lookup, creation, field, and serialization failures.
#[derive(Debug, PartialEq, Eq)]
pub enum ComponentError {
    AlreadyExists,
    NotFound,
    MethodNotFound,
    TypeNotFound,
    CreatorFailed,
    FieldNotFound,
    FieldNoGetter,
    FieldNotMutable,
    FieldNoSerializer,
    FieldNoDeserializer,
    NoCreator(PluginError),
    NoDestroyer(PluginError),
    FieldParsing,
    FieldValueParsing,
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => f.write_str("component already exists on the entity"),
            Self::NotFound => f.write_str("component was not found on the entity"),
            Self::MethodNotFound => f.write_str("component method was not found"),
            Self::TypeNotFound => f.write_str("no loaded plugin provides this component type"),
            Self::CreatorFailed => f.write_str("component creator returned a null pointer"),
            Self::FieldNotFound => f.write_str("component field was not found"),
            Self::FieldNoGetter => f.write_str("component field has no getter"),
            Self::FieldNotMutable => f.write_str("component field is not mutable"),
            Self::FieldNoSerializer => f.write_str("component field has no serializer"),
            Self::FieldNoDeserializer => f.write_str("component field has no deserializer"),
            Self::NoCreator(error) => write!(f, "component creator could not be resolved: {error}"),
            Self::NoDestroyer(error) => {
                write!(f, "component destroyer could not be resolved: {error}")
            }
            Self::FieldParsing => {
                f.write_str("component query fields do not match the requested type")
            }
            Self::FieldValueParsing => {
                f.write_str("component field value could not be parsed or rendered")
            }
        }
    }
}

impl Error for ComponentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoCreator(error) | Self::NoDestroyer(error) => Some(error),
            _ => None,
        }
    }
}
