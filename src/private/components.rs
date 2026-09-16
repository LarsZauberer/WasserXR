use std::{
    ffi::c_void,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    definitions::components::Destroyer,
    errors::ComponentError,
    field::FieldAccess,
    ids::{FieldID, FieldTypeID, PluginID},
    private::{
        fields::ComponentField, id_store::IDStore, manifests::components::ComponentManifest,
    },
};

/// Keeps a queried field pointer and its lock guard alive together.
///
/// # Purpose
///
/// A query may contain both shared and exclusive field access. Each variant
/// stores the matching pointer and guard until the query callback finishes.
///
/// # Usage
///
/// [`Component::lock_fields`] creates these in the global lock order. The
/// entity exposes their raw pointers only while the guards remain alive.
///
/// # Design decision
///
/// Read and write guards have different types, so an enum is required to keep
/// both in one collection. Retaining the guards here keeps every field locked
/// without exposing synchronization details in the public API.
pub(crate) enum LockedField<'a> {
    Read {
        id: FieldID,
        pointer: *const c_void,
        _guard: RwLockReadGuard<'a, ComponentField>,
    },
    Write {
        id: FieldID,
        pointer: *mut c_void,
        _guard: RwLockWriteGuard<'a, ComponentField>,
    },
}

impl LockedField<'_> {
    pub(crate) fn field(&self) -> (FieldID, *mut c_void) {
        match self {
            Self::Read { id, pointer, .. } => (*id, pointer.cast_mut()),
            Self::Write { id, pointer, .. } => (*id, *pointer),
        }
    }
}

/// The component is the concrete data record of a component. It carries the
/// information about which plugin it belongs to, stores it's component manifest
/// and the actual data.
#[derive(Debug)]
pub(crate) struct Component {
    plugin_id: PluginID,
    name: String,
    /// # Design Decision
    ///
    /// The [`ComponentField`] requires an [`RwLock`] to make control the access
    /// to the field pointers. From a concrete [`ComponentField`] or more
    /// precisely a [`RwLockWriteGuard<'a, ComponentField>`] a
    /// [`LockedField<'a>`] is then created that encompasses the state that the
    /// field is locked and carries the pointer with it.
    fields: IDStore<FieldTypeID, FieldID, RwLock<ComponentField>>,
    destroyer: Destroyer,
    data: *mut c_void,
}

// SAFETY: Component access is synchronized by its owning collection lock.
// Plugin creators, getters, and destroyers must uphold the thread-safety
// contract of their opaque data pointer.
unsafe impl Send for Component {}
unsafe impl Sync for Component {}

impl Component {
    /// Creates a new component. This function will run the creator of the
    /// component to generate allocate the data.
    ///
    /// The destroyer and all other required function pointers will be saved in
    /// this concrete implementation.
    ///
    /// The destroyer is especially important since it is used during drop of
    /// the component to deallocate the user's allocated data.
    ///
    /// This **requires** that the destroyer code is still loaded by the plugin
    /// at the time the component is dropped.
    pub(crate) fn new(manifest: &ComponentManifest, plugin_id: PluginID) -> Self {
        let data = unsafe { (manifest.creator)() };
        let mut fields = IDStore::default();
        for (field_type_id, field) in manifest.fields.iter() {
            let field = ComponentField::from(field);
            fields.insert_named(field_type_id, RwLock::new(field));
        }
        Self {
            plugin_id,
            name: manifest.name.clone(),
            fields,
            destroyer: manifest.destroyer,
            data,
        }
    }

    /// Get the name of the component
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the name of a field
    pub(crate) fn get_field_name(&self, id: FieldID) -> Result<String, ComponentError> {
        Ok(self
            .fields
            .get(id)
            .ok_or(ComponentError::FieldNotFound)?
            .read()
            .expect("component field lock poisoned")
            .get_name()
            .to_owned())
    }

    /// Get a field ID from its field type ID.
    pub(crate) fn resolve_field_id(
        &self,
        field_type_id: FieldTypeID,
    ) -> Result<FieldID, ComponentError> {
        self.fields
            .resolve_id(&field_type_id)
            .ok_or(ComponentError::FieldNotFound)
    }

    /// Locks fields in the supplied global lock order. Requests must be sorted
    /// by field ID and contain no duplicates.
    pub(crate) fn lock_fields(
        &self,
        requests: &[(FieldID, FieldAccess)],
    ) -> Result<Vec<LockedField<'_>>, ComponentError> {
        debug_assert!(requests.windows(2).all(|fields| fields[0].0 < fields[1].0));
        let mut locked = Vec::with_capacity(requests.len());
        for (id, access) in requests {
            let field = self.fields.get(*id).ok_or(ComponentError::FieldNotFound)?;
            locked.push(match access {
                FieldAccess::Read => {
                    let guard = field.read().expect("component field lock poisoned");
                    let pointer = guard.get(self.data)?;
                    LockedField::Read {
                        id: *id,
                        pointer,
                        _guard: guard,
                    }
                }
                FieldAccess::Write => {
                    let guard = field.write().expect("component field lock poisoned");
                    let pointer = guard.get_mut(self.data)?;
                    LockedField::Write {
                        id: *id,
                        pointer,
                        _guard: guard,
                    }
                }
            });
        }
        Ok(locked)
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
