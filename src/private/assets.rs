use std::ffi::c_void;

use crate::{
    definitions::components::Destroyer,
    errors::AssetError,
    ids::{AssetFieldID, AssetFieldTypeID},
    private::{fields::AssetField, id_store::IDStore, manifests::assets::AssetManifest},
};

/// This is a concrete asset that is created from [`AssetManifest`]
#[derive(Debug)]
pub(crate) struct Asset {
    destroyer: Destroyer,
    /// # Design Decision
    ///
    /// The [`AssetField`] record doesn't require an [`RwLock`] since all fields
    /// are read-only by design. Hence, they don't need exclusive access
    fields: IDStore<AssetFieldTypeID, AssetFieldID, AssetField>,
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
        for (field_type_id, field) in manifest.fields.iter() {
            let field = AssetField::from(field);
            fields.insert_named(field_type_id, field);
        }

        Ok(Self {
            destroyer: manifest.destroyer,
            fields,
            data,
        })
    }

    /// Get the [`AssetFieldID`] from its field type ID.
    pub(crate) fn resolve_field_id(
        &self,
        field_type_id: AssetFieldTypeID,
    ) -> Result<AssetFieldID, AssetError> {
        self.fields
            .resolve_id(&field_type_id)
            .ok_or(AssetError::FieldNotFound)
    }

    /// Returns the complete asset's immutable plugin data.
    ///
    /// # Design decisions
    ///
    /// Asset queries request whole assets, so no field lookup is needed. The
    /// caller keeps the asset collection locked while exposing this pointer.
    pub(crate) fn data(&self) -> *const c_void {
        self.data.cast_const()
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
