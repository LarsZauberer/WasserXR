use std::{error::Error, fmt::Display};

use crate::utils::{ffi::StringError, version::Version};

#[derive(Debug, PartialEq, Eq)]
pub enum PluginDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    EngineVersionMismatch {
        name: String,
        expected: Version,
        actual: Version,
    },
    ComponentsIsNull(String),
    ComponentInvalid(String, ComponentDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    CreatorIsNull(String),
    DestroyerIsNull(String),
    FieldsIsNull(String),
    FieldInvalid(String, ComponentFieldDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentFieldDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    MutableButNoGetter(String),
    SerializableButNoSerializer(String),
    DeserializableButNoDeserializer(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetFieldDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    GetterIsNull(String),
}

impl From<StringError> for PluginDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl From<StringError> for ComponentDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl From<StringError> for ComponentFieldDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl From<StringError> for AssetFieldDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl<N> From<(N, ComponentFieldDefinitionError)> for ComponentDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, ComponentFieldDefinitionError)) -> Self {
        Self::FieldInvalid(name.into(), error)
    }
}

impl<N> From<(N, ComponentDefinitionError)> for PluginDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, ComponentDefinitionError)) -> Self {
        Self::ComponentInvalid(name.into(), error)
    }
}

impl Display for PluginDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("plugin name is null"),
            Self::NameIsNotUtf8 => f.write_str("plugin name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("plugin name is empty"),
            Self::EngineVersionMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "plugin '{name}' targets WasserXR {actual}, but the current engine is {expected}"
            ),
            Self::ComponentsIsNull(name) => {
                write!(f, "plugin '{name}' component list is null")
            }
            Self::ComponentInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid component: {error}")
            }
        }
    }
}

impl Error for PluginDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentInvalid(_, error) => Some(error),
            _ => None,
        }
    }
}

impl Display for ComponentDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("component name is null"),
            Self::NameIsNotUtf8 => f.write_str("component name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("component name is empty"),
            Self::CreatorIsNull(name) => write!(f, "component '{name}' creator is null"),
            Self::DestroyerIsNull(name) => write!(f, "component '{name}' destroyer is null"),
            Self::FieldsIsNull(name) => write!(f, "component '{name}' field list is null"),
            Self::FieldInvalid(name, error) => {
                write!(f, "component '{name}' has an invalid field: {error}")
            }
        }
    }
}

impl Error for ComponentDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldInvalid(_, error) => Some(error),
            _ => None,
        }
    }
}

impl Display for ComponentFieldDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("component field name is null"),
            Self::NameIsNotUtf8 => f.write_str("component field name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("component field name is empty"),
            Self::MutableButNoGetter(name) => {
                write!(f, "mutable component field '{name}' has no getter")
            }
            Self::SerializableButNoSerializer(name) => {
                write!(f, "serializable component field '{name}' has no serializer")
            }
            Self::DeserializableButNoDeserializer(name) => {
                write!(
                    f,
                    "deserializable component field '{name}' has no deserializer"
                )
            }
        }
    }
}

impl Error for ComponentFieldDefinitionError {}

impl Display for AssetFieldDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("asset field name is null"),
            Self::NameIsNotUtf8 => f.write_str("asset field name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("asset field name is empty"),
            Self::GetterIsNull(name) => write!(f, "asset field '{name}' has no getter"),
        }
    }
}

impl Error for AssetFieldDefinitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_nested_definition_errors() {
        let error = PluginDefinitionError::ComponentInvalid(
            "example".to_owned(),
            ComponentDefinitionError::FieldInvalid(
                "Transform".to_owned(),
                ComponentFieldDefinitionError::MutableButNoGetter("position".to_owned()),
            ),
        );

        assert_eq!(
            error.to_string(),
            "plugin 'example' has an invalid component: component 'Transform' has an invalid field: mutable component field 'position' has no getter"
        );
        assert!(error.source().is_some());
    }

    #[test]
    fn converts_nested_errors_with_context() {
        let component_error: ComponentDefinitionError =
            ("Transform", ComponentFieldDefinitionError::NameIsNull).into();
        let plugin_error: PluginDefinitionError = ("example", component_error).into();

        assert_eq!(
            plugin_error.to_string(),
            "plugin 'example' has an invalid component: component 'Transform' has an invalid field: component field name is null"
        );
    }

    #[test]
    fn converts_component_errors_with_explicit_plugin_context() {
        let plugin_error: PluginDefinitionError = (
            "example",
            ComponentDefinitionError::FieldsIsNull("Transform".to_owned()),
        )
            .into();
        assert_eq!(
            plugin_error,
            PluginDefinitionError::ComponentInvalid(
                "example".to_owned(),
                ComponentDefinitionError::FieldsIsNull("Transform".to_owned()),
            )
        );
    }
}
