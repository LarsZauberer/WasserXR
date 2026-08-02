use std::{error::Error, fmt};

use crate::scene::plugin::PluginError;

/// Recoverable asset type, asset instance, and field failures.
#[derive(Debug, PartialEq, Eq)]
pub enum AssetError {
    FieldNotFound,
    FieldNoGetter,
    FieldParsing,
    InvalidAsset,
    NoCreator(PluginError),
    NoSchema(PluginError),
    NoDestroyer(PluginError),
    AssetTypeNotFound,
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldNotFound => f.write_str("asset field was not found"),
            Self::FieldNoGetter => f.write_str("asset field has no getter"),
            Self::FieldParsing => f.write_str("asset query fields do not match the requested type"),
            Self::InvalidAsset => f.write_str("asset creator returned an invalid asset"),
            Self::NoCreator(error) => write!(f, "asset creator could not be resolved: {error}"),
            Self::NoSchema(error) => write!(f, "asset schema could not be resolved: {error}"),
            Self::NoDestroyer(error) => write!(f, "asset destroyer could not be resolved: {error}"),
            Self::AssetTypeNotFound => f.write_str("no loaded plugin provides this asset type"),
        }
    }
}

impl Error for AssetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoCreator(error) | Self::NoSchema(error) | Self::NoDestroyer(error) => {
                Some(error)
            }
            _ => None,
        }
    }
}
