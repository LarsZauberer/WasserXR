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
    AssetsIsNull(String),
    AssetInvalid(String, AssetDefinitionError),
    SystemsIsNull(String),
    SystemInvalid(String, SystemDefinitionError),
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
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetFieldDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    GetterIsNull(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    CreatorIsNull(String),
    DestroyerIsNull(String),
    FieldsIsNull(String),
    FieldInvalid(String, AssetFieldDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeIDRequestError {
    Component(StringError),
    Field(StringError),
    Asset(StringError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    RequiresIsNull(String),
    RequiredSystemInvalid(String, StringError),
    WantedByIsNull(String),
    WantedBySystemInvalid(String, StringError),
    TypeIDRequestsIsNull(String),
    TypeIDRequestInvalid(String, TypeIDRequestError),
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

impl From<StringError> for AssetDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl From<StringError> for SystemDefinitionError {
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

impl<N> From<(N, AssetDefinitionError)> for PluginDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, AssetDefinitionError)) -> Self {
        Self::AssetInvalid(name.into(), error)
    }
}

impl<N> From<(N, SystemDefinitionError)> for PluginDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, SystemDefinitionError)) -> Self {
        Self::SystemInvalid(name.into(), error)
    }
}

impl<N> From<(N, AssetFieldDefinitionError)> for AssetDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, AssetFieldDefinitionError)) -> Self {
        Self::FieldInvalid(name.into(), error)
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
            Self::AssetsIsNull(name) => write!(f, "plugin '{name}' asset list is null"),
            Self::AssetInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid asset: {error}")
            }
            Self::SystemsIsNull(name) => write!(f, "plugin '{name}' system list is null"),
            Self::SystemInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid system: {error}")
            }
        }
    }
}

impl Error for PluginDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentInvalid(_, error) => Some(error),
            Self::AssetInvalid(_, error) => Some(error),
            Self::SystemInvalid(_, error) => Some(error),
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

impl Display for AssetDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("asset name is null"),
            Self::NameIsNotUtf8 => f.write_str("asset name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("asset name is empty"),
            Self::CreatorIsNull(name) => write!(f, "asset '{name}' creator is null"),
            Self::DestroyerIsNull(name) => write!(f, "asset '{name}' destroyer is null"),
            Self::FieldsIsNull(name) => write!(f, "asset '{name}' field list is null"),
            Self::FieldInvalid(name, error) => {
                write!(f, "asset '{name}' has an invalid field: {error}")
            }
        }
    }
}

impl Error for AssetDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldInvalid(_, error) => Some(error),
            _ => None,
        }
    }
}

fn string_error(error: &StringError) -> &'static str {
    match error {
        StringError::Null => "is null",
        StringError::NotUtf8 => "is not valid UTF-8",
        StringError::Empty => "is empty",
    }
}

impl Display for TypeIDRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, error) = match self {
            Self::Component(error) => ("component", error),
            Self::Field(error) => ("field", error),
            Self::Asset(error) => ("asset", error),
        };
        write!(f, "{name} name {}", string_error(error))
    }
}

impl Error for TypeIDRequestError {}

impl Display for SystemDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("system name is null"),
            Self::NameIsNotUtf8 => f.write_str("system name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("system name is empty"),
            Self::RequiresIsNull(name) => {
                write!(f, "system '{name}' requires list is null")
            }
            Self::RequiredSystemInvalid(name, error) => write!(
                f,
                "system '{name}' has a required system name that {}",
                string_error(error)
            ),
            Self::WantedByIsNull(name) => {
                write!(f, "system '{name}' wanted-by list is null")
            }
            Self::WantedBySystemInvalid(name, error) => write!(
                f,
                "system '{name}' has a wanted-by system name that {}",
                string_error(error)
            ),
            Self::TypeIDRequestsIsNull(name) => {
                write!(f, "system '{name}' type ID request list is null")
            }
            Self::TypeIDRequestInvalid(name, error) => {
                write!(f, "system '{name}' has an invalid type ID request: {error}")
            }
        }
    }
}

impl Error for SystemDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypeIDRequestInvalid(_, error) => Some(error),
            _ => None,
        }
    }
}
