use std::{collections::HashMap, ffi::c_void};

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
    /// Resolves from the asset name and the data string the [`AssetID`]
    pub(crate) fn resolve_asset_id(
        &self,
        name: &str,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        self.asset_ids
            .get(name)
            .and_then(|assets| assets.get(data_string))
            .copied()
            .ok_or(SceneError::AssetNotFound)
    }

    /// Resolve the [`AssetFieldID`] from the [`AssetID`] and the name of the
    /// field
    pub(crate) fn resolve_asset_field_id(
        &self,
        id: AssetID,
        field_name: &str,
    ) -> Result<AssetFieldID, SceneError> {
        self.assets
            .get(id)
            .ok_or(SceneError::AssetNotFound)?
            .resolve_field_id(field_name)
            .map_err(SceneError::from)
    }

    /// Query the field of an asset and get the field pointer
    pub(crate) fn get_asset_field_ptr(
        &self,
        asset_id: AssetID,
        field_id: AssetFieldID,
    ) -> Result<*const c_void, SceneError> {
        self.assets
            .get(asset_id)
            .ok_or(SceneError::AssetNotFound)?
            .get_field(field_id)
            .map_err(SceneError::from)
    }

    /// Add a new asset
    pub(crate) fn add_asset(
        &mut self,
        data_string: String,
        manifest: &AssetManifest,
    ) -> Result<AssetID, SceneError> {
        let asset = Asset::new(manifest).map_err(SceneError::from)?;
        let id = self.assets.insert(asset);
        self.asset_ids
            .entry(manifest.name.clone())
            .or_default()
            .insert(data_string, id);
        Ok(id)
    }
}
