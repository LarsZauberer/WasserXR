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
    DuplicateComponentName(String),
    ComponentInvalid(String, ComponentDefinitionError),
    AssetsIsNull(String),
    DuplicateAssetName(String),
    AssetInvalid(String, AssetDefinitionError),
    SystemsIsNull(String),
    DuplicateSystemName(String),
    SystemInvalid(String, SystemDefinitionError),
    FunctionsIsNull(String),
    DuplicateFunctionName(String),
    FunctionInvalid(String, FunctionDefinitionError),
}

/// Errors found while validating a raw global function definition.
#[derive(Debug, PartialEq, Eq)]
pub enum FunctionDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    FunctionIsNull(String),
}

/// Errors found while converting a function definition into a manifest.
#[derive(Debug, PartialEq, Eq)]
pub enum FunctionManifestError {
    DefinitionInvalid(FunctionDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    CreatorIsNull(String),
    DestroyerIsNull(String),
    FieldsIsNull(String),
    DuplicateFieldName(String),
    FieldInvalid(String, ComponentFieldDefinitionError),
    MethodsIsNull(String),
    DuplicateMethodName(String),
    MethodInvalid(String, MethodDefinitionError),
}

/// Errors found while validating a component method definition.
#[derive(Debug, PartialEq, Eq)]
pub enum MethodDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    MethodIsNull(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentFieldDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    /// The contained `u32` is the unrecognized
    /// [`TypeHint`](super::fields::TypeHint) discriminant received through the
    /// C ABI.
    InvalidTypeHint(u32),
    MutableButNoGetter(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssetFieldDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    /// The contained `u32` is the unrecognized
    /// [`TypeHint`](super::fields::TypeHint) discriminant received through the
    /// C ABI.
    InvalidTypeHint(u32),
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
    DuplicateFieldName(String),
    FieldInvalid(String, AssetFieldDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeIDRequestError {
    Component(StringError),
    Field(StringError),
    Asset(StringError),
    Function(StringError),
}

impl From<StringError> for FunctionDefinitionError {
    fn from(error: StringError) -> Self {
        match error {
            StringError::Null => Self::NameIsNull,
            StringError::NotUtf8 => Self::NameIsNotUtf8,
            StringError::Empty => Self::NameIsEmpty,
        }
    }
}

impl<N> From<(N, FunctionDefinitionError)> for PluginDefinitionError
where
    N: Into<String>,
{
    fn from((name, error): (N, FunctionDefinitionError)) -> Self {
        Self::FunctionInvalid(name.into(), error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemDefinitionError {
    NameIsNull,
    NameIsNotUtf8,
    NameIsEmpty,
    RunnerIsNull(String),
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

impl From<StringError> for MethodDefinitionError {
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
            Self::DuplicateComponentName(name) => {
                write!(f, "plugin contains duplicate component name '{name}'")
            }
            Self::ComponentInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid component: {error}")
            }
            Self::AssetsIsNull(name) => write!(f, "plugin '{name}' asset list is null"),
            Self::DuplicateAssetName(name) => {
                write!(f, "plugin contains duplicate asset name '{name}'")
            }
            Self::AssetInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid asset: {error}")
            }
            Self::SystemsIsNull(name) => write!(f, "plugin '{name}' system list is null"),
            Self::DuplicateSystemName(name) => {
                write!(f, "plugin contains duplicate system name '{name}'")
            }
            Self::SystemInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid system: {error}")
            }
            Self::FunctionsIsNull(name) => write!(f, "plugin '{name}' function list is null"),
            Self::DuplicateFunctionName(name) => {
                write!(f, "plugin contains duplicate function name '{name}'")
            }
            Self::FunctionInvalid(name, error) => {
                write!(f, "plugin '{name}' has an invalid function: {error}")
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
            Self::FunctionInvalid(_, error) => Some(error),
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
            Self::DuplicateFieldName(name) => {
                write!(f, "component contains duplicate field name '{name}'")
            }
            Self::FieldInvalid(name, error) => {
                write!(f, "component '{name}' has an invalid field: {error}")
            }
            Self::MethodsIsNull(name) => write!(f, "component '{name}' method list is null"),
            Self::DuplicateMethodName(name) => {
                write!(f, "component contains duplicate method name '{name}'")
            }
            Self::MethodInvalid(name, error) => {
                write!(f, "component '{name}' has an invalid method: {error}")
            }
        }
    }
}

impl Error for ComponentDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FieldInvalid(_, error) => Some(error),
            Self::MethodInvalid(_, error) => Some(error),
            _ => None,
        }
    }
}

impl Display for MethodDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("method name is null"),
            Self::NameIsNotUtf8 => f.write_str("method name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("method name is empty"),
            Self::MethodIsNull(name) => write!(f, "method '{name}' callback is null"),
        }
    }
}

impl Error for MethodDefinitionError {}

impl Display for ComponentFieldDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("component field name is null"),
            Self::NameIsNotUtf8 => f.write_str("component field name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("component field name is empty"),
            Self::InvalidTypeHint(value) => write!(f, "invalid field type hint {value}"),
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
            Self::InvalidTypeHint(value) => write!(f, "invalid field type hint {value}"),
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
            Self::DuplicateFieldName(name) => {
                write!(f, "asset contains duplicate field name '{name}'")
            }
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
            Self::Function(error) => ("function", error),
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
            Self::RunnerIsNull(name) => write!(f, "system '{name}' runner is null"),
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

impl Display for FunctionDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsNull => f.write_str("function name is null"),
            Self::NameIsNotUtf8 => f.write_str("function name is not valid UTF-8"),
            Self::NameIsEmpty => f.write_str("function name is empty"),
            Self::FunctionIsNull(name) => write!(f, "function '{name}' callback is null"),
        }
    }
}

impl Error for FunctionDefinitionError {}

impl Display for FunctionManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefinitionInvalid(error) => write!(f, "invalid function definition: {error}"),
        }
    }
}

impl Error for FunctionManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DefinitionInvalid(error) => Some(error),
        }
    }
}
