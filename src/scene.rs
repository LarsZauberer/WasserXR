use std::{
    ffi::c_void,
    path::Path,
    sync::{RwLock, mpsc},
};

use crate::{
    definitions::{fields::TypeHint, plugins::PluginDefinition},
    errors::{
        AssetError, ComponentError, PluginCompatibilityError, PluginError, SceneError, SystemError,
    },
    field::AccessRequest,
    ids::{
        AssetFieldID, AssetFieldTypeID, AssetID, AssetSlot, AssetTypeID, ComponentID,
        ComponentTypeID, EntityID, EntitySlot, FieldID, FieldTypeID, PluginID, PluginSlot,
        SystemID, SystemTypeID, TypeID,
    },
    logging::{LogEntry, LogHandler, LogLevel, LogManager},
    private::{
        assets::Asset,
        entities::{self, Entity},
        id_store::IDStore,
        manifests::{Manifest, plugins::PluginManifest, type_id_requests::TypeIDRequestManifest},
        plugins::Plugin,
        system::System,
        system_storage::SystemStorage,
        system_storage_snapshot::SystemStorageSnapshot,
        thread_pool::ThreadPool,
    },
    utils::ring::Ring,
};

pub(crate) type EntityStorage = IDStore<String, EntitySlot, Entity>;

/// # Design Decision
///
/// A plugin doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
type PluginStorage = IDStore<String, PluginSlot, Plugin>;

/// # Design Decision
///
/// An asset doesn't require an RwLock since it is a read-only object. There are
/// no operations that require exclusive access to it.
pub(crate) type AssetStorage = IDStore<(AssetTypeID, String), AssetSlot, Asset>;

/// One component requirement: plugin, component type, lock mode, and ordered
/// field types. An empty field slice still requires and locks the component.
pub type ComponentQuery<'a> = (ComponentTypeID, AccessRequest, &'a [FieldTypeID]);

/// One complete asset requested by plugin, asset type, and cache data string.
pub type AssetQuery<'a> = (AssetTypeID, &'a str);

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
    logging: LogManager,
    systems: RwLock<SystemStorage>,
    entities: RwLock<EntityStorage>,
    plugins: RwLock<PluginStorage>,
    assets: RwLock<AssetStorage>,
    thread_pool: ThreadPool,
}

