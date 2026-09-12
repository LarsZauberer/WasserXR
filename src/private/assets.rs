use std::ffi::c_void;

use crate::{
    definitions::components::Destroyer,
    errors::AssetError,
    private::{fields::AssetField, id_store::IDStore, manifests::assets::AssetManifest},
    scene::AssetFieldID,
};

/// This is a concrete asset that is created from [`AssetManifest`]
#[derive(Debug)]
pub(crate) struct Asset {
    destroyer: Destroyer,
    /// # Design Decision
    ///
    /// The [`AssetField`] record doesn't require an [`RwLock`] since all fields
    /// are read-only by design. Hence, they don't need exclusive access
    fields: IDStore<String, AssetFieldID, AssetField>,
    data: *mut c_void,
}

// SAFETY: Assets are immutable after creation and access to their owning
// collection is synchronized. Plugin callbacks must uphold the thread-safety
// contract of their opaque data pointer.
unsafe impl Send for Asset {}
unsafe impl Sync for Asset {}

impl Asset {
    /// Create a new asset from a data string and the asset type manifest
    pub(crate) fn new(manifest: &AssetManifest) -> Result<Self, AssetError> {
        let data = unsafe { (manifest.creator)() };
        if data.is_null() {
            return Err(AssetError::CreationFailure);
        }

        let mut fields = IDStore::default();
        for field in manifest.fields.values() {
            let field = AssetField::from(field);
            let name = field.get_name().to_owned();
            fields.insert_named(name, field);
        }

        Ok(Self {
            destroyer: manifest.destroyer,
            fields,
            data,
        })
    }

    /// Get the [`FieldID`] from the name of a field
    pub(crate) fn resolve_field_id(&self, name: &str) -> Result<AssetFieldID, AssetError> {
        self.fields
            .resolve_id(name)
            .ok_or(AssetError::FieldNotFound)
    }

    /// Get the field pointer from a given [`FieldID`]
    pub(crate) fn get_field(&self, id: AssetFieldID) -> Result<*const c_void, AssetError> {
        self.fields
            .get(id)
            .map(|field| field.get(self.data))
            .ok_or(AssetError::FieldNotFound)
    }
}

impl Drop for Asset {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.data) }
    }
}
