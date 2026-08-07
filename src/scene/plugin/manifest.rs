//! Validation and host-owned copying for plugin descriptor graphs.

mod error;

pub use error::ManifestError;

use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, c_char},
    mem::size_of,
    rc::Rc,
    slice,
};

use crate::scene::{
    assets::AssetType,
    component::{ComponentDefinition, FieldType},
    system::SystemDefinition,
};

use super::WXRPluginDescriptor;

pub(crate) struct ValidatedManifest {
    name: String,
    components: HashMap<String, Rc<ComponentDefinition>>,
    assets: HashMap<String, Rc<AssetType>>,
    systems: HashMap<String, Rc<SystemDefinition>>,
}

impl ValidatedManifest {
    pub(crate) fn new(
        name: String,
        components: HashMap<String, Rc<ComponentDefinition>>,
        assets: HashMap<String, Rc<AssetType>>,
        systems: HashMap<String, Rc<SystemDefinition>>,
    ) -> Self {
        Self {
            name,
            components,
            assets,
            systems,
        }
    }

    /// Copies and validates an entire process-lifetime descriptor graph.
    ///
    /// # Safety
    /// Every non-null pointer and count in the graph must describe readable,
    /// immutable process-lifetime storage. Function pointers must have their
    /// declared signatures and satisfy the callback contracts documented by
    /// [`crate::scene::Scene::load_plugin`].
    pub(crate) unsafe fn from_descriptor(
        descriptor: *const WXRPluginDescriptor,
    ) -> Result<Self, ManifestError> {
        let descriptor = unsafe { descriptor.as_ref() }.ok_or(ManifestError::NullDescriptor)?;
        unsafe { descriptor.validate() }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.name
    }

    pub(crate) fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.components
            .keys()
            .chain(self.assets.keys())
            .chain(self.systems.keys())
            .map(String::as_str)
    }

    pub(crate) fn component(&self, id: &str) -> Option<Rc<ComponentDefinition>> {
        self.components.get(id).cloned()
    }

    pub(crate) fn asset(&self, id: &str) -> Option<Rc<AssetType>> {
        self.assets.get(id).cloned()
    }

    pub(crate) fn system(&self, id: &str) -> Option<Rc<SystemDefinition>> {
        self.systems.get(id).cloned()
    }

    pub(crate) fn asset_names(&self) -> impl Iterator<Item = &str> {
        self.assets.keys().map(String::as_str)
    }

    pub(crate) fn is_consistent(&self) -> bool {
        self.components.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        }) && self.assets.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        }) && self.systems.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        })
    }
}

pub(crate) fn field_type(value: u32) -> Result<FieldType, ManifestError> {
    FieldType::try_from(value).map_err(ManifestError::UnknownFieldType)
}

pub(crate) fn register_definition(
    names: &mut HashSet<String>,
    name: &str,
) -> Result<(), ManifestError> {
    if names.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(ManifestError::DuplicateDefinition(name.to_owned()))
    }
}

pub(crate) fn register_local(
    names: &mut HashSet<String>,
    name: &str,
    kind: &'static str,
) -> Result<(), ManifestError> {
    if names.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(ManifestError::DuplicateName {
            kind,
            name: name.to_owned(),
        })
    }
}

pub(crate) fn missing_callback(kind: &str, name: &str, callback: &str) -> ManifestError {
    ManifestError::MissingCallback(format!("{kind} `{name}` {callback}"))
}

pub(crate) unsafe fn copy_name(
    pointer: *const c_char,
    kind: &'static str,
) -> Result<String, ManifestError> {
    if pointer.is_null() {
        return Err(ManifestError::NullName(kind));
    }
    let name = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map_err(|_| ManifestError::InvalidUtf8(kind))?;
    if name.is_empty() {
        return Err(ManifestError::EmptyName(kind));
    }
    Ok(name.to_owned())
}

pub(crate) unsafe fn descriptor_slice<'a, T>(
    pointer: *const T,
    count: usize,
    kind: &'static str,
) -> Result<&'a [T], ManifestError> {
    if count == 0 {
        return if pointer.is_null() {
            Ok(&[])
        } else {
            Err(ManifestError::InvalidPointerCount(kind))
        };
    }
    if pointer.is_null() || size_of::<T>() != 0 && count > isize::MAX as usize / size_of::<T>() {
        return Err(ManifestError::InvalidPointerCount(kind));
    }
    Ok(unsafe { slice::from_raw_parts(pointer, count) })
}

