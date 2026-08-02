//! C ABI bindings for WasserXR.

pub mod logging;
pub mod scene;
pub mod schema;
pub mod utils;

use std::{
    cell::Cell,
    ffi::{CStr, CString, c_char},
};

use crate::scene::{
    SceneError, assets::AssetError, component::ComponentError, entity::EntityError,
    plugin::PluginError, resource::ResourceError, serialization::SerializationError,
    system::SystemError,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// C ABI error code for the latest failed WasserXR binding call.
pub enum WXRSceneError {
    /// No error has been recorded for this thread.
    NoError,
    /// A required pointer argument was null.
    NullPointer,
    /// A C string argument was null or not valid UTF-8.
    InvalidString,
    /// The entity id is not present in the scene.
    EntityNotFound,
    /// The entity already has a component with this id.
    ComponentAlreadyExists,
    /// A resource with this name is already registered.
    ResourceAlreadyExists,
    /// A system with this id is already registered.
    SystemAlreadyExists,
    /// A plugin with this path is already loaded.
    PluginAlreadyLoaded,
    /// The system id is not present in the scene.
    SystemNotFound,
    /// The resource name is not present in the scene.
    ResourceNotFound,
    /// The plugin id or path is not present in the scene.
    PluginNotFound,
    /// The built-in static plugin cannot be unloaded.
    StaticPluginUnload,
    /// The component id is not present on the requested entity.
    ComponentNotFound,
    /// The owning plugin does not export the requested method symbol.
    MethodNotFound,
    /// A component field operation failed.
    ComponentFieldError,
    /// An asset operation failed.
    AssetError,
    /// Loading a plugin failed.
    PluginLoading,
    /// A system could not be created from any loaded plugin.
    SystemCreation,
    /// Component symbols could not be resolved from any loaded plugin.
    ComponentCreation,
    /// The component creator returned a null pointer.
    ComponentCreatorFailed,
    /// Scene serialization failed.
    Serialization,
    /// Scene deserialization failed.
    Deserialization,
    /// Reading or writing a scene file failed.
    FileIo,
}

thread_local! {
    static LAST_ERROR: Cell<WXRSceneError> = const { Cell::new(WXRSceneError::NoError) };
}

/// Returns the latest error recorded by a WasserXR C binding on this thread.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_error() -> WXRSceneError {
    LAST_ERROR.with(Cell::get)
}

pub(crate) fn clear_error() {
    LAST_ERROR.with(|error| error.set(WXRSceneError::NoError));
}

pub(crate) fn set_error(error: WXRSceneError) {
    LAST_ERROR.with(|slot| slot.set(error));
}

pub(crate) fn set_scene_error(error: SceneError) {
    set_error(error.into());
}

pub(crate) fn result_code(result: Result<(), SceneError>) -> i32 {
    match result {
        Ok(()) => {
            clear_error();
            0
        }
        Err(error) => {
            set_scene_error(error);
            -1
        }
    }
}

pub(crate) unsafe fn str_from_ptr<'a>(ptr: *const c_char) -> Result<&'a str, WXRSceneError> {
    if ptr.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| WXRSceneError::InvalidString)
}

pub(crate) fn string_to_ptr(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a C string returned by WasserXR bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_free_string(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

impl From<SceneError> for WXRSceneError {
    fn from(value: SceneError) -> Self {
        match value {
            SceneError::Entity(error) => error.into(),
            SceneError::Component(error) => error.into(),
            SceneError::Resource(error) => error.into(),
            SceneError::System(error) => error.into(),
            SceneError::Plugin(error) => error.into(),
            SceneError::Asset(error) => error.into(),
            SceneError::Serialization(error) => error.into(),
            SceneError::Io(_) => Self::FileIo,
        }
    }
}

impl From<EntityError> for WXRSceneError {
    fn from(_: EntityError) -> Self {
        Self::EntityNotFound
    }
}

impl From<ComponentError> for WXRSceneError {
    fn from(value: ComponentError) -> Self {
        match value {
            ComponentError::AlreadyExists => Self::ComponentAlreadyExists,
            ComponentError::NotFound => Self::ComponentNotFound,
            ComponentError::MethodNotFound => Self::MethodNotFound,
            ComponentError::TypeNotFound
            | ComponentError::NoCreator(_)
            | ComponentError::NoDestroyer(_) => Self::ComponentCreation,
            ComponentError::CreatorFailed => Self::ComponentCreatorFailed,
            ComponentError::FieldNotFound
            | ComponentError::FieldNoGetter
            | ComponentError::FieldNotMutable
            | ComponentError::FieldNoSerializer
            | ComponentError::FieldNoDeserializer
            | ComponentError::FieldParsing
            | ComponentError::FieldValueParsing => Self::ComponentFieldError,
        }
    }
}

impl From<AssetError> for WXRSceneError {
    fn from(_: AssetError) -> Self {
        Self::AssetError
    }
}

impl From<PluginError> for WXRSceneError {
    fn from(value: PluginError) -> Self {
        match value {
            PluginError::AlreadyLoaded => Self::PluginAlreadyLoaded,
            PluginError::NotLoaded => Self::PluginNotFound,
            PluginError::StaticPluginCannotUnload => Self::StaticPluginUnload,
            PluginError::LoadIo(_)
            | PluginError::Linking(_)
            | PluginError::MissingSymbol(_)
            | PluginError::InvalidSymbol(_) => Self::PluginLoading,
        }
    }
}

impl From<ResourceError> for WXRSceneError {
    fn from(value: ResourceError) -> Self {
        match value {
            ResourceError::AlreadyExists => Self::ResourceAlreadyExists,
            ResourceError::NotFound => Self::ResourceNotFound,
        }
    }
}

impl From<SystemError> for WXRSceneError {
    fn from(value: SystemError) -> Self {
        match value {
            SystemError::AlreadyExists => Self::SystemAlreadyExists,
            SystemError::NotFound => Self::SystemNotFound,
            SystemError::TypeNotFound | SystemError::NoRunner(_) => Self::SystemCreation,
        }
    }
}

impl From<SerializationError> for WXRSceneError {
    fn from(value: SerializationError) -> Self {
        match value {
            SerializationError::Encode(_) => Self::Serialization,
            SerializationError::InvalidHeader
            | SerializationError::MissingVersion
            | SerializationError::UnsupportedVersion(_)
            | SerializationError::Decode(_)
            | SerializationError::TrailingBytes => Self::Deserialization,
        }
    }
}
