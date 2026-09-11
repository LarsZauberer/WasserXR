use std::{ffi::c_void, path::Path, sync::RwLock};

use slotmap::new_key_type;

use crate::{
    definitions::plugins::PluginDefinition,
    errors::{PluginCompatibilityError, PluginError, SceneError},
    field::{Field, FieldAccess},
    private::{
        assets::Asset,
        entities::Entity,
        id_store::IDStore,
        manifests::{Manifest, plugins::PluginManifest},
        plugins::Plugin,
    },
};

new_key_type! {
/// EntityID is a cheap copyable handle for entities. It uniquely identifies an entity
/// within a Scene. It is not a globally unique identifier across multiple scenes (if you are
/// maintaining multiple scenes)
pub struct EntityID;

/// Handle for a loaded plugin. It describes a plugin uniquely to the scene and cannot like the [`EntityID`] be used
/// in different scenes. This behavior is not supported.
pub struct PluginID;

/// Handle that is cheap to copy and address a component. It is only unique within a
/// single entity and cannot be used across multiple entity.
pub struct ComponentID;

/// Handle that is cheap to copy and address a field in a component. It is only unique within a
/// single entity and component. It is not unique across multiple components.
pub struct FieldID;

/// Handle that is cheap to copy to address assets. An AssetID is unique to an asset type and it's
/// data string. Meaning two assets of the same type but have different data strings will have
/// different ID's
pub struct AssetID;

/// Handle that is cheap to copy and uniquely identifies a field inside of an asset. It is only
/// unique inside of a single Asset and it's data string.
pub struct AssetFieldID;
}

type EntityStorage = IDStore<EntityID, RwLock<Entity>>;

/// # Design Decision
///
/// A plugin doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
type PluginStorage = IDStore<PluginID, Plugin>;

/// # Design Decision
///
/// An asset doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
type AssetStorage = IDStore<AssetID, Asset>;

/// The scene is the core object in WasserXR. It contains the main public API to
/// access and maintain all ECS objects.
///
/// While it is possible to have mutliple scenes per application, the scene is
/// designed to only have one Scene per application maintaining all the
/// entities, components, systems, assets and plugins currently active.
/// Its contents are internally synchronized so a scene can be shared across
/// threads.
#[derive(Debug, Default)]
pub struct Scene {
    entities: RwLock<EntityStorage>,
    plugins: RwLock<PluginStorage>,
    assets: RwLock<AssetStorage>,
}

impl Scene {
    fn asset_key(name: &str, data_string: &str) -> String {
        format!("{}:{name}{data_string}", name.len())
    }

    /// Creates a new empty scene
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new entity and returns it's handle. The handle will be unique
    /// to every other entity ever created within this scene.
    pub fn add_entity(&self) -> EntityID {
        let entity = Entity::new();
        self.entities
            .write()
            .expect("scene entity lock poisoned")
            .insert(RwLock::new(entity))
    }

    /// Removes a previsouly created entity from the scene. This will also
    /// delete all the associated components of the entity.
    ///
    /// If the entity couldn't be found with the handle, the function will
    /// return a [`SceneError::EntityNotFound`]
    pub fn remove_entity(&self, id: EntityID) -> Result<(), SceneError> {
        let _ = self
            .entities
            .write()
            .expect("scene entity lock poisoned")
            .remove(id)
            .ok_or(SceneError::EntityNotFound)?;
        Ok(())
    }

    /// Returns a [`Vec<EntityID>`] of all the entity handles that are currently
    /// active in the scene.
    pub fn get_entities(&self) -> Vec<EntityID> {
        self.entities
            .read()
            .expect("scene entity lock poisoned")
            .keys()
            .collect()
    }

    /// This will reset the scene's main objects. Meaning it will remove all the
    /// entities, components and systems
    ///
    /// It will **not** unload any plugins or remove cached assets
    pub fn reset(&self) -> Result<(), SceneError> {
        let entities =
            std::mem::take(&mut *self.entities.write().expect("scene entity lock poisoned"));
        drop(entities);
        Ok(())
    }

