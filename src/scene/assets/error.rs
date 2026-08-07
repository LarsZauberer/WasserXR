use std::{error::Error, fmt};

/// Recoverable asset type, asset instance, and field failures.
#[derive(Debug, PartialEq, Eq)]
pub enum AssetError {
    FieldNotFound,
    FieldNoGetter,
    FieldParsing,
    InvalidAsset,
    AssetTypeNotFound,
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldNotFound => f.write_str("asset field was not found"),
            Self::FieldNoGetter => f.write_str("asset field has no getter"),
            Self::FieldParsing => f.write_str("asset query fields do not match the requested type"),
            Self::InvalidAsset => f.write_str("asset creator returned an invalid asset"),
            Self::AssetTypeNotFound => f.write_str("no loaded plugin provides this asset type"),
        }
    }
}

impl Error for AssetError {}
