use std::{error::Error, ffi::NulError, fmt, io};

/// Recoverable plugin loading, lookup, and lifecycle failures.
#[derive(Debug)]
pub enum PluginError {
    AlreadyLoaded,
    NotLoaded,
    StaticPluginCannotUnload,
    LoadIo(io::Error),
    Linking(String),
    MissingSymbol(String),
    InvalidSymbol(NulError),
}

impl PartialEq for PluginError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AlreadyLoaded, Self::AlreadyLoaded)
            | (Self::NotLoaded, Self::NotLoaded)
            | (Self::StaticPluginCannotUnload, Self::StaticPluginCannotUnload) => true,
            (Self::LoadIo(left), Self::LoadIo(right)) => {
                left.kind() == right.kind() && left.to_string() == right.to_string()
            }
            (Self::Linking(left), Self::Linking(right))
            | (Self::MissingSymbol(left), Self::MissingSymbol(right)) => left == right,
            (Self::InvalidSymbol(left), Self::InvalidSymbol(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyLoaded => f.write_str("plugin is already loaded"),
            Self::NotLoaded => f.write_str("plugin is not loaded"),
            Self::StaticPluginCannotUnload => f.write_str("the static plugin cannot be unloaded"),
            Self::LoadIo(error) => write!(f, "plugin file could not be copied: {error}"),
            Self::Linking(error) => write!(f, "plugin could not be linked: {error}"),
            Self::MissingSymbol(symbol) => write!(f, "plugin symbol `{symbol}` was not found"),
            Self::InvalidSymbol(error) => write!(f, "plugin symbol contains a null byte: {error}"),
        }
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LoadIo(error) => Some(error),
            Self::InvalidSymbol(error) => Some(error),
            _ => None,
        }
    }
}