    /// Checks if the plugin that should be added to the scene is valid to be
    /// added to the scene. It checks the following conditions
    ///
    /// - Is a plugin with the same name already loaded?
    fn add_plugin(&self, new_plugin: Plugin) -> Result<PluginID, SceneError> {
        let mut plugins = self.plugins.write().expect("scene plugin lock poisoned");
        if plugins.contains_name(new_plugin.get_name()) {
            return Err(SceneError::from(
                PluginCompatibilityError::PluginWithSameNameExists,
            ));
        }

        let name = new_plugin.get_name().to_owned();
        Ok(plugins.insert_named(name, new_plugin))
    }

    /// Runs an action with an entity while holding the entity collection's
    /// read lock, preventing the entity from being removed during the action.
    fn with_entity<T>(
        &self,
        id: EntityID,
        action: impl FnOnce(&Entity) -> Result<T, SceneError>,
    ) -> Result<T, SceneError> {
        let entities = self.entities.read().expect("scene entity lock poisoned");
        let entity = entities.get(id).ok_or(SceneError::EntityNotFound)?;
        action(&entity.read().expect("entity lock poisoned"))
    }

    /// Load a plugin from a shared object library.
    ///
    /// It is not allowed to have a plugin with the same name already loaded in
    /// the scene.
    ///
    /// # Safety
    ///
    /// The path must point to a valid shared object that can be read and has a
    /// globally defined variable called `wxr_plugin`. The `wxr_plugin`
    /// variable has to be of type [`PluginDefinition`] as has to be valid.
    /// Furthermore, the [`PluginDefinition`] musn't have any malformed content
    /// within it. Component data and callbacks supplied by the plugin must be
    /// safe to access from multiple threads.
    pub unsafe fn load_plugin(&self, path: &Path) -> Result<PluginID, SceneError> {
        let plugin = unsafe { Plugin::load_shared(path) }.map_err(SceneError::from)?;
        self.add_plugin(plugin)
    }

    /// Load a plugin from a statically linked and already [`PluginDefinition`]
    ///
    /// It is not allowed to have a plugin with the same name already loaded in
    /// the scene.
    ///
    /// # Safety
    ///
    /// The [`PluginDefinition`] must be valid and not have not any malformed
    /// content within. Its component data and callbacks must be safe to access
    /// from multiple threads.
    pub unsafe fn load_static_plugin(
        &self,
        definition: PluginDefinition,
    ) -> Result<PluginID, SceneError> {
        let manifest: PluginManifest = unsafe { Manifest::checked_convert(definition) }
            .map_err(|err| SceneError::from(PluginError::from(err)))?;
        let plugin = Plugin::load_static(manifest);

        self.add_plugin(plugin)
    }

