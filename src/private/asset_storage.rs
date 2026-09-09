use std::{collections::HashMap, ffi::c_void, sync::RwLock};

use slotmap::SlotMap;

use crate::{
    errors::SceneError,
    private::{assets::Asset, manifests::assets::AssetManifest},
    scene::{AssetFieldID, AssetID},
};

type AssetStore = SlotMap<AssetID, Asset>;
type AssetIDResolver = HashMap<String, HashMap<String, AssetID>>;

/// AssetStorage is a sturct maintaining the asset cache of all the currently
/// loaded assets.
/// It should be wrapped in an RwLock in the [`wasserxr::scene::Scene`] to make
/// it correctly thread-safe.
#[derive(Debug, Default)]
pub(crate) struct AssetStorage {
    assets: AssetStore,
    asset_ids: AssetIDResolver,
}

impl AssetStorage {
    /// Create a new empty asset storage
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Resolves from the asset name and the data string the [`AssetID`]
    pub(crate) fn resolve_asset_id(&self, name: &str, data_string: &str) -> AssetID {
        todo!()
    }

    /// Runs an action with an [`Asset`] given an [`AssetID`]
    fn with_asset<T>(
        &self,
        id: AssetID,
        action: impl FnOnce(&Asset) -> Result<T, SceneError>,
    ) -> Result<T, SceneError> {
        todo!()
    }

    /// Resolve the [`AssetFieldID`] from the [`AssetID`] and the name of the
    /// field
    pub(crate) fn resolve_asset_field_id(&self, id: AssetID, field_name: &str) -> AssetFieldID {
        todo!()
    }

    /// Query the field of an asset and get the field pointer
    pub(crate) fn get_asset_field_ptr(
        &self,
        asset_id: AssetID,
        field_id: AssetFieldID,
    ) -> *const c_void {
        todo!()
    }

    /// Add a new asset
    pub(crate) fn add_asset(
        &mut self,
        data_string: String,
        manifest: &AssetManifest,
    ) -> Result<AssetID, SceneError> {
        todo!()
    }

    /// Remove a specific asset
    pub(crate) fn remove_asset(&mut self, id: AssetID) -> Result<AssetID, SceneError> {
        todo!()
    }

    /// Clear asset cache. It will remove all the current assets and invalidate
    /// all the current assets
    pub(crate) fn clear(&mut self) {
        todo!()
    }
}
