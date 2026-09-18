use std::{
    ffi::c_void,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    definitions::components::Destroyer,
    errors::ComponentError,
    field::AccessRequest,
    ids::{ComponentTypeID, FieldSlot, FieldTypeID},
    private::{
        fields::ComponentField, id_store::IDStore, manifests::components::ComponentManifest,
    },
};

/// Concrete component data and its fields.
#[derive(Debug)]
pub(crate) struct Component {
    name: String,
    fields: IDStore<FieldTypeID, FieldSlot, ComponentField>,
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
    pub(crate) fn new(manifest: &ComponentManifest, component_type: ComponentTypeID) -> Self {
        let data = unsafe { (manifest.creator)() };
        let mut fields = IDStore::default();
        for (field_slot, field) in manifest.fields.iter() {
            fields.insert_named(
                FieldTypeID(component_type.0, component_type.1, field_slot),
                ComponentField::from(field),
            );
        }
        Self {
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
    pub(crate) fn get_field_name(&self, id: FieldSlot) -> Result<String, ComponentError> {
        Ok(self
            .fields
            .get(id)
            .ok_or(ComponentError::FieldNotFound)?
            .get_name()
            .to_owned())
    }

    /// Get a component-local field slot from its field type ID.
    pub(crate) fn resolve_field_slot(
        &self,
        field_type_id: FieldTypeID,
    ) -> Result<FieldSlot, ComponentError> {
        self.fields
            .resolve_id(&field_type_id)
            .ok_or(ComponentError::FieldNotFound)
    }

    /// Resolves one field pointer under the caller's component lock.
    ///
    /// # Design decisions
    ///
    /// Access is checked per query entry, even if another entry upgraded the
    /// component's lock to exclusive. This allows immutable fields to be read
    /// alongside writes to other fields of the same component.
    pub(crate) fn query_field(
        &self,
        field_type: FieldTypeID,
        access: AccessRequest,
    ) -> Result<*mut c_void, ComponentError> {
        let field = self
            .fields
            .resolve_id(&field_type)
            .and_then(|field| self.fields.get(field))
            .ok_or(ComponentError::FieldNotFound)?;
        match access {
            AccessRequest::Read => Ok(field.get(self.data)?.cast_mut()),
            AccessRequest::Write => Ok(field.get_mut(self.data)?),
        }
    }
}

/// Owns either kind of component lock until the query callback finishes.
pub(crate) enum ComponentGuard<'a> {
    Read(RwLockReadGuard<'a, Component>),
    Write(RwLockWriteGuard<'a, Component>),
}

impl<'a> ComponentGuard<'a> {
    /// Acquires the requested lock; the caller must enforce global ordering.
    ///
    /// # Design decisions
    ///
    /// An enum keeps standard-library guards of both kinds in one collection.
    /// Their normal drop behavior releases locks on success, error, or unwind.
    pub(crate) fn lock(component: &'a RwLock<Component>, access: AccessRequest) -> Self {
        match access {
            AccessRequest::Read => Self::Read(component.read().expect("component lock poisoned")),
            AccessRequest::Write => {
                Self::Write(component.write().expect("component lock poisoned"))
            }
        }
    }

    /// Borrows the locked record to resolve its opaque field pointers.
    ///
    /// # Design decisions
    ///
    /// The record's metadata is immutable in both modes. Mutation of plugin
    /// data happens only through field pointers inside the query callback.
    pub(crate) fn component(&self) -> &Component {
        match self {
            Self::Read(component) => component,
            Self::Write(component) => component,
        }
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
