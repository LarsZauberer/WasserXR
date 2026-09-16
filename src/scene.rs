use std::{ffi::c_void, path::Path, sync::RwLock};

use crate::{
    definitions::plugins::PluginDefinition,
    errors::{PluginCompatibilityError, PluginError, SceneError, SystemError},
    ids::{
        AssetFieldID, AssetFieldTypeID, AssetID, AssetTypeID, ComponentID, ComponentTypeID,
        EntityID, FieldID, FieldTypeID, PluginID, SystemID, SystemTypeID, TypeID,
    },
    private::{
        assets::Asset,
        entities::Entity,
        id_store::IDStore,
        manifests::{Manifest, plugins::PluginManifest, type_id_requests::TypeIDRequestManifest},
        plugins::Plugin,
        system_storage::SystemStorage,
        system_storage_snapshot::SystemStorageSnapshot,
    },
    query::{self, AssetQuery, ComponentQuery, ComponentQueryResult},
};

pub(crate) type EntityStorage = IDStore<String, EntityID, Entity>;

/// # Design Decision
///
/// A plugin doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
type PluginStorage = IDStore<String, PluginID, Plugin>;

/// # Design Decision
///
/// An asset doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
pub(crate) type AssetStorage = IDStore<(PluginID, AssetTypeID, String), AssetID, Asset>;

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
    systems: RwLock<SystemStorage>,
    entities: RwLock<EntityStorage>,
    plugins: RwLock<PluginStorage>,
    assets: RwLock<AssetStorage>,
}

impl Scene {
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
            .insert(entity)
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
    /// It will **not** unload any plugins
    pub fn reset(&self) -> Result<(), SceneError> {
        // Replace the old system ID store with a fresh, empty one before detaching
        // its systems.
        let systems =
            std::mem::take(&mut *self.systems.write().expect("scene system lock poisoned"));
        for system in systems.into_values() {
            system.detach(self);
        }
        let entities =
            std::mem::take(&mut *self.entities.write().expect("scene entity lock poisoned"));
        drop(entities);
        let assets = std::mem::take(&mut *self.assets.write().expect("scene asset lock poisoned"));
        drop(assets);
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
        action(entity)
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
    pub fn resolve_plugin_id(&self, name: &str) -> Option<PluginID> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .resolve_id(name)
    }

    /// Returns the name of the plugin identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not identify a loaded plugin.
    pub fn get_plugin_name(&self, id: PluginID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(id)
            .expect("plugin ID does not identify a loaded plugin")
            .get_name() as *const str;

        // SAFETY: Plugins are never removed from a Scene and their owned names
        // are never mutated, so the string data remains valid for the Scene's
        // lifetime even after releasing the collection lock.
        unsafe { &*name }
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

    /// Resolves a component type name within a plugin manifest.
    pub fn resolve_component_type_id(
        &self,
        plugin_id: PluginID,
        name: &str,
    ) -> Result<ComponentTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(plugin_id)
            .and_then(|plugin| plugin.resolve_component_type_id(name))
            .ok_or(SceneError::NoComponentType)
    }

