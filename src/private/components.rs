use std::{collections::HashMap, ffi::c_void};

use slotmap::SlotMap;

use crate::{
    definitions::components::Destroyer,
    errors::ComponentError,
    private::{fields::ComponentField, manifests::components::ComponentManifest},
    scene::{FieldID, PluginID},
};

/// The component is the concrete data record of a component. It carries the
/// information about which plugin it belongs to, stores it's component manifest
/// and the actual data.
#[derive(Debug)]
pub(crate) struct Component {
    plugin_id: PluginID,
    name: String,
    fields: SlotMap<FieldID, ComponentField>,
    field_ids: HashMap<String, FieldID>,
    destroyer: Destroyer,
    data: *mut c_void,
}

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
        let mut fields = SlotMap::with_key();
        let mut field_ids = HashMap::new();
        manifest.fields.iter().for_each(|(_, field)| {
            let field = ComponentField::from(field);
            let name = field.get_name().to_owned();
            let id = fields.insert(field);
            field_ids.insert(name, id);
        });
        Self {
            plugin_id,
            name: manifest.name.clone(),
            fields,
            field_ids,
            destroyer: manifest.destroyer,
            data,
        }
    }

    /// Get the name of the component
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    /// Get a field from its id.
    pub(crate) fn get_field(&self, id: FieldID) -> Result<&ComponentField, ComponentError> {
        self.fields.get(id).ok_or(ComponentError::FieldNotFound)
    }

    /// Get the name of a field
    pub(crate) fn get_field_name(&self, id: FieldID) -> Result<&str, ComponentError> {
        Ok(self.get_field(id)?.get_name())
    }

    /// Get field id from the field name
    pub(crate) fn resolve_field_id(&self, name: &str) -> Result<FieldID, ComponentError> {
        self.field_ids
            .get(name)
            .copied()
            .ok_or(ComponentError::FieldNotFound)
    }

    /// Get the data pointer of a specific field
    pub(crate) fn get_field_ptr(&self, id: FieldID) -> Result<*const c_void, ComponentError> {
        self.get_field(id)?
            .get(self.data)
            .map_err(ComponentError::from)
    }

    pub(crate) fn get_field_mut_ptr(&self, id: FieldID) -> Result<*mut c_void, ComponentError> {
        self.get_field(id)?
            .get_mut(self.data)
            .map_err(ComponentError::from)
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
