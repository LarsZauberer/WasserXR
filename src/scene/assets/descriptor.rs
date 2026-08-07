//! C-compatible asset declarations used by plugin manifests.

use std::{
    collections::HashSet,
    ffi::{c_char, c_void},
};

use crate::scene::{
    Scene,
    plugin::manifest::{
        ManifestError, copy_name, descriptor_slice, field_type, missing_callback, register_local,
    },
};

use super::{AssetType, schema::Schema};

pub type Creator = unsafe extern "C" fn(*mut Scene, *const c_char) -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut Scene, *mut c_void);

/// C-compatible declaration of one asset field.
#[repr(C)]
pub struct WXRAssetFieldDescriptor {
    pub name: *const c_char,
    pub field_type: u32,
    pub getter: Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void>,
}

/// C-compatible declaration of one plugin-provided asset type.
#[repr(C)]
pub struct WXRAssetDescriptor {
    pub name: *const c_char,
    // Expanded for cbindgen; aliases inside Option emit incomplete C types.
    pub creator: Option<unsafe extern "C" fn(*mut Scene, *const c_char) -> *mut c_void>,
    pub destroyer: Option<unsafe extern "C" fn(*mut Scene, *mut c_void)>,
    pub fields: *const WXRAssetFieldDescriptor,
    pub field_count: usize,
}

// Descriptors are immutable process-lifetime declarations. Loading their raw
// pointers remains unsafe and validation copies all data used by the host.
unsafe impl Sync for WXRAssetFieldDescriptor {}
unsafe impl Sync for WXRAssetDescriptor {}

impl WXRAssetDescriptor {
    pub(crate) unsafe fn validate(&self, plugin: &str) -> Result<AssetType, ManifestError> {
        let name = unsafe { copy_name(self.name, "asset") }?;
        let creator = self
            .creator
            .ok_or_else(|| missing_callback("asset", &name, "creator"))?;
        let destroyer = self
            .destroyer
            .ok_or_else(|| missing_callback("asset", &name, "destroyer"))?;
        let fields = unsafe { descriptor_slice(self.fields, self.field_count, "asset fields") }?;
        let mut field_names = HashSet::new();
        let mut schema = Schema::default();
        for field in fields {
            let field_name = unsafe { copy_name(field.name, "asset field") }?;
            register_local(&mut field_names, &field_name, "asset field")?;
            let getter = field
                .getter
                .ok_or_else(|| missing_callback("asset field", &field_name, "getter"))?;
            schema.add_field(field_name, field_type(field.field_type)?, Some(getter));
        }
        Ok(AssetType::new(
            name,
            plugin.to_owned(),
            creator,
            destroyer,
            schema,
        ))
    }
}
