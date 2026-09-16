use std::{collections::BTreeMap, ffi::c_void, path::Path, sync::RwLock};

use crate::{
    definitions::plugins::PluginDefinition,
    errors::{EntityError, PluginCompatibilityError, PluginError, SceneError, SystemError},
    field::AccessRequest,
    ids::{
        AssetFieldID, AssetFieldTypeID, AssetID, AssetTypeID, ComponentID, ComponentTypeID,
        EntityID, FieldID, FieldTypeID, PluginID, SystemID, SystemTypeID, TypeID,
    },
    private::{
        assets::Asset,
        components::ComponentGuard,
        entities::Entity,
        id_store::IDStore,
        manifests::{Manifest, plugins::PluginManifest, type_id_requests::TypeIDRequestManifest},
        plugins::Plugin,
        system_storage::SystemStorage,
        system_storage_snapshot::SystemStorageSnapshot,
    },
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

/// One component requirement: plugin, component type, lock mode, and ordered
/// field types. An empty field slice still requires and locks the component.
pub type ComponentQuery<'a> = (PluginID, ComponentTypeID, AccessRequest, &'a [FieldTypeID]);

/// One complete asset requested by plugin, asset type, and cache data string.
pub type AssetQuery<'a> = (PluginID, AssetTypeID, &'a str);

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

    /// Removes all systems from the scene and runs their detachers.
    pub fn reset_systems(&self) {
        // Replace the old system ID store with a fresh, empty one before detaching
        // its systems.
        let systems =
            std::mem::take(&mut *self.systems.write().expect("scene system lock poisoned"));
        for system in systems.into_values() {
            system.detach(self);
        }
    }

    /// Removes all entities and their components from the scene.
    pub fn reset_entities(&self) {
        let entities =
            std::mem::take(&mut *self.entities.write().expect("scene entity lock poisoned"));
        drop(entities);
    }

    /// Removes all cached assets from the scene.
    pub fn reset_assets(&self) {
        let assets = std::mem::take(&mut *self.assets.write().expect("scene asset lock poisoned"));
        drop(assets);
    }

    /// This will reset the scene's main objects. Meaning it will remove all the
    /// entities, components, systems, and cached assets.
    ///
    /// It will **not** unload any plugins
    pub fn reset(&self) -> Result<(), SceneError> {
        self.reset_systems();
        self.reset_entities();
        self.reset_assets();
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

    /// Calls `action` once with fields from all entities matching every query
    /// entry. Pointers are flattened in ascending entity ID order, then query
    /// entry order, then field order. Duplicate entries and fields are
    /// retained. Empty queries and queries without matches call `action`
    /// with an empty slice.
    ///
    /// All requested components stay locked together until `action` returns.
    /// Duplicate component requests share one lock; any `Write` request makes
    /// it exclusive. A missing component excludes that entity. An invalid or
    /// inaccessible field on a matching entity returns an error without calling
    /// `action`. `Write` requests reject fields not declared mutable.
    ///
    /// ## Deadlocks
    ///
    /// Requests to change something in the scene while the locks are active may
    /// result in a deadlock. For example, when you try to add an entity
    /// while you are in the action, will trigger a deadlock, since the
    /// entity storage is locked.
    ///
    /// # Pointer and callback contract
    ///
    /// Pointers may only be dereferenced during `action`, with the plugin's
    /// correct field type. `Read` pointers must not be written through, despite
    /// their mutable pointer type. Duplicates can alias: callers must not
    /// create overlapping mutable references. Do not call scene methods
    /// from `action` or wait for work needing these locks; lock ordering
    /// cannot make recursive acquisition safe. Locks also release if the
    /// callback panics, although standard `RwLock` poisoning then applies
    /// to write-locked components.
    ///
    /// # Design decisions
    ///
    /// Lock order is scene entities, entity component collections in ascending
    /// entity ID order, then component data in ascending `(EntityID,
    /// ComponentID)` order. Sorted maps both enforce this order and merge
    /// duplicate locks. Output order is tracked separately so sorting never
    /// rearranges requests. Collection read guards prevent removal while
    /// pointers are in use. This simple scan also blocks component additions
    /// and removals on unmatched entities until the callback finishes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wasserxr::{scene::Scene, field::AccessRequest, ids::*, errors::SceneError};
    /// # fn update(scene: &Scene, plugin: PluginID, component: ComponentTypeID,
    /// #           counter: FieldTypeID) -> Result<(), SceneError> {
    /// scene.query_components(
    ///     &[(plugin, component, AccessRequest::Write, &[counter])],
    ///     |fields| {
    ///         for &field in fields {
    ///             // SAFETY: This plugin defines `counter` as a mutable u32.
    ///             unsafe { *field.cast::<u32>() += 1 };
    ///         }
    ///     },
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query_components(
        &self,
        query: &[ComponentQuery<'_>],
        action: impl FnOnce(&[*mut c_void]),
    ) -> Result<(), SceneError> {
        if query.is_empty() {
            action(&[]);
            return Ok(());
        }

        let entities = self.entities.read().expect("scene entity lock poisoned");
        // Acquire every collection guard before any component data guard.

        // Note that a BTreeMap automatically stores everything in a sorted way by
        // design.
        let ordered_entities: BTreeMap<_, _> = entities.iter().collect();
        let collections: Vec<_> = ordered_entities
            .iter()
            .map(|(&id, entity)| (id, entity.lock_components()))
            .collect();
        // Keep lock acquisition separate from the callback's pointer order:
        // - `locks` contains each component once, sorted by (EntityID, ComponentID) to
        //   prevent order-induced deadlocks. Any Write request wins.
        // - `requests` preserves query order, duplicates, fields, and each entry's
        //   access mode. A Read of an immutable field remains valid even if another
        //   entry requires a Write lock on the same component.
        // For example, B/read, A/write, B/write locks A/write then B/write,
        // but returns the requested fields in B, A, B order.
        let mut locks = BTreeMap::new();
        let mut requests = Vec::new();
        for (entity_id, components) in &collections {
            // Check which entities have all the queried components. If they have all the
            // queried components, return a list of the ComponentIDs
            let ids: Option<Vec<_>> = query
                .iter()
                .map(|(plugin, component_type, _, _)| {
                    components.resolve_id(&(*plugin, *component_type))
                })
                .collect();
            let Some(ids) = ids else { continue };

            // Building the request list and lock list of all the locks that should be
            // aquired.
            //
            // This zip works since the ComponentID is directly mapped to the requested
            // component
            for (id, &(_, _, access, fields)) in ids.into_iter().zip(query) {
                let key = (*entity_id, id);
                let component = components.get(id).expect("resolved component exists");
                let (_, lock_access) = locks.entry(key).or_insert((component, access));
                if access == AccessRequest::Write {
                    *lock_access = AccessRequest::Write;
                }
                requests.push((key, access, fields));
            }
        }

        // Aquire all the locks
        let guards: BTreeMap<_, _> = locks
            .into_iter()
            .map(|(key, (component, access))| (key, ComponentGuard::lock(component, access)))
            .collect();

        // Get from the locks of the components all the pointers in the other they were
        // requested
        let mut pointers = Vec::new();
        for (key, access, fields) in requests {
            for &field in fields {
                pointers.push(
                    guards[&key]
                        .component()
                        .query_field(field, access)
                        .map_err(EntityError::from)?,
                );
            }
        }
        // All guards remain in scope through the one callback.
        action(&pointers);
        Ok(())
    }

    /// Loads or reuses each requested asset, then calls `action` once with
    /// pointers to complete asset data in request order, including duplicates.
    /// An empty query calls `action` with an empty slice.
    ///
    /// A load failure skips the callback; assets already loaded remain cached.
    /// A concurrent reset between loading and locking can return
    /// [`SceneError::AssetNotFound`]. Once locked, assets remain alive
    /// throughout the callback. Data strings use the same cache semantics
    /// as [`Self::get_asset_id`].
    ///
    /// # Pointer and callback contract
    ///
    /// Pointers may only be read during `action`, using the correct plugin
    /// asset type. They must never be mutated. Do not call scene methods
    /// from `action` or wait for work needing the asset collection lock.
    /// The lock releases on return or panic.
    ///
    /// # Design decisions
    ///
    /// Assets are immutable, so one collection read lock protects every
    /// pointer; there are no per-asset locks to sort. Loading finishes
    /// before taking that lock, avoiding read-to-write upgrades. This
    /// reuses the existing cache and needs no additional ownership or
    /// synchronization machinery.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wasserxr::{scene::Scene, ids::*, errors::SceneError};
    /// # struct Settings { scale: f32 }
    /// # fn read(scene: &Scene, plugin: PluginID, settings: AssetTypeID) -> Result<(), SceneError> {
    /// scene.query_assets(&[(plugin, settings, "default")], |assets| {
    ///     // SAFETY: This plugin's settings asset uses the `Settings` layout.
    ///     let settings = unsafe { &*assets[0].cast::<Settings>() };
    ///     println!("{}", settings.scale);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query_assets(
        &self,
        query: &[AssetQuery<'_>],
        action: impl FnOnce(&[*const c_void]),
    ) -> Result<(), SceneError> {
        let ids = query
            .iter()
            .map(|&(plugin, asset_type, data)| self.get_asset_id(plugin, asset_type, data))
            .collect::<Result<Vec<_>, _>>()?;
        let assets = self.assets.read().expect("scene asset lock poisoned");
        let pointers = ids
            .into_iter()
            .map(|id| {
                assets
                    .get(id)
                    .map(Asset::data)
                    .ok_or(SceneError::AssetNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;
        action(&pointers);
        Ok(())
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
