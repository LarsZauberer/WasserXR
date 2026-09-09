use std::{collections::HashMap, ffi::c_void};

use slotmap::SlotMap;

use crate::{
    definitions::components::Destroyer,
    errors::AssetError,
    private::{fields::AssetField, manifests::assets::AssetManifest},
    scene::AssetFieldID,
};

/// This is a concrete asset that is created from [`AssetManifest`]
#[derive(Debug)]
pub(crate) struct Asset {
    name: String,
    data_string: String,
    destroyer: Destroyer,
    fields: SlotMap<AssetFieldID, AssetField>,
    field_ids: HashMap<String, AssetFieldID>,
    data: *const c_void,
}

impl Asset {
    /// Create a new asset from a data string and the asset type manifest
    pub(crate) fn new(data_string: String, manifest: &AssetManifest) -> Result<Self, AssetError> {
        // Note: Pleaes remember that the asset creation can fail if the creator returns
        // a null pointer.
        todo!()
    }

    /// Get the [`FieldID`] from the name of a field
    pub(crate) fn resolve_field_id(&self, name: &str) -> Result<AssetFieldID, AssetError> {
        todo!()
    }

    /// Get the field pointer from a given [`FieldID`]
    pub(crate) fn get_field(&self, id: AssetFieldID) -> Result<*const c_void, AssetError> {
        todo!()
    }
}

impl Drop for Asset {
    fn drop(&mut self) {
        todo!("Call to the destroyer is needed")
    }
}