    /// Resolves a component field type name within a component type manifest.
    pub fn resolve_field_type_id(
        &self,
        plugin_id: PluginID,
        component_type_id: ComponentTypeID,
        name: &str,
    ) -> Result<FieldTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(plugin_id)
            .and_then(|plugin| plugin.resolve_field_type_id(component_type_id, name))
            .ok_or(SceneError::NoComponentType)
    }

    /// Resolves an asset type name within a plugin manifest.
    pub fn resolve_asset_type_id(
        &self,
        plugin_id: PluginID,
        name: &str,
    ) -> Result<AssetTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(plugin_id)
            .and_then(|plugin| plugin.resolve_asset_type_id(name))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Returns the name of an asset type identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if either ID does not identify a loaded plugin or one of its
    /// asset types.
    pub fn get_asset_name(&self, plugin_id: PluginID, id: AssetTypeID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(plugin_id)
            .and_then(|plugin| plugin.get_asset_name(id))
            .expect("IDs do not identify a loaded asset type") as *const str;

        // SAFETY: Plugin manifests are never removed from a Scene and their
        // owned names are never mutated, so the string data remains valid for
        // the Scene's lifetime after releasing the collection lock.
        unsafe { &*name }
    }

    /// Resolves an asset field type name within an asset type manifest.
    pub fn resolve_asset_field_type_id(
        &self,
        plugin_id: PluginID,
        asset_type_id: AssetTypeID,
        name: &str,
    ) -> Result<AssetFieldTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(plugin_id)
            .and_then(|plugin| plugin.resolve_asset_field_type_id(asset_type_id, name))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Resolves a system type name within a plugin manifest.
    pub fn resolve_system_type_id(
        &self,
        plugin_id: PluginID,
        name: &str,
    ) -> Result<SystemTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(plugin_id)
            .and_then(|plugin| plugin.resolve_system_type_id(name))
            .ok_or(SceneError::SystemNotFound)
    }

    /// Returns the name of a system type identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if either ID does not identify a loaded plugin or one of its
    /// system types.
    pub fn get_system_name(&self, plugin_id: PluginID, id: SystemTypeID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(plugin_id)
            .and_then(|plugin| plugin.get_system_name(id))
            .expect("IDs do not identify a loaded system type") as *const str;

        // SAFETY: Plugin manifests are never removed from a Scene and their
        // owned names are never mutated, so the string data remains valid for
        // the Scene's lifetime after releasing the collection lock.
        unsafe { &*name }
    }

    /// Resolves the type IDs requested by a system manifest.
    pub fn resolve_requested_type_ids(
        &self,
        plugin_id: PluginID,
        system_type_id: SystemTypeID,
    ) -> Result<Vec<TypeID>, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let plugin = plugins.get(plugin_id).ok_or(SceneError::SystemNotFound)?;
        let manifest = plugin
            .get_system(system_type_id)
            .ok_or(SceneError::SystemNotFound)?;

        manifest
            .type_id_requests
            .iter()
            .map(|request| {
                let type_id = match request {
                    TypeIDRequestManifest::ComponentTypeID { component } => plugin
                        .resolve_component_type_id(component)
                        .map(TypeID::from),
                    TypeIDRequestManifest::FieldTypeID { component, field } => plugin
                        .resolve_component_type_id(component)
                        .and_then(|component| plugin.resolve_field_type_id(component, field))
                        .map(TypeID::from),
                    TypeIDRequestManifest::AssetTypeID { asset } => {
                        plugin.resolve_asset_type_id(asset).map(TypeID::from)
                    }
                    TypeIDRequestManifest::AssetFieldTypeID { asset, field } => plugin
                        .resolve_asset_type_id(asset)
                        .and_then(|asset| plugin.resolve_asset_field_type_id(asset, field))
                        .map(TypeID::from),
                };
                type_id.ok_or(SceneError::RequestedTypeIDNotFound)
            })
            .collect()
    }

    /// Add a component type to an entity
    ///
    /// This function may fail, if the entity cannot be found or if the entity
    /// has already an existing component of that type
    pub fn add_component(
        &self,
        entity_id: EntityID,
        plugin_id: PluginID,
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentID, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let manifest = plugins
            .get(plugin_id)
            .and_then(|plugin| plugin.get_component(component_type_id))
            .ok_or(SceneError::NoComponentType)?;

        self.with_entity(entity_id, |entity| {
            entity
                .add_component(plugin_id, component_type_id, manifest)
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

    /// Resolves a component type attached to an entity to its [`ComponentID`].
    pub fn resolve_component_id(
        &self,
        entity_id: EntityID,
        plugin_id: PluginID,
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentID, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .resolve_component_id(plugin_id, component_type_id)
                .map_err(SceneError::from)
        })
    }

    /// Resolves a field type within a component to its [`FieldID`].
    pub fn resolve_field_id(
        &self,
        entity_id: EntityID,
        component_id: ComponentID,
        field_type_id: FieldTypeID,
    ) -> Result<FieldID, SceneError> {
        self.with_entity(entity_id, |entity| {
            entity
                .resolve_field_id(component_id, field_type_id)
                .map_err(SceneError::from)
        })
    }

    /// Queries components across all entities and invokes `action` once while
    /// every returned field remains locked.
    ///
    /// An entity matches when it contains every requested component. Results
    /// preserve entity, component, and field order. Field access is governed by
    /// each corresponding request, and pointers are valid only during the
    /// callback.
    pub fn query_components<T>(
        &self,
        requests: &[ComponentQuery<'_>],
        action: impl FnOnce(&ComponentQueryResult) -> T,
    ) -> Result<T, SceneError> {
        let entities = self.entities.read().expect("scene entity lock poisoned");
        query::query_components(&entities, requests, action)
    }

    /// Resolve the [`AssetID`] from a given asset type and data string.
    ///
    /// This function will **not** load a new asset if the asset doesn't exist.
    pub fn resolve_asset_id(
        &self,
        plugin_id: PluginID,
        asset_type_id: AssetTypeID,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .resolve_id(&(plugin_id, asset_type_id, data_string.to_owned()))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Resolves an asset field type within an asset to its [`AssetFieldID`].
    ///
    /// This function will **not** load a new asset if the asset doesn't exist.
    pub fn resolve_asset_field_id(
        &self,
        asset_id: AssetID,
        field_type_id: AssetFieldTypeID,
    ) -> Result<AssetFieldID, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .get(asset_id)
            .ok_or(SceneError::AssetNotFound)?
            .resolve_field_id(field_type_id)
            .map_err(SceneError::from)
    }

    /// Resolves the asset id and if it doesn't exist, it will try to load the
    /// asset
    pub fn get_asset_id(
        &self,
        plugin_id: PluginID,
        asset_type_id: AssetTypeID,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        let key = (plugin_id, asset_type_id, data_string.to_owned());
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
            .get(plugin_id)
            .and_then(|plugin| plugin.get_asset(asset_type_id))
            .ok_or(SceneError::AssetNotFound)?;

        let mut assets = self.assets.write().expect("scene asset lock poisoned");
        if let Some(id) = assets.resolve_id(&key) {
            return Ok(id);
        }
        let asset = Asset::new(manifest).map_err(SceneError::from)?;
        Ok(assets.insert_named(key, asset))
    }

    /// Loads and queries asset fields, keeping all assets alive until the
    /// single callback returns. The returned read-only pointers are valid only
    /// during that callback.
    pub fn query_assets<T>(
        &self,
        requests: &[AssetQuery<'_>],
        action: impl FnOnce(&[Vec<(AssetFieldID, *const c_void)>]) -> T,
    ) -> Result<T, SceneError> {
        // Loading requires write access, so finish it before taking the shared
        // asset lock that protects every pointer passed to the callback.
        let mut requested_assets = Vec::with_capacity(requests.len());
        for (plugin_id, asset_type_id, data_string, fields) in requests {
            let asset_id = self.get_asset_id(*plugin_id, *asset_type_id, data_string)?;
            requested_assets.push((asset_id, *fields));
        }

        let assets = self.assets.read().expect("scene asset lock poisoned");
        let mut results = Vec::with_capacity(requested_assets.len());
        for (asset_id, requested_fields) in requested_assets {
            let asset = assets.get(asset_id).ok_or(SceneError::AssetNotFound)?;
            let mut fields = Vec::with_capacity(requested_fields.len());
            for field_type_id in requested_fields {
                let field_id = asset
                    .resolve_field_id(*field_type_id)
                    .map_err(SceneError::from)?;
                let pointer = asset.get_field(field_id).map_err(SceneError::from)?;
                fields.push((field_id, pointer));
            }
            results.push(fields);
        }
        Ok(action(&results))
    }

    /// Gets an existing concrete system ID from its plugin and system type.
    pub fn get_system_id(
        &self,
        plugin_id: PluginID,
        system_type_id: SystemTypeID,
    ) -> Result<SystemID, SceneError> {
        self.systems
            .read()
            .expect("scene system lock poisoned")
            .resolve_id(&(plugin_id, system_type_id))
            .ok_or(SceneError::SystemNotFound)
    }

    /// Adds a system to the scene and returns its concrete ID.
    pub fn add_system(
        &self,
        plugin_id: PluginID,
        system_type_id: SystemTypeID,
    ) -> Result<SystemID, SceneError> {
        let key = (plugin_id, system_type_id);
        if self
            .systems
            .read()
            .expect("scene system lock poisoned")
            .resolve_id(&key)
            .is_some()
        {
            return Err(SystemError::AlreadyExists.into());
        }

        // Resolve every requested type ID before creating or attaching the system.
        let type_ids = self.resolve_requested_type_ids(plugin_id, system_type_id)?;
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let plugin = plugins.get(plugin_id).ok_or(SceneError::SystemNotFound)?;
        let manifest = plugin
            .get_system(system_type_id)
            .ok_or(SceneError::SystemNotFound)?;
        // Resolve dependency names to stable keys so storage can check presence
        // and build the dependency graph without string lookups.
        let requires = manifest
            .requires
            .iter()
            .map(|name| {
                plugin
                    .resolve_system_type_id(name)
                    .map(|system_type_id| (plugin_id, system_type_id))
                    .ok_or_else(|| SceneError::from(SystemError::DependencyNotFound(name.clone())))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let wanted_by = manifest
            .wanted_by
            .iter()
            .map(|name| {
                plugin
                    .resolve_system_type_id(name)
                    .map(|system_type_id| (plugin_id, system_type_id))
                    .ok_or_else(|| SceneError::from(SystemError::DependencyNotFound(name.clone())))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut systems = self.systems.write().expect("scene system lock poisoned");
        if systems.resolve_id(&key).is_some() {
            return Err(SystemError::AlreadyExists.into());
        }
        systems.add_system(self, key, manifest, type_ids, requires, wanted_by)
    }

    /// Removes a concrete system and runs its detacher.
    ///
    /// Removal fails while another active system requires this system.
    pub fn remove_system(&self, system_id: SystemID) -> Result<(), SceneError> {
        let system = self
            .systems
            .write()
            .expect("scene system lock poisoned")
            .remove(system_id)?;
        system.detach(self);
        Ok(())
    }

    /// Runs every system once in dependency order.
    ///
    /// Systems are run sequentially. Additions and removals made by a runner
    /// take effect on the next tick.
    pub fn tick(&mut self) {
        let mut snapshot =
            SystemStorageSnapshot::new(&self.systems.read().expect("scene system lock poisoned"));

        while let Some((runner, type_ids)) = snapshot.next() {
            unsafe { runner(self, type_ids.as_ptr(), type_ids.len()) };
        }
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        let systems = std::mem::take(self.systems.get_mut().expect("scene system lock poisoned"));
        for system in systems.into_values() {
            system.detach(self);
        }
    }
}