    /// Get the handle of a plugin ([`PluginID`]) by searching for the name of a
    /// plugin
    pub fn get_plugin(&self, name: &str) -> Option<PluginID> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .resolve_id(name)
    }

    /// Get all the [`PluginID`] of the currently actively loaded plugins in the
    /// scene
    pub fn get_plugins(&self) -> Vec<PluginID> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .keys()
            .collect()
    }

    /// Add a component type to an entity
    ///
    /// This function may fail, if the entity cannot be found or if the entity
    /// has already an existing component of that type
    pub fn add_component(
        &self,
        entity_id: EntityID,
        component_type: &str,
    ) -> Result<ComponentID, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let (plugin_id, manifest) = plugins
            .iter()
            .find_map(|(plugin_id, plugin)| {
                plugin
                    .get_component(component_type)
                    .map(|manifest| (plugin_id, manifest))
            })
            .ok_or(SceneError::NoComponentType)?;

        self.with_entity(entity_id, |entity| {
            entity
                .add_component(plugin_id, manifest)
                .map_err(SceneError::EntityError)
        })
    }

    /// Remove a component type from an entity
    ///
    /// The function may fail, if the [`EntityID`] cannot be found in the scene.
    pub fn remove_component(
        &self,
        entity_id: EntityID,
        component: ComponentID,
    ) -> Result<(), SceneError> {
        self.with_entity(entity_id, |entity| {
            entity.remove_component(component).map_err(SceneError::from)
        })
    }

    /// Returns all the component names of the given [`EntityID`]
    pub fn get_components(&self, entity_id: EntityID) -> Result<Vec<ComponentID>, SceneError> {
        self.with_entity(entity_id, |entity| Ok(entity.get_components()))
    }

    /// Return the name of some component handle
    pub fn get_component_name(
        &self,
        entity_id: EntityID,
        component_id: ComponentID,
    ) -> Result<String, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .get_component_name(component_id)
                .map_err(SceneError::from)
        })
    }

    /// Resolves the [`ComponentID`] of a component given it's name and an
    /// [`EntityID`] which should have the component attached to it
    pub fn resolve_component_id(
        &self,
        entity_id: EntityID,
        component_name: &str,
    ) -> Result<ComponentID, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .resolve_component_id(component_name)
                .map_err(SceneError::from)
        })
    }

    /// Resolve the [`FieldID`] from the name of a field providing the
    /// [`EntityID`] and the [`ComponentID`].
    pub fn resolve_field_id(
        &self,
        entity_id: EntityID,
        component_id: ComponentID,
        name: &str,
    ) -> Result<FieldID, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .resolve_field_id(component_id, name)
                .map_err(SceneError::from)
        })
    }

    /// Locks component fields in field-ID order.
    ///
    /// The fields are locked in field-ID order to avoid ordering deadlocks, but
    /// are passed to `action` in the order requested. They remain locked until
    /// `action` returns. Write access to an immutable field returns
    /// [`crate::errors::FieldError::NotMutable`].
    pub fn query_component_fields<T>(
        &self,
        entity_id: EntityID,
        component_id: ComponentID,
        requests: &[(FieldID, FieldAccess)],
        action: impl FnOnce(&[Field]) -> T,
    ) -> Result<T, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .query_component_fields(component_id, requests, action)
                .map_err(SceneError::from)
        })
    }

    /// Resolve the [`AssetID`] from a given asset name and data string
    ///
    /// This function will **not** load a new asset if the asset doesn't exist.
    pub fn resolve_asset_id(
        &self,
        asset_name: &str,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .resolve_id(&Self::asset_key(asset_name, data_string))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Resolve the [`AssetFieldID`] from ta given [`AssetID`] and the field
    /// name
    ///
    /// This function will **not** load a new asset if the asset doesn't exist.
    pub fn resolve_asset_field_id(
        &self,
        asset_id: AssetID,
        field_name: &str,
    ) -> Result<AssetFieldID, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .get(asset_id)
            .ok_or(SceneError::AssetNotFound)?
            .resolve_field_id(field_name)
            .map_err(SceneError::from)
    }

    /// Resolves the asset id and if it doesn't exist, it will try to load the
    /// asset
    pub fn get_asset_id(&self, asset_name: &str, data_string: &str) -> Result<AssetID, SceneError> {
        let key = Self::asset_key(asset_name, data_string);
        if let Some(id) = self
            .assets
            .read()
            .expect("scene asset lock poisoned")
            .resolve_id(&key)
        {
            return Ok(id);
        }

        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let manifest = plugins
            .values()
            .find_map(|plugin| plugin.get_asset(asset_name))
            .ok_or(SceneError::AssetNotFound)?;

        let mut assets = self.assets.write().expect("scene asset lock poisoned");
        if let Some(id) = assets.resolve_id(&key) {
            return Ok(id);
        }
        let asset = Asset::new(manifest).map_err(SceneError::from)?;
        Ok(assets.insert_named(key, asset))
    }

    /// Get the assets field pointer to access an asset's field.
    ///
    /// This can be thread safely done, since all the assets are read-only
    pub fn query_asset_field(
        &self,
        asset_id: AssetID,
        field_id: AssetFieldID,
    ) -> Result<*const c_void, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .get(asset_id)
            .ok_or(SceneError::AssetNotFound)?
            .get_field(field_id)
            .map_err(SceneError::from)
    }
}
