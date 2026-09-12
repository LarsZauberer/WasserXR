use std::{
    ffi::c_void,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    definitions::components::Destroyer,
    errors::ComponentError,
    field::{Field, FieldAccess},
    private::{
        fields::ComponentField, id_store::IDStore, manifests::components::ComponentManifest,
    },
    scene::{FieldID, PluginID},
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
/// [`Component::query_fields`] creates these after sorting requests by field
/// ID, then converts them to the public [`Field`] values passed to the
/// callback.
///
/// # Design decision
///
/// Read and write guards have different types, so an enum is required to keep
/// both in one collection. Retaining the guards here keeps every field locked
/// without exposing synchronization details in the public API.
enum LockedField<'a> {
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
    fn field(&self) -> Field {
        match self {
            Self::Read { id, pointer, .. } => Field::Read(*id, *pointer),
            Self::Write { id, pointer, .. } => Field::Write(*id, *pointer),
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
    /// Field is locked and carries the pointer with it.
    fields: IDStore<String, FieldID, RwLock<ComponentField>>,
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
        for field in manifest.fields.values() {
            let field = ComponentField::from(field);
            let name = field.get_name().to_owned();
            fields.insert_named(name, RwLock::new(field));
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

    /// Runs an action with a field lock resolved from its ID.
    fn with_field<'a, T>(
        &'a self,
        id: FieldID,
        action: impl FnOnce(&'a RwLock<ComponentField>) -> Result<T, ComponentError>,
    ) -> Result<T, ComponentError> {
        action(self.fields.get(id).ok_or(ComponentError::FieldNotFound)?)
    }

    /// Get the name of a field
    pub(crate) fn get_field_name(&self, id: FieldID) -> Result<String, ComponentError> {
        self.with_field(id, |field| {
            Ok(field
                .read()
                .expect("component field lock poisoned")
                .get_name()
                .to_owned())
        })
    }

    /// Get field id from the field name
    pub(crate) fn resolve_field_id(&self, name: &str) -> Result<FieldID, ComponentError> {
        self.fields
            .resolve_id(name)
            .ok_or(ComponentError::FieldNotFound)
    }

    /// Lock fields in ID order to prevent ordering deadlocks.
    pub(crate) fn query_fields<T>(
        &self,
        requests: &[(FieldID, FieldAccess)],
        action: impl FnOnce(&[Field]) -> T,
    ) -> Result<T, ComponentError> {
        let mut requests = requests.iter().copied().enumerate().collect::<Vec<_>>();
        requests.sort_by_key(|(_, (field, _))| *field);
        let mut locked = Vec::with_capacity(requests.len());
        for (position, (id, access)) in requests {
            let field = self.with_field(id, |field| {
                Ok(match access {
                    FieldAccess::Read => {
                        let guard = field.read().expect("component field lock poisoned");
                        let pointer = guard.get(self.data)?;
                        LockedField::Read {
                            id,
                            pointer,
                            _guard: guard,
                        }
                    }
                    FieldAccess::Write => {
                        let guard = field.write().expect("component field lock poisoned");
                        let pointer = guard.get_mut(self.data)?;
                        LockedField::Write {
                            id,
                            pointer,
                            _guard: guard,
                        }
                    }
                })
            })?;
            locked.push((position, field));
        }
        let mut fields = locked
            .iter()
            .map(|(position, field)| (*position, field.field()))
            .collect::<Vec<_>>();
        fields.sort_by_key(|(position, _)| *position);
        let fields = fields
            .into_iter()
            .map(|(_, field)| field)
            .collect::<Vec<_>>();
        Ok(action(&fields))
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