impl Scene {
    /// Submits a message to this scene's asynchronous logger.
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        self.logging.log(level, message.into());
    }

    /// Registers a callback for subsequently processed log entries.
    ///
    /// # Safety
    ///
    /// `handler.data` must remain valid and safe to access from the logger
    /// thread until this scene has been dropped. The callback must not retain
    /// `entry` or any pointer obtained from it after returning, and must not
    /// unwind across the `extern "C"` boundary.
    pub unsafe fn add_log_handler(&self, handler: LogHandler) {
        self.logging.add_handler(handler);
    }

    /// Returns a snapshot of log entries processed before this request.
    pub fn get_logs(&self) -> Ring<LogEntry> {
        self.logging.get_logs()
    }

    /// Logs a detailed diagnostic message.
    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Logs an informational message.
    pub fn info(&self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Logs a warning message.
    pub fn warning(&self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    /// Logs an error message.
    pub fn error(&self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }

    /// Creates a new empty scene
    ///
    /// # Panics
    ///
    /// Panics if one of the worker threads cannot be created.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new entity and returns it's handle. The handle will be unique
    /// to every other entity ever created within this scene.
    pub fn add_entity(&self) -> EntityID {
        EntityID(
            self.entities
                .write()
                .expect("scene entity lock poisoned")
                .insert(Entity::new()),
        )
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
            .remove(id.0)
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
            .map(EntityID)
            .collect()
    }

    /// Removes all systems from the scene and runs their detachers.
    pub fn reset_systems(&self) {
        let systems = self
            .systems
            .write()
            .expect("scene system lock poisoned")
            .drain()
            .collect::<Vec<_>>();
        for system in systems {
            self.run_detacher(system);
        }
    }

    /// Runs a system's detacher after the system has been removed from storage.
    fn run_detacher(&self, system: System) {
        if let Some((detacher, type_ids)) = system.detacher() {
            unsafe { detacher(self, type_ids.as_ptr(), type_ids.len()) };
        }
    }

    /// Removes all entities and their components from the scene.
    pub fn reset_entities(&self) {
        self.entities
            .write()
            .expect("scene entity lock poisoned")
            .clear();
    }

    /// Removes all cached assets from the scene.
    pub fn reset_assets(&self) {
        self.assets
            .write()
            .expect("scene asset lock poisoned")
            .clear();
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
        Ok(PluginID(plugins.insert_named(name, new_plugin)))
    }

    /// Runs an action with an entity while holding the entity collection's
    /// read lock, preventing the entity from being removed during the action.
    fn with_entity<T>(
        &self,
        id: EntitySlot,
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
        self.add_plugin(Plugin::load_static(manifest))
    }

    /// Get the handle of a plugin ([`PluginID`]) by searching for the name of a
    /// plugin
    pub fn resolve_plugin_id(&self, name: &str) -> Option<PluginID> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .resolve_id(name)
            .map(PluginID)
    }

    /// Returns the name of the plugin identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not identify a loaded plugin.
    pub fn get_plugin_name(&self, id: PluginID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(id.0)
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
            .map(PluginID)
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
            .get(plugin_id.0)
            .and_then(|plugin| plugin.resolve_component_type_slot(name))
            .map(|component| ComponentTypeID(plugin_id.0, component))
            .ok_or(SceneError::NoComponentType)
    }

    /// Resolves a component field type name within a component type manifest.
    pub fn resolve_field_type_id(
        &self,
        component_type_id: ComponentTypeID,
        name: &str,
    ) -> Result<FieldTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(component_type_id.0)
            .and_then(|plugin| plugin.resolve_field_type_slot(component_type_id.1, name))
            .map(|field| FieldTypeID(component_type_id.0, component_type_id.1, field))
            .ok_or(SceneError::NoComponentType)
    }

    /// Returns the primitive type hint for a component field type.
    pub fn get_field_type(&self, field_type_id: FieldTypeID) -> Result<TypeHint, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let component = plugins
            .get(field_type_id.0)
            .and_then(|plugin| plugin.get_component(field_type_id.1))
            .ok_or(SceneError::NoComponentType)?;

        component
            .fields
            .get(field_type_id.2)
            .map(|field| field.type_hint)
            .ok_or_else(|| SceneError::from(ComponentError::FieldNotFound))
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
            .get(plugin_id.0)
            .and_then(|plugin| plugin.resolve_asset_type_slot(name))
            .map(|asset| AssetTypeID(plugin_id.0, asset))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Returns the name of an asset type identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if either ID does not identify a loaded plugin or one of its
    /// asset types.
    pub fn get_asset_name(&self, id: AssetTypeID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(id.0)
            .and_then(|plugin| plugin.get_asset_name(id.1))
            .expect("IDs do not identify a loaded asset type") as *const str;

        // SAFETY: Plugin manifests are never removed from a Scene and their
        // owned names are never mutated, so the string data remains valid for
        // the Scene's lifetime after releasing the collection lock.
        unsafe { &*name }
    }

    /// Resolves an asset field type name within an asset type manifest.
    pub fn resolve_asset_field_type_id(
        &self,
        asset_type_id: AssetTypeID,
        name: &str,
    ) -> Result<AssetFieldTypeID, SceneError> {
        self.plugins
            .read()
            .expect("scene plugin lock poisoned")
            .get(asset_type_id.0)
            .and_then(|plugin| plugin.resolve_asset_field_type_slot(asset_type_id.1, name))
            .map(|field| AssetFieldTypeID(asset_type_id.0, asset_type_id.1, field))
            .ok_or(SceneError::AssetNotFound)
    }

    /// Returns the primitive type hint for an asset field type.
    pub fn get_asset_field_type(
        &self,
        field_type_id: AssetFieldTypeID,
    ) -> Result<TypeHint, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let asset = plugins
            .get(field_type_id.0)
            .and_then(|plugin| plugin.get_asset(field_type_id.1))
            .ok_or(SceneError::AssetNotFound)?;

        asset
            .fields
            .get(field_type_id.2)
            .map(|field| field.type_hint)
            .ok_or_else(|| SceneError::from(AssetError::FieldNotFound))
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
            .get(plugin_id.0)
            .and_then(|plugin| plugin.resolve_system_type_slot(name))
            .map(|system| SystemTypeID(plugin_id.0, system))
            .ok_or(SceneError::SystemNotFound)
    }

    /// Returns the name of a system type identified by `id`.
    ///
    /// # Panics
    ///
    /// Panics if either ID does not identify a loaded plugin or one of its
    /// system types.
    pub fn get_system_name(&self, id: SystemTypeID) -> &str {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let name = plugins
            .get(id.0)
            .and_then(|plugin| plugin.get_system_name(id.1))
            .expect("IDs do not identify a loaded system type") as *const str;

        // SAFETY: Plugin manifests are never removed from a Scene and their
        // owned names are never mutated, so the string data remains valid for
        // the Scene's lifetime after releasing the collection lock.
        unsafe { &*name }
    }

    /// Resolves the type IDs requested by a system manifest.
    pub fn resolve_requested_type_ids(
        &self,
        system_type_id: SystemTypeID,
    ) -> Result<Vec<TypeID>, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let plugin = plugins
            .get(system_type_id.0)
            .ok_or(SceneError::SystemNotFound)?;
        let manifest = plugin
            .get_system(system_type_id.1)
            .ok_or(SceneError::SystemNotFound)?;

        manifest
            .type_id_requests
            .iter()
            .map(|request| {
                let type_id = match request {
                    TypeIDRequestManifest::ComponentTypeID { component } => plugin
                        .resolve_component_type_slot(component)
                        .map(|component| ComponentTypeID(system_type_id.0, component).into()),
                    TypeIDRequestManifest::FieldTypeID { component, field } => plugin
                        .resolve_component_type_slot(component)
                        .and_then(|component| {
                            plugin
                                .resolve_field_type_slot(component, field)
                                .map(|field| {
                                    TypeID::from(FieldTypeID(system_type_id.0, component, field))
                                })
                        }),
                    TypeIDRequestManifest::AssetTypeID { asset } => plugin
                        .resolve_asset_type_slot(asset)
                        .map(|asset| AssetTypeID(system_type_id.0, asset).into()),
                    TypeIDRequestManifest::AssetFieldTypeID { asset, field } => {
                        plugin.resolve_asset_type_slot(asset).and_then(|asset| {
                            plugin
                                .resolve_asset_field_type_slot(asset, field)
                                .map(|field| {
                                    TypeID::from(AssetFieldTypeID(system_type_id.0, asset, field))
                                })
                        })
                    }
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
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentID, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let manifest = plugins
            .get(component_type_id.0)
            .and_then(|plugin| plugin.get_component(component_type_id.1))
            .ok_or(SceneError::NoComponentType)?;

        self.with_entity(entity_id.0, |entity| {
            entity
                .add_component(component_type_id, manifest)
                .map(|component| ComponentID(entity_id.0, component))
                .map_err(SceneError::EntityError)
        })
    }

    /// Returns an arbitrary entity with the requested component, creating one
    /// if none exists.
    ///
    /// Existing duplicate components are left untouched. Concurrent calls to
    /// this function cannot create duplicates.
    pub fn ensure_singleton(
        &self,
        component_type_id: ComponentTypeID,
    ) -> Result<EntityID, SceneError> {
        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let manifest = plugins
            .get(component_type_id.0)
            .and_then(|plugin| plugin.get_component(component_type_id.1))
            .ok_or(SceneError::NoComponentType)?;
        let mut entities = self.entities.write().expect("scene entity lock poisoned");

        if let Some((id, _)) = entities
            .iter()
            .find(|(_, entity)| entity.resolve_component_slot(component_type_id).is_ok())
        {
            return Ok(EntityID(id));
        }

        let entity_slot = entities.insert(Entity::new());
        entities
            .get(entity_slot)
            .expect("entity was just inserted")
            .add_component(component_type_id, manifest)
            .map_err(SceneError::from)?;
        Ok(EntityID(entity_slot))
    }

    /// Remove a component type from an entity
    ///
    /// The function may fail, if the [`EntityID`] cannot be found in the scene.
    pub fn remove_component(&self, component: ComponentID) -> Result<(), SceneError> {
        self.with_entity(component.0, |entity| {
            entity
                .remove_component(component.1)
                .map_err(SceneError::from)
        })
    }

    /// Returns all the component names of the given [`EntityID`]
    pub fn get_components(&self, entity_id: EntityID) -> Result<Vec<ComponentID>, SceneError> {
        self.with_entity(entity_id.0, |entity| {
            Ok(entity
                .get_component_slots()
                .into_iter()
                .map(|component| ComponentID(entity_id.0, component))
                .collect())
        })
    }

    /// Return the name of some component handle
    pub fn get_component_name(&self, component_id: ComponentID) -> Result<String, SceneError> {
        self.with_entity(component_id.0, |entity| {
            entity
                .get_component_name(component_id.1)
                .map_err(SceneError::from)
        })
    }

    /// Resolves a component type attached to an entity to its [`ComponentID`].
    pub fn resolve_component_id(
        &self,
        entity_id: EntityID,
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentID, SceneError> {
        self.with_entity(entity_id.0, |entity| {
            entity
                .resolve_component_slot(component_type_id)
                .map(|component| ComponentID(entity_id.0, component))
                .map_err(SceneError::from)
        })
    }

    /// Resolves a field type within a component to its [`FieldID`].
    pub fn resolve_field_id(
        &self,
        component_id: ComponentID,
        field_type_id: FieldTypeID,
    ) -> Result<FieldID, SceneError> {
        self.with_entity(component_id.0, |entity| {
            entity
                .resolve_field_slot(component_id.1, field_type_id)
                .map(|field| FieldID(component_id.0, component_id.1, field))
                .map_err(SceneError::from)
        })
    }

    /// Ensures an entity with the requested component exists, then calls
    /// `action` with that component's requested fields in field order.
    ///
    /// If multiple entities have the component, an arbitrary one's fields are
    /// passed to `action`; all matches are locked in the global order used by
    /// [`Self::with_component_fields`]. Pointers follow the same access and
    /// lifetime contract.
    ///
    /// # Errors
    ///
    /// For a query with fields, returns [`SceneError::EntityNotFound`] if all
    /// matching entities or components are removed after ensuring the singleton
    /// but before the component query acquires its locks. In that case,
    /// `action` is not called.
    pub fn with_singleton_fields(
        &self,
        query: ComponentQuery<'_>,
        action: impl FnOnce(&[*mut c_void]),
    ) -> Result<(), SceneError> {
        let (component_type_id, _, fields) = query;
        self.ensure_singleton(component_type_id)?;
        let mut queried = false;
        self.with_component_fields(std::slice::from_ref(&query), |pointers| {
            // Component queries flatten the fields from every match. Expose only the
            // first match, using `get` so concurrent removal returns an error instead
            // of panicking on an out-of-bounds slice.
            if let Some(pointers) = pointers.get(..fields.len()) {
                queried = true;
                action(pointers);
            }
        })?;
        queried.then_some(()).ok_or(SceneError::EntityNotFound)
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
    /// # fn update(scene: &Scene, component: ComponentTypeID,
    /// #           counter: FieldTypeID) -> Result<(), SceneError> {
    /// scene.with_component_fields(&[(component, AccessRequest::Write, &[counter])], |fields| {
    ///     for &field in fields {
    ///         // SAFETY: This plugin defines `counter` as a mutable u32.
    ///         unsafe { *field.cast::<u32>() += 1 };
    ///     }
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_component_fields(
        &self,
        query: &[ComponentQuery<'_>],
        action: impl FnOnce(&[*mut c_void]),
    ) -> Result<(), SceneError> {
        if query.is_empty() {
            action(&[]);
            return Ok(());
        }

        let entities = self.entities.read().expect("scene entity lock poisoned");
        entities::with_component_fields(entities.iter(), query, action).map_err(SceneError::from)
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
    /// # fn read(scene: &Scene, settings: AssetTypeID) -> Result<(), SceneError> {
    /// scene.with_asset_fields(&[(settings, "default")], |assets| {
    ///     // SAFETY: This plugin's settings asset uses the `Settings` layout.
    ///     let settings = unsafe { &*assets[0].cast::<Settings>() };
    ///     println!("{}", settings.scale);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_asset_fields(
        &self,
        query: &[AssetQuery<'_>],
        action: impl FnOnce(&[*const c_void]),
    ) -> Result<(), SceneError> {
        let ids = query
            .iter()
            .map(|&(asset_type, data)| self.get_asset_id(asset_type, data))
            .collect::<Result<Vec<_>, _>>()?;
        let assets = self.assets.read().expect("scene asset lock poisoned");
        let pointers = ids
            .into_iter()
            .map(|id| {
                assets
                    .get(id.2)
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
        asset_type_id: AssetTypeID,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        self.assets
            .read()
            .expect("scene asset lock poisoned")
            .resolve_id(&(asset_type_id, data_string.to_owned()))
            .map(|asset| AssetID(asset_type_id.0, asset_type_id.1, asset))
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
            .get(asset_id.2)
            .ok_or(SceneError::AssetNotFound)?
            .resolve_field_slot(field_type_id)
            .map(|field| AssetFieldID(asset_id.0, asset_id.1, asset_id.2, field))
            .map_err(SceneError::from)
    }

    /// Resolves the asset id and if it doesn't exist, it will try to load the
    /// asset
    pub fn get_asset_id(
        &self,
        asset_type_id: AssetTypeID,
        data_string: &str,
    ) -> Result<AssetID, SceneError> {
        let key = (asset_type_id, data_string.to_owned());
        if let Some(id) = self
            .assets
            .read()
            .expect("scene asset lock poisoned")
            .resolve_id(&key)
        {
            return Ok(AssetID(asset_type_id.0, asset_type_id.1, id));
        }

        let plugins = self.plugins.read().expect("scene plugin lock poisoned");
        let manifest = plugins
            .get(asset_type_id.0)
            .and_then(|plugin| plugin.get_asset(asset_type_id.1))
            .ok_or(SceneError::AssetNotFound)?;

        let mut assets = self.assets.write().expect("scene asset lock poisoned");
        if let Some(id) = assets.resolve_id(&key) {
            return Ok(AssetID(asset_type_id.0, asset_type_id.1, id));
        }
        let asset = Asset::new(manifest, asset_type_id).map_err(SceneError::from)?;
        let asset = assets.insert_named(key, asset);
        Ok(AssetID(asset_type_id.0, asset_type_id.1, asset))
    }

    /// Gets an existing concrete system ID from its plugin and system type.
    pub fn get_system_id(&self, system_type_id: SystemTypeID) -> Result<SystemID, SceneError> {
        self.systems
            .read()
            .expect("scene system lock poisoned")
            .resolve_id(&system_type_id)
            .ok_or(SceneError::SystemNotFound)
    }

    /// Adds a system to the scene and returns its concrete ID.
    pub fn add_system(&self, system_type_id: SystemTypeID) -> Result<SystemID, SceneError> {
        let key = system_type_id;
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
        let type_ids = self.resolve_requested_type_ids(system_type_id)?;
        let (system_id, attacher) = {
            let plugins = self.plugins.read().expect("scene plugin lock poisoned");
            let plugin = plugins
                .get(system_type_id.0)
                .ok_or(SceneError::SystemNotFound)?;
            let manifest = plugin
                .get_system(system_type_id.1)
                .ok_or(SceneError::SystemNotFound)?;
            // Resolve dependency names to stable keys so storage can check presence
            // and build the dependency graph without string lookups.
            let requires = manifest
                .requires
                .iter()
                .map(|name| {
                    plugin
                        .resolve_system_type_slot(name)
                        .map(|system| SystemTypeID(system_type_id.0, system))
                        .ok_or_else(|| {
                            SceneError::from(SystemError::DependencyNotFound(name.clone()))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let wanted_by = manifest
                .wanted_by
                .iter()
                .map(|name| {
                    plugin
                        .resolve_system_type_slot(name)
                        .map(|system| SystemTypeID(system_type_id.0, system))
                        .ok_or_else(|| {
                            SceneError::from(SystemError::DependencyNotFound(name.clone()))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut systems = self.systems.write().expect("scene system lock poisoned");
            if systems.resolve_id(&key).is_some() {
                return Err(SystemError::AlreadyExists.into());
            }
            systems.add_system(key, manifest, type_ids, requires, wanted_by)?
        };

        if let Some((attacher, type_ids)) = attacher {
            unsafe { attacher(self, type_ids.as_ptr(), type_ids.len()) };
        }
        Ok(system_id)
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
        self.run_detacher(system);
        Ok(())
    }

    /// Runs every system once, in parallel where dependencies allow it.
    ///
    /// Additions and removals made by a runner take effect on the next tick.
    /// Returns only after every scheduled system has finished and the scene's
    /// thread pool is idle.
    pub fn tick(&mut self) {
        let mut snapshot =
            SystemStorageSnapshot::new(&self.systems.read().expect("scene system lock poisoned"));
        let system_count = snapshot.len();
        let scene = self as *const Self as usize;
        let (completed, completions) = mpsc::channel();

        let schedule = |(id, (runner, type_ids)): (
            SystemID,
            (crate::definitions::systems::Runner, Vec<TypeID>),
        )| {
            let completed = completed.clone();
            self.thread_pool.execute(move || {
                // SAFETY: `tick` holds an exclusive borrow of the Scene and
                // waits for the pool to become idle before returning, so the
                // Scene remains alive and stationary for the whole callback.
                unsafe { runner(scene as *const Self, type_ids.as_ptr(), type_ids.len()) };
                completed
                    .send(id)
                    .expect("tick stopped receiving system completions");
            });
        };

        for execution in snapshot.take_ready() {
            schedule(execution);
        }
        for _ in 0..system_count {
            let id = completions
                .recv()
                .expect("system task ended without reporting completion");
            for execution in snapshot.complete(id) {
                schedule(execution);
            }
        }
        self.thread_pool.wait_until_idle();
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        let systems = self
            .systems
            .get_mut()
            .expect("scene system lock poisoned")
            .drain()
            .collect::<Vec<_>>();
        for system in systems {
            self.run_detacher(system);
        }
    }
}
