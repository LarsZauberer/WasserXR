use std::ffi::c_void;

use crate::{
    errors::ComponentError, private::manifests::components::ComponentManifest, scene::PluginID,
};

/// The component is the concrete data record of a component. It carries the
/// information about which plugin it belongs to, stores it's component manifest
/// and the actual data.
#[derive(Debug)]
pub(crate) struct Component {
    plugin_id: PluginID,
    manifest: ComponentManifest,
    data: *mut c_void,
}

impl Component {
    /// Creates a new component. This function will run the creator of the
    /// component to generate allocate the data.
    ///
    /// It will also store the entirety of the manifest so that it can on drop
    /// call the destroyer with the data.
    ///
    /// This **requires** that the destroyer code is still loaded by the plugin
    /// at the time the component is dropped.
    pub(crate) fn new(plugin_id: PluginID, manifest: ComponentManifest) -> Self {
        let data = unsafe { (manifest.creator)() };
        Self {
            plugin_id,
            manifest,
            data,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.manifest.name
    }

    /// If the component has a field with the name and a getter it will call the
    /// getter method and return the raw pointer to that field.
    pub(crate) fn get_field(&self, name: &str) -> Result<*const c_void, ComponentError> {
        let field = self
            .manifest
            .fields
            .iter()
            .find(|field| field.name == name)
            .ok_or(ComponentError::FieldNotFound)?;
        let getter = field.getter.ok_or(ComponentError::FieldHasNoGetter)?;
        Ok(unsafe { getter(self.data.cast_const()) }.cast_const())
    }

    /// If the component has a mutable field with the name and a getter it will
    /// call the getter method and return the raw pointer to that field.
    ///
    /// This function will fail, if the field is not
    pub(crate) fn get_mut_field(&self, name: &str) -> Result<*mut c_void, ComponentError> {
        let field = self
            .manifest
            .fields
            .iter()
            .find(|field| field.name == name)
            .ok_or(ComponentError::FieldNotFound)?;
        if !field.mutable {
            return Err(ComponentError::FieldIsNotMutable);
        }
        let getter = field.getter.ok_or(ComponentError::FieldHasNoGetter)?;
        Ok(unsafe { getter(self.data.cast_const()) })
    }

    // TODO: Later add support for methods
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.manifest.destroyer)(self.data) }
    }
}
