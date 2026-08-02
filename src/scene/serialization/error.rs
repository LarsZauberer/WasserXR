use std::{error::Error, fmt};

/// Recoverable scene encoding and decoding failures.
#[derive(Debug)]
pub enum SerializationError {
    InvalidHeader,
    MissingVersion,
    UnsupportedVersion(u32),
    Encode(bincode::error::EncodeError),
    Decode(bincode::error::DecodeError),
    TrailingBytes,
}

impl PartialEq for SerializationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidHeader, Self::InvalidHeader)
            | (Self::MissingVersion, Self::MissingVersion)
            | (Self::TrailingBytes, Self::TrailingBytes) => true,
            (Self::UnsupportedVersion(left), Self::UnsupportedVersion(right)) => left == right,
            (Self::Encode(left), Self::Encode(right)) => left.to_string() == right.to_string(),
            (Self::Decode(left), Self::Decode(right)) => left.to_string() == right.to_string(),
            _ => false,
        }
    }
}

impl Eq for SerializationError {}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => f.write_str("invalid scene serialization header"),
            Self::MissingVersion => f.write_str("missing scene serialization version"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported scene serialization version `{version}`")
            }
            Self::Encode(error) => write!(f, "scene encoding failed: {error}"),
            Self::Decode(error) => write!(f, "scene decoding failed: {error}"),
            Self::TrailingBytes => f.write_str("trailing bytes after scene serialization data"),
        }
    }
}

impl Error for SerializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Decode(error) => Some(error),
            _ => None,
        }
    }
}

impl From<bincode::error::EncodeError> for SerializationError {
    fn from(error: bincode::error::EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<bincode::error::DecodeError> for SerializationError {
    fn from(error: bincode::error::DecodeError) -> Self {
        Self::Decode(error)
    }
}
