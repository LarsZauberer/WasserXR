use std::ffi::c_void;

use crate::{
    definitions::components::Destroyer,
    errors::ComponentError,
    field::FieldAccess,
    ids::{FieldID, FieldTypeID, PluginID},
    private::{
        fields::ComponentField, id_store::IDStore, manifests::components::ComponentManifest,
    },
};

/// The component is the concrete data record of a component. It carries the
/// information about which plugin it belongs to, stores it's component manifest
/// and the actual data.
#[derive(Debug)]
pub(crate) struct Component {
    plugin_id: PluginID,
    name: String,
    fields: IDStore<FieldTypeID, FieldID, ComponentField>,
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
            fields.insert_named(field_type_id, ComponentField::from(field));
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

    /// Resolves requested field pointers while the caller holds this
    /// component's lock.
    pub(crate) fn query_fields(
        &self,
        requests: &[(FieldID, FieldAccess)],
    ) -> Result<Vec<(FieldID, *mut c_void)>, ComponentError> {
        debug_assert!(requests.windows(2).all(|fields| fields[0].0 < fields[1].0));
        let mut fields = Vec::with_capacity(requests.len());
        for (id, access) in requests {
            let field = self.fields.get(*id).ok_or(ComponentError::FieldNotFound)?;
            let pointer = match access {
                FieldAccess::Read => field.get(self.data)?.cast_mut(),
                FieldAccess::Write => field.get_mut(self.data)?,
            };
            fields.push((*id, pointer));
        }
        Ok(fields)
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
