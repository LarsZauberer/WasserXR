use std::{error::Error, ffi::NulError, fmt, io};

use super::manifest::ManifestError;

/// Recoverable plugin loading, validation, and lifecycle failures.
#[derive(Debug)]
pub enum PluginError {
    AlreadyLoaded(String),
    NotLoaded,
    DefinitionCollision(String),
    LoadIo(io::Error),
    Linking(String),
    MissingManifestSymbol,
    InvalidPath(NulError),
    InvalidManifest(ManifestError),
}

impl PartialEq for PluginError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AlreadyLoaded(left), Self::AlreadyLoaded(right))
            | (Self::DefinitionCollision(left), Self::DefinitionCollision(right))
            | (Self::Linking(left), Self::Linking(right)) => left == right,
            (Self::NotLoaded, Self::NotLoaded)
            | (Self::MissingManifestSymbol, Self::MissingManifestSymbol) => true,
            (Self::LoadIo(left), Self::LoadIo(right)) => {
                left.kind() == right.kind() && left.to_string() == right.to_string()
            }
            (Self::InvalidPath(left), Self::InvalidPath(right)) => left == right,
            (Self::InvalidManifest(left), Self::InvalidManifest(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyLoaded(name) => write!(f, "plugin `{name}` is already loaded"),
            Self::NotLoaded => f.write_str("plugin is not loaded"),
            Self::DefinitionCollision(name) => {
                write!(f, "definition `{name}` is already registered")
            }
            Self::LoadIo(error) => write!(f, "plugin file could not be copied: {error}"),
            Self::Linking(error) => write!(f, "plugin could not be linked: {error}"),
            Self::MissingManifestSymbol => f.write_str("plugin symbol `wxr_plugin` was not found"),
            Self::InvalidPath(error) => write!(f, "plugin path contains a null byte: {error}"),
            Self::InvalidManifest(error) => write!(f, "plugin manifest is invalid: {error}"),
        }
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LoadIo(error) => Some(error),
            Self::InvalidPath(error) => Some(error),
            Self::InvalidManifest(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ManifestError> for PluginError {
    fn from(error: ManifestError) -> Self {
        Self::InvalidManifest(error)
    }
}
