use std::{error::Error, fmt};

use super::{
    assets::AssetError, component::ComponentError, entity::EntityError, plugin::PluginError,
    resource::ResourceError, serialization::SerializationError, system::SystemError,
};

/// Errors returned by high-level [`super::Scene`] operations.
#[derive(Debug)]
pub enum SceneError {
    Entity(EntityError),
    Component(ComponentError),
    Resource(ResourceError),
    System(SystemError),
    Plugin(PluginError),
    Asset(AssetError),
    Serialization(SerializationError),
    Io(std::io::Error),
}

impl PartialEq for SceneError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Entity(left), Self::Entity(right)) => left == right,
            (Self::Component(left), Self::Component(right)) => left == right,
            (Self::Resource(left), Self::Resource(right)) => left == right,
            (Self::System(left), Self::System(right)) => left == right,
            (Self::Plugin(left), Self::Plugin(right)) => left == right,
            (Self::Asset(left), Self::Asset(right)) => left == right,
            (Self::Serialization(left), Self::Serialization(right)) => left == right,
            (Self::Io(left), Self::Io(right)) => {
                left.kind() == right.kind() && left.to_string() == right.to_string()
            }
            _ => false,
        }
    }
}

impl Eq for SceneError {}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity(error) => write!(f, "entity operation failed: {error}"),
            Self::Component(error) => write!(f, "component operation failed: {error}"),
            Self::Resource(error) => write!(f, "resource operation failed: {error}"),
            Self::System(error) => write!(f, "system operation failed: {error}"),
            Self::Plugin(error) => write!(f, "plugin operation failed: {error}"),
            Self::Asset(error) => write!(f, "asset operation failed: {error}"),
            Self::Serialization(error) => write!(f, "scene serialization failed: {error}"),
            Self::Io(error) => write!(f, "scene file I/O failed: {error}"),
        }
    }
}

impl Error for SceneError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Entity(error) => error,
            Self::Component(error) => error,
            Self::Resource(error) => error,
            Self::System(error) => error,
            Self::Plugin(error) => error,
            Self::Asset(error) => error,
            Self::Serialization(error) => error,
            Self::Io(error) => error,
        })
    }
}

macro_rules! scene_error_from {
    ($error:ty, $variant:ident) => {
        impl From<$error> for SceneError {
            fn from(error: $error) -> Self {
                Self::$variant(error)
            }
        }
    };
}

scene_error_from!(EntityError, Entity);
scene_error_from!(ComponentError, Component);
scene_error_from!(ResourceError, Resource);
scene_error_from!(SystemError, System);
scene_error_from!(PluginError, Plugin);
scene_error_from!(AssetError, Asset);
scene_error_from!(SerializationError, Serialization);
scene_error_from!(std::io::Error, Io);
