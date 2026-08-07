use std::{error::Error, fmt};

use crate::scene::plugin::Version;

/// A rejected plugin descriptor graph.
#[derive(Debug, PartialEq, Eq)]
pub enum ManifestError {
    NullDescriptor,
    IncompatibleVersion { host: Version, plugin: Version },
    NullName(&'static str),
    InvalidUtf8(&'static str),
    EmptyName(&'static str),
    InvalidPointerCount(&'static str),
    UnknownFieldType(u32),
    MissingCallback(String),
    MutableWithoutGetter(String),
    DuplicateDefinition(String),
    DuplicateName { kind: &'static str, name: String },
    DuplicateEntityGroup(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullDescriptor => f.write_str("plugin descriptor pointer is null"),
            Self::IncompatibleVersion { host, plugin } => write!(
                f,
                "plugin version {}.{}.{} is incompatible with host {}.{}.{}",
                plugin.major, plugin.minor, plugin.patch, host.major, host.minor, host.patch
            ),
            Self::NullName(kind) => write!(f, "{kind} name pointer is null"),
            Self::InvalidUtf8(kind) => write!(f, "{kind} name is not valid UTF-8"),
            Self::EmptyName(kind) => write!(f, "{kind} name is empty"),
            Self::InvalidPointerCount(kind) => write!(f, "{kind} pointer/count pair is invalid"),
            Self::UnknownFieldType(value) => write!(f, "field type code `{value}` is unknown"),
            Self::MissingCallback(callback) => {
                write!(f, "required callback is missing: {callback}")
            }
            Self::MutableWithoutGetter(field) => {
                write!(f, "mutable component field `{field}` has no getter")
            }
            Self::DuplicateDefinition(name) => write!(f, "definition `{name}` is duplicated"),
            Self::DuplicateName { kind, name } => write!(f, "{kind} name `{name}` is duplicated"),
            Self::DuplicateEntityGroup(system) => {
                write!(f, "system `{system}` contains duplicate entity groups")
            }
        }
    }
}

impl Error for ManifestError {}
