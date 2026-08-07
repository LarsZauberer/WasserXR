//! Asset schema support used by asset plugins and scene asset queries.

/// C-compatible asset declarations used by plugin manifests.
pub mod descriptor;
mod error;
/// Asset field metadata.
pub mod field;
pub(crate) mod schema;

use std::ffi::{CString, c_void};

use crate::scene::Scene;

pub use error::AssetError;

pub use descriptor::{Creator, Destroyer, WXRAssetDescriptor, WXRAssetFieldDescriptor};
pub use field::Getter;

pub(crate) struct AssetType {
    id: String,
    plugin_id: String,
    creator: Creator,
    destroyer: Destroyer,
    schema: schema::Schema,
}

impl AssetType {
    pub(crate) fn new(
        id: String,
        plugin_id: String,
        creator: Creator,
        destroyer: Destroyer,
        schema: schema::Schema,
    ) -> Self {
        Self {
            id,
            plugin_id,
            creator,
            destroyer,
            schema,
        }
    }

    pub(crate) fn create_asset(
        &self,
        scene: &mut Scene,
        data_string: &str,
    ) -> Result<Asset, AssetError> {
        let data_string = CString::new(data_string).map_err(|_| AssetError::InvalidAsset)?;

        let data = unsafe { (self.creator)(scene as *mut Scene, data_string.as_ptr()) };

        if data.is_null() {
            return Err(AssetError::InvalidAsset);
        }

        Ok(Asset::new(data, self.destroyer))
    }

    pub(crate) unsafe fn get_field_ptr(
        &self,
        asset: &Asset,
        field: &str,
    ) -> Result<*mut c_void, AssetError> {
        unsafe { self.schema.get_field_ptr(field, asset.data()) }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

pub(crate) struct Asset {
    data: *mut c_void,
    destroyer: Destroyer,
}

impl Asset {
    fn new(data: *mut c_void, destroyer: Destroyer) -> Self {
        Self { data, destroyer }
    }

    fn data(&self) -> *mut c_void {
        self.data
    }

    pub(crate) fn destroy(self, scene: &mut Scene) {
        unsafe {
            (self.destroyer)(scene as *mut Scene, self.data);
        }
    }
}
