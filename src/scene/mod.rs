//! Scene runtime, ECS storage, plugin loading, and typed query APIs.

/// Asset type schemas and asset field query support.
pub mod assets;
/// Component schemas, field metadata, and serialization byte buffers.
pub mod component;
pub(crate) mod entity;
/// Scene logging types, callbacks, and exported log macros.
pub mod logging;
pub(crate) mod plugin;
pub mod query;
/// Type-erased scene resources.
pub mod resource;
pub(crate) mod serialization;
pub(crate) mod system;

use assets::{Asset, AssetType};
use component::{Component, FieldType};
use entity::Entity;
use plugin::Plugin;
use query::{SceneQuery, SceneQueryMut};
use resource::Resource;
use system::System;

use crate::error::{AssetError, PluginError, SceneError};
use crate::scene::logging::LogManager;
use crate::scene::serialization::{ComponentData, SceneData, SystemData};
use crate::scene::system::{Runner, Selector};

use std::ffi::c_void;
use std::{collections::HashMap, fs, path::Path};

use uuid::Uuid;

enum DeferredCall {
    Reload,
    Load(Vec<u8>),
    UnloadPlugin(String),
    RemoveSystem(String),
}

/// Runtime container for entities, components, systems, resources, plugins, and assets.
///
/// `Scene` is the main API entry point for applications and plugins.
///
/// # Examples
///
/// ```
/// let mut scene = wasserxr::scene::Scene::new();
/// let entity = scene.add_entity();
///
/// scene.set_entity_name(entity, "Player".to_owned()).unwrap();
/// assert_eq!(scene.get_entity_name(entity).unwrap(), "Player");
/// ```
pub struct Scene {
    entities: HashMap<Uuid, Entity>,
    plugins: HashMap<String, Plugin>,
    systems: HashMap<String, System>,
    components: HashMap<Uuid, HashMap<String, Component>>,
    resources: HashMap<String, Resource>,
    asset_types: HashMap<String, AssetType>,
    assets: HashMap<String, HashMap<String, Asset>>,

    // Deferred Calls
    deferred_calls: Vec<DeferredCall>,

    // Logging
    log_manager: LogManager,

    // Flags
    is_ticking: bool,
    should_exit: bool,
}

impl Default for Scene {
    fn default() -> Self {
        let mut plugins = HashMap::new();
        plugins.insert("".to_owned(), Plugin::new_static());
        Self {
            entities: HashMap::new(),
            plugins,
            systems: HashMap::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
            asset_types: HashMap::new(),
            assets: HashMap::new(),

            // Deferred Calls
            deferred_calls: Vec::new(),

            // Logging
            log_manager: LogManager::new("WasserXR".to_owned()),

            // Flags
            is_ticking: false,
            should_exit: false,
        }
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        self.clear_assets();
    }
}

impl Scene {
    /// Creates an empty scene with the built-in static plugin registered.
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all assets, systems, entities, and components from the scene.
    ///
    /// Loaded plugins and resources stay registered.
    pub fn reset(&mut self) -> Result<(), SceneError> {
        self.clear_assets();

        // Kill all systems
        let systems: Vec<String> = self.systems.keys().cloned().collect();
        for i in systems {
            self.remove_system(&i)?;
        }

        // Kill all entities
        let entities: Vec<Uuid> = self.entities.keys().copied().collect();
        for i in entities {
            self.remove_entity(i)?;
        }

        // Plugins will stay as they are
        crate::info!(self, "Scene reset");
        Ok(())
    }

    /// Serializes entities, systems, and components into WasserXR scene bytes.
    ///
    /// Assets, resources, and loaded plugin handles are not serialized.
    pub fn serialize(&self) -> Result<Vec<u8>, SceneError> {
        let entities: Vec<_> = self.entities.values().map(Entity::serialize).collect();

        let systems: Vec<_> = self.systems.values().map(System::serialize).collect();

        let components: Vec<_> = self
            .components
            .iter()
            .flat_map(|(entity_id, components)| {
                components
                    .values()
                    .map(|component| component.serialize(*entity_id))
                    .collect::<Vec<_>>()
            })
            .collect();

        SceneData {
            entities,
            systems,
            components,
        }
        .encode()
        .map_err(SceneError::Serialization)
    }

    /// Replaces the scene state with serialized WasserXR scene bytes.
    ///
    /// This calls `reset` first, then recreates entities, systems, and
    /// components from the decoded scene data. Invalid individual systems or
    /// components are skipped with a warning.
    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), SceneError> {
        self.reset()?;

        let scene_data = SceneData::decode(data).map_err(SceneError::Deserialization)?;

        for entity_data in scene_data.entities {
            let entity_id = entity_data.id;
            if self.entities.contains_key(&entity_id) {
                crate::warn!(
                    self,
                    "Entity `{}` is duplicated in serialized scene",
                    entity_id
                );
                continue;
            }

            self.entities
                .insert(entity_id, Entity::deserialize(entity_data));
            self.components.insert(entity_id, HashMap::new());
        }

        for system_data in scene_data.systems {
            let system_id = system_data.id.clone();
            if let Err(error) = self.add_system_from_data(system_data) {
                crate::warn!(
                    self,
                    "System `{}` could not be deserialized: {:?}",
                    system_id,
                    error
                );
            }
        }

        for component_data in scene_data.components {
            let component_id = component_data.id.clone();
            let entity_id = component_data.entity_id;

            if let Err(error) = self.add_component_from_data(component_data) {
                crate::warn!(
                    self,
                    "Component `{}` on entity `{}` could not be deserialized: {:?}",
                    component_id,
                    entity_id,
                    error
                );
            }
        }

        Ok(())
    }

    /// Writes serialized scene bytes to `path`.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SceneError> {
        let data = self.serialize()?;
        fs::write(path, data).map_err(|error| SceneError::FileIo(error.to_string()))
    }

    /// Loads serialized scene bytes from `path`.
    ///
    /// If called while a system is ticking, the load is deferred until the
    /// current system runner returns.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SceneError> {
        let data = fs::read(path).map_err(|error| SceneError::FileIo(error.to_string()))?;
        if self.is_ticking {
            self.deferred_calls.push(DeferredCall::Load(data));
            return Ok(());
        }

        self.deserialize(&data)
    }

    /// Returns sorted ids for dynamically loaded plugins.
    ///
    /// The built-in static plugin is intentionally omitted.
    pub fn get_plugins(&self) -> Vec<String> {
        let mut plugins: Vec<String> = self
            .plugins
            .keys()
            .filter(|id| !id.is_empty())
            .cloned()
            .collect();
        plugins.sort();
        plugins
    }

    fn plugins_dynamic_first(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins
            .values()
            .filter(|plugin| !plugin.get_id().is_empty())
            .chain(
                self.plugins
                    .values()
                    .filter(|plugin| plugin.get_id().is_empty()),
            )
    }

    /// Loads a dynamic plugin from `path`.
    ///
    /// Plugin symbols are resolved lazily when systems, components, or asset
    /// types are created.
    pub fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        if self.plugins.contains_key(&path) {
            crate::warn!(self, "Plugin `{}` is already loaded", path);
            return Err(SceneError::PluginAlreadyLoaded);
        }

        let plugin = match Plugin::new(path.to_owned()) {
            Ok(plugin) => plugin,
            Err(error) => {
                match &error {
                    PluginError::LinkingError(message) => {
                        crate::error!(self, "Plugin `{}` could not be loaded: {}", path, message);
                    }
                    PluginError::InvalidSymbol => {
                        crate::error!(self, "Plugin path contains a null byte");
                    }
                    PluginError::MissingSymbol(symbol) => {
                        crate::error!(self, "Plugin `{}` missed symbol `{}`", path, symbol);
                    }
                    PluginError::NotFound => {
                        crate::error!(self, "Plugin `{}` could not be found", path);
                    }
                }
                return Err(SceneError::PluginLoading(error));
            }
        };
        self.plugins.insert(path.to_owned(), plugin);

        crate::info!(self, "Plugin `{}` loaded", path);
        Ok(())
    }

    /// Unloads a dynamic plugin and removes its systems, asset types, and components.
    ///
    /// If called while a system is ticking, the unload is deferred until the
    /// current system runner returns.
    pub fn unload_plugin(&mut self, path: &str) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls
                .push(DeferredCall::UnloadPlugin(path.to_owned()));
            return Ok(());
        }

        if path.is_empty() {
            crate::warn!(self, "Static plugin cannot be unloaded");
            return Err(SceneError::StaticPluginUnload);
        }

        if !self.plugins.contains_key(path) {
            crate::warn!(self, "Plugin `{}` is not loaded", path);
            return Err(SceneError::PluginNotFound);
        }

        // Unload all systems that are still loaded with this plugin
        let systems: Vec<String> = self
            .systems
            .values()
            .filter(|system| system.get_plugin_id() == path)
            .map(|system| system.get_id().to_owned())
            .collect();

        for system_id in systems {
            self.remove_system(&system_id)?;
        }

        // Unload all the asset types that are still loaded with this plugin
        let asset_types: Vec<String> = self
            .asset_types
            .values()
            .filter(|asset_type| asset_type.get_plugin_id() == path)
            .map(|asset_type| asset_type.get_id().to_owned())
            .collect();

        for asset_type in asset_types {
            self.remove_asset_type(&asset_type);
        }

        // Unload all the components that are still loaded with this plugin
        let components: Vec<(Uuid, String)> = self
            .components
            .iter()
            .flat_map(|(entity, components)| {
                components
                    .values()
                    .filter(|component| component.get_plugin_id() == path)
                    .map(|component| (*entity, component.get_id().to_owned()))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (entity, component_id) in components {
            self.remove_component(entity, &component_id)?;
        }

        // Remove the plugin itself
        self.plugins.remove(path);

        crate::info!(self, "Plugin `{}` unloaded", path);
        Ok(())
    }

    /// Adds a new entity and returns its id.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut scene = wasserxr::scene::Scene::new();
    /// let entity = scene.add_entity();
    ///
    /// assert!(scene.get_entities().contains(&entity));
    /// ```
    pub fn add_entity(&mut self) -> Uuid {
        let entity = Entity::new();
        let uuid = entity.get_id();
        self.entities.insert(uuid, entity);
        self.components.insert(uuid, HashMap::new());
        crate::info!(self, "Entity `{}` added", uuid);
        uuid
    }

    /// Returns all entity ids in sorted order.
    pub fn get_entities(&self) -> Vec<Uuid> {
        let mut entities: Vec<Uuid> = self.entities.keys().copied().collect();
        entities.sort();
        entities
    }

    /// Removes an entity and all of its components.
    pub fn remove_entity(&mut self, id: Uuid) -> Result<(), SceneError> {
        let Some(_) = self.entities.remove(&id) else {
            crate::warn!(self, "Entity `{}` was not found for removal", id);
            return Err(SceneError::EntityNotFound);
        };

        let Some(components) = self.components.remove(&id) else {
            crate::error!(
                self,
                "Entity `{}` had no components carrier associated. This is a bug. Please report it",
                id
            );
            return Ok(());
        };

        for (component_id, component) in components {
            drop(component);
            crate::info!(
                self,
                "Component `{}` removed from entity `{}`",
                component_id,
                id
            );
        }

        crate::info!(self, "Entity `{}` removed", id);
        Ok(())
    }

    fn get_entity(&self, id: Uuid) -> Result<&Entity, SceneError> {
        match self.entities.get(&id) {
            Some(entity) => Ok(entity),
            None => {
                crate::warn!(self, "Entity `{}` was not found", id);
                Err(SceneError::EntityNotFound)
            }
        }
    }

    fn get_entity_mut(&mut self, id: Uuid) -> Result<&mut Entity, SceneError> {
        if !self.entities.contains_key(&id) {
            crate::warn!(self, "Entity `{}` was not found", id);
            return Err(SceneError::EntityNotFound);
        }

        Ok(self
            .entities
            .get_mut(&id)
            .expect("entity was checked before mutable lookup"))
    }

    /// Returns the display name for an entity.
    pub fn get_entity_name(&self, id: Uuid) -> Result<&str, SceneError> {
        let entity = self.get_entity(id)?;
        Ok(entity.get_name())
    }

    /// Replaces the display name for an entity.
    pub fn set_entity_name(&mut self, id: Uuid, name: String) -> Result<(), SceneError> {
        let entity = self.get_entity_mut(id)?;
        entity.set_name(name);
        crate::info!(self, "Entity `{}` renamed", id);
        Ok(())
    }

    /// Adds a system by id and priority.
    ///
    /// The runtime looks for `wxr_system_<id>` and its companion binding
    /// functions in loaded plugins. Higher priorities run earlier in `tick`.
    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        self.add_system_from_data(SystemData { id, priority })
    }

    fn add_system_from_data(&mut self, data: SystemData) -> Result<(), SceneError> {
        let id = data.id.clone();
        if self.systems.contains_key(&id) {
            crate::warn!(self, "System `{}` already exists", id);
            return Err(SceneError::SystemAlreadyExists);
        }

        let system: Option<System> = self
            .plugins_dynamic_first()
            .find_map(|plugin| System::deserialize(data.clone(), plugin, self).ok());

        match system {
            Some(system) => {
                let system_id = system.get_id().to_owned();
                let attacher = system.get_attacher();
                self.set_logger(system_id.clone());
                unsafe {
                    attacher(self as *mut Scene);
                }
                self.reset_logger();
                crate::info!(self, "System `{}` added", system_id);
                self.systems.insert(id, system);
                Ok(())
            }
            None => {
                crate::warn!(self, "System `{}` could not be created", id);
                Err(SceneError::SystemCreation)
            }
        }
    }

    /// Removes a system and calls its detach binding.
    ///
    /// If called while a system is ticking, the removal is deferred until the
    /// current system runner returns.
    pub fn remove_system(&mut self, id: &str) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls
                .push(DeferredCall::RemoveSystem(id.to_owned()));
            return Ok(());
        }

        let Some(system) = self.systems.remove(id) else {
            crate::warn!(self, "System `{}` was not found for removal", id);
            return Err(SceneError::SystemNotFound);
        };

        let detacher = system.get_detacher();
        self.set_logger(id.to_owned());
        unsafe {
            detacher(self as *mut Scene);
        }
        self.reset_logger();
        crate::info!(self, "System `{}` removed", id);
        Ok(())
    }

    /// Returns all system ids in sorted order.
    pub fn get_systems(&self) -> Vec<String> {
        let mut systems: Vec<String> = self.systems.keys().cloned().collect();
        systems.sort();
        systems
    }

    /// Returns a system's priority.
    pub fn get_system_priority(&self, system_id: &str) -> Result<usize, SceneError> {
        match self.systems.get(system_id) {
            Some(system) => Ok(system.get_priority()),
            None => {
                crate::warn!(
                    self,
                    "System `{}` was not found for priority lookup",
                    system_id
                );
                Err(SceneError::SystemNotFound)
            }
        }
    }

    /// Returns the id of the plugin that provided a system.
    pub fn get_system_plugin_id(&self, system_id: &str) -> Result<&str, SceneError> {
        match self.systems.get(system_id) {
            Some(system) => Ok(system.get_plugin_id()),
            None => {
                crate::warn!(
                    self,
                    "System `{}` was not found for plugin lookup",
                    system_id
                );
                Err(SceneError::SystemNotFound)
            }
        }
    }

    /// Adds a component to an entity by component id.
    ///
    /// The runtime resolves `wxr_create_<component>`, `wxr_destroy_<component>`,
    /// and schema bindings from loaded plugins.
    pub fn add_component(
        &mut self,
        entity_id: Uuid,
        component_id: String,
    ) -> Result<(), SceneError> {
        self.add_component_from_data(ComponentData {
            id: component_id,
            entity_id,
            fields: Vec::new(),
        })
    }

    fn add_component_from_data(&mut self, data: ComponentData) -> Result<(), SceneError> {
        let component_id = data.id.clone();
        let entity_id = data.entity_id;

        // Check if entity exists
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::debug!(
                self,
                "Entity `{}` was not found for component addition",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        if entity_components.contains_key(&component_id) {
            crate::debug!(
                self,
                "Component `{}` already exists on entity `{}`",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentAlreadyExists);
        }

        // Build the plugin
        let Some(component_symbols) = self
            .plugins_dynamic_first()
            .find_map(|plugin| Component::symbols(&component_id, plugin, self).ok())
        else {
            crate::warn!(self, "Component `{}` could not be created", component_id);
            return Err(SceneError::ComponentCreation);
        };

        let Some(mut component) =
            Component::create_with(component_id.clone(), component_symbols, self)
        else {
            crate::warn!(self, "Component creator for `{}` failed", component_id);
            return Err(SceneError::ComponentCreatorFailed);
        };

        component.deserialize_fields(data.fields);

        let entity_components = self
            .components
            .get_mut(&entity_id)
            .expect("entity was checked before creating the component");
        let component_id_for_log = component_id.clone();
        entity_components.insert(component_id, component);

        crate::info!(
            self,
            "Component `{}` added to entity `{}`",
            component_id_for_log,
            entity_id
        );
        Ok(())
    }

    /// Removes a component from an entity.
    pub fn remove_component(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<(), SceneError> {
        // Check if entity exists
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component removal",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        let Some(component) = entity_components.remove(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for removal",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentAlreadyExists);
        };

        drop(component);

        crate::info!(
            self,
            "Component `{}` removed from entity `{}`",
            component_id,
            entity_id
        );
        Ok(())
    }

    /// Returns sorted component ids attached to an entity.
    pub fn get_entity_components(&self, entity_id: Uuid) -> Result<Vec<String>, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component list lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let mut components: Vec<String> = entity_components.keys().cloned().collect();
        components.sort();
        Ok(components)
    }

    /// Returns the first entity that has `component_id`, if any.
    pub fn get_entity_with_component(&self, component_id: &str) -> Option<Uuid> {
        self.get_entities()
            .into_iter()
            .find(|entity_id| self.has_component(*entity_id, component_id))
    }

    /// Returns whether an entity has a component with this id.
    pub fn has_component(&self, entity_id: Uuid, component_id: &str) -> bool {
        self.components
            .get(&entity_id)
            .is_some_and(|components| components.contains_key(component_id))
    }

    /// Hot-reloads all dynamic plugins while preserving serializable scene state.
    ///
    /// The scene is serialized, dynamic plugins are unloaded and loaded again,
    /// and then the serialized state is deserialized. If called while a system
    /// is ticking, the reload is deferred until the current system runner
    /// returns.
    pub fn reload(&mut self) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls.push(DeferredCall::Reload);
            return Ok(());
        }

        // Serialize the important data
        let plugins: Vec<String> = self
            .plugins
            .keys()
            .filter_map(|id| {
                if id.is_empty() {
                    None
                } else {
                    Some(id.to_owned())
                }
            })
            .collect();
        let serialization_data = self.serialize().inspect_err(|err| match err {
            SceneError::Serialization(msg) => {
                crate::error!(self, "Scene serialization failed: {}", msg);
            }
            _ => {
                crate::error!(self, "Failed to serialize the scene: {:?}", err);
            }
        })?;

        // Remove the plugins
        for plugin in &plugins {
            if self.unload_plugin(plugin).is_err() {
                crate::error!(self, "Failed to unload the plugin: {}", plugin);
            }
        }

        // Reload the plugins
        for plugin in plugins {
            match self.load_plugin(plugin) {
                Err(SceneError::PluginLoading(PluginError::InvalidSymbol)) => {
                    crate::error!(
                        self,
                        "Symbol has a null byte during the plugin reloading! This is a bug! Please report this!"
                    );
                }
                Err(SceneError::PluginLoading(PluginError::LinkingError(err))) => {
                    crate::error!(
                        self,
                        "Plugin has a linking error while hotreloading: {}",
                        err
                    );
                }
                _ => {}
            }
        }

        // Deserialize the scene data (Scene reset also happens in here)
        self.deserialize(&serialization_data)
            .inspect_err(|err| match err {
                SceneError::Deserialization(msg) => {
                    crate::error!(self, "Scene deserialization failed: {}", msg);
                }
                _ => {
                    crate::error!(self, "Failed to deserialize the scene: {:?}", err);
                }
            })?;

        Ok(())
    }

    fn run_system(&mut self, system_id: &str, groups: usize, selector: Selector, runner: Runner) {
        self.set_logger(system_id.to_owned());
        let mut entities: Vec<Vec<*const u8>> = vec![Vec::new(); groups];

        for i in self.entities.keys() {
            let selection = unsafe { selector(self as *const Scene, i.as_bytes() as *const u8) };
            if selection >= 0
                && let Some(group) = entities.get_mut(selection as usize)
            {
                group.push(i.as_bytes() as *const u8);
            }
        }

        let sizes: Vec<usize> = entities.iter().map(|group| group.len()).collect();
        let sizes_ptr = sizes.as_ptr();

        let entities: Vec<*const *const u8> = entities.iter().map(|group| group.as_ptr()).collect();
        let entities_ptr = entities.as_ptr();

        unsafe {
            runner(self as *mut Scene, entities_ptr, sizes_ptr);
        }
        self.reset_logger();
    }

    fn run_deferred_calls(&mut self) {
        let deferred_calls = std::mem::take(&mut self.deferred_calls);

        for deferred_call in deferred_calls {
            match deferred_call {
                DeferredCall::Reload => {
                    if self.reload().is_err() {
                        crate::warn!(
                            self,
                            "Reload Failed! The scene has maybe been partially been reloaded."
                        );
                    }
                }
                DeferredCall::Load(data) => {
                    if self.deserialize(&data).is_err() {
                        crate::warn!(
                            self,
                            "Deferred scene load failed! The scene has maybe been partially loaded."
                        );
                    }
                }
                DeferredCall::UnloadPlugin(plugin_id) => {
                    if self.unload_plugin(&plugin_id).is_err() {
                        crate::warn!(self, "Deferred unload of plugin `{}` failed", plugin_id);
                    }
                }
                DeferredCall::RemoveSystem(system_id) => {
                    if self.remove_system(&system_id).is_err() {
                        crate::warn!(self, "Deferred removal of system `{}` failed", system_id);
                    }
                }
            }
        }
    }

    /// Runs all systems once and processes deferred scene changes.
    ///
    /// Systems run from highest priority to lowest priority. Returns `false`
    /// only when `should_exit` was requested during the tick.
    pub fn tick(&mut self) -> bool {
        let mut systems_sorted: Vec<&System> = self.systems.values().collect();
        systems_sorted.sort_by_key(|a| a.get_priority());

        let system_ids: Vec<String> = systems_sorted
            .iter()
            .rev()
            .map(|system| system.get_id().to_owned())
            .collect();

        for system_id in system_ids {
            let Some(system) = self.systems.get(&system_id) else {
                continue;
            };

            let groups = system.get_groups();
            let selector = system.get_selector();
            let runner = system.get_runner();

            self.is_ticking = true;
            self.run_system(&system_id, groups, selector, runner);
            self.is_ticking = false;

            self.run_deferred_calls();
        }

        let should_exit = self.should_exit;
        self.should_exit = false;
        !should_exit
    }

    fn get_component(&self, entity_id: Uuid, component_id: &str) -> Result<&Component, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            return Err(SceneError::ComponentNotFound);
        };

        Ok(component)
    }

    pub(crate) unsafe fn get_field_ptrs(
        &self,
        entity_id: Uuid,
        component_id: &str,
        fields: &[&str],
    ) -> Result<Vec<*mut c_void>, SceneError> {
        let component = self.get_component(entity_id, component_id)?;
        fields
            .iter()
            .map(|field| {
                unsafe { component.get_field_ptr(field) }.map_err(SceneError::ComponentFieldError)
            })
            .collect()
    }

    /// Borrows component fields as typed shared references.
    ///
    /// `fields` must have the same length and order as the requested tuple.
    ///
    /// SAFETY: this is a safe API over raw component storage. The requested
    /// Rust reference types must match the schema field types exposed by the
    /// component plugin. A wrong type can create invalid references.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let (health,) = scene.query::<(&i64,)>(entity, "Health", &["value"])?;
    /// println!("{health}");
    /// ```
    pub fn query<'scene, Q>(
        &'scene self,
        entity_id: Uuid,
        component_id: &str,
        fields: &[&str],
    ) -> Result<Q, SceneError>
    where
        Q: SceneQuery<'scene>,
    {
        if fields.len() != Q::FIELD_COUNT {
            return Err(SceneError::ComponentFieldError(
                crate::error::ComponentError::FieldParsing,
            ));
        }

        let field_ptrs = unsafe { self.get_field_ptrs(entity_id, component_id, fields)? };
        unsafe { Q::fetch(&field_ptrs) }
    }

    /// Borrows component fields as typed shared or mutable references.
    ///
    /// Mutable fields must be registered as mutable in the component schema,
    /// and duplicate field names are rejected before references are created.
    ///
    /// SAFETY: this is a safe API over raw component storage. The requested
    /// Rust reference types must match the schema field types exposed by the
    /// component plugin. A wrong type can create invalid references.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let (health,) = scene.query_mut::<(&mut i64,)>(entity, "Health", &["value"])?;
    /// *health += 1;
    /// ```
    pub fn query_mut<'scene, Q>(
        &'scene mut self,
        entity_id: Uuid,
        component_id: &str,
        fields: &[&str],
    ) -> Result<Q, SceneError>
    where
        Q: SceneQueryMut<'scene>,
    {
        if fields.len() != Q::FIELD_COUNT {
            return Err(SceneError::ComponentFieldError(
                crate::error::ComponentError::FieldParsing,
            ));
        }

        // Check if there are no duplicates in the fields
        // This is not a thorough check but at least a good error prevention
        for index in 0..fields.len() {
            if fields[index + 1..].contains(&fields[index]) {
                return Err(SceneError::ComponentFieldError(
                    crate::error::ComponentError::FieldParsing,
                ));
            }
        }

        let component = self.get_component(entity_id, component_id)?;
        Q::for_each_mutable_field(fields, |field| {
            if component
                .is_field_mutable(field)
                .map_err(SceneError::ComponentFieldError)?
            {
                Ok(())
            } else {
                Err(SceneError::ComponentFieldError(
                    crate::error::ComponentError::FieldNotMutable,
                ))
            }
        })?;

        let field_ptrs = unsafe { self.get_field_ptrs(entity_id, component_id, fields)? };
        unsafe { Q::fetch(&field_ptrs) }
    }

    fn clear_assets(&mut self) {
        let assets = std::mem::take(&mut self.assets);

        for (asset_type, assets) in assets {
            for asset in assets.into_values() {
                self.set_logger(asset_type.clone());
                asset.destroy(self);
                self.reset_logger();
            }
        }
    }

    fn remove_asset_type(&mut self, asset_type: &str) {
        if let Some(assets) = self.assets.remove(asset_type) {
            for asset in assets.into_values() {
                self.set_logger(asset_type.to_owned());
                asset.destroy(self);
                self.reset_logger();
            }
        }

        self.asset_types.remove(asset_type);
        crate::info!(self, "Asset type `{}` removed", asset_type);
    }

    fn ensure_asset_type(&mut self, asset_type: &str) -> Result<(), SceneError> {
        if self.asset_types.contains_key(asset_type) {
            return Ok(());
        }

        let asset_type_data = self
            .plugins_dynamic_first()
            .find_map(|plugin| AssetType::new(asset_type.to_owned(), plugin, self).ok());

        let Some(asset_type_data) = asset_type_data else {
            crate::warn!(self, "Asset type `{}` could not be created", asset_type);
            return Err(SceneError::AssetError(AssetError::AssetTypeNotFound));
        };

        self.asset_types
            .insert(asset_type.to_owned(), asset_type_data);
        Ok(())
    }

    /// Ensures an asset of `asset_type` with `data_string` is loaded.
    ///
    /// If the asset already exists, this is a no-op. Otherwise the asset type
    /// is registered on demand and its creator binding is called.
    pub fn ensure_asset_loaded(
        &mut self,
        asset_type: &str,
        data_string: &str,
    ) -> Result<(), SceneError> {
        if self
            .assets
            .get(asset_type)
            .is_some_and(|assets| assets.contains_key(data_string))
        {
            return Ok(());
        }

        self.ensure_asset_type(asset_type)?;

        let (creator, destroyer) = {
            let asset_type_data = self
                .asset_types
                .get(asset_type)
                .ok_or(SceneError::AssetError(AssetError::AssetTypeNotFound))?;
            (asset_type_data.creator(), asset_type_data.destroyer())
        };

        let asset = AssetType::create_asset_with(self, data_string, creator, destroyer)
            .map_err(SceneError::AssetError)?;

        self.assets
            .entry(asset_type.to_owned())
            .or_default()
            .insert(data_string.to_owned(), asset);

        crate::info!(
            self,
            "Asset `{}` with data `{}` created",
            asset_type,
            data_string
        );
        Ok(())
    }

    /// Returns sorted data strings for loaded assets of an asset type.
    pub fn get_loaded_asset_data_strings(&self, asset_type: &str) -> Vec<String> {
        let Some(assets) = self.assets.get(asset_type) else {
            return Vec::new();
        };

        let mut data_strings: Vec<String> = assets.keys().cloned().collect();
        data_strings.sort();
        data_strings
    }

    fn get_asset(&self, asset_type: &str, data_string: &str) -> Result<&Asset, SceneError> {
        let Some(assets) = self.assets.get(asset_type) else {
            crate::debug!(
                self,
                "Asset type `{}` was not found for asset lookup",
                asset_type
            );
            return Err(SceneError::AssetError(AssetError::AssetTypeNotFound));
        };

        let Some(asset) = assets.get(data_string) else {
            crate::debug!(
                self,
                "Asset `{}` with data `{}` was not found",
                asset_type,
                data_string
            );
            return Err(SceneError::AssetError(AssetError::InvalidAsset));
        };

        Ok(asset)
    }

    pub(crate) unsafe fn get_asset_field_ptrs(
        &self,
        asset_type: &str,
        data_string: &str,
        fields: &[&str],
    ) -> Result<Vec<*mut c_void>, SceneError> {
        let Some(asset_type_data) = self.asset_types.get(asset_type) else {
            crate::debug!(
                self,
                "Asset type `{}` was not found for field lookup",
                asset_type
            );
            return Err(SceneError::AssetError(AssetError::AssetTypeNotFound));
        };
        let asset = self.get_asset(asset_type, data_string)?;

        fields
            .iter()
            .map(|field| {
                unsafe { asset_type_data.get_field_ptr(asset, field) }
                    .map_err(SceneError::AssetError)
            })
            .collect()
    }

    /// Borrows fields from an already loaded asset as typed shared references.
    ///
    /// `fields` must have the same length and order as the requested tuple.
    ///
    /// SAFETY: this is a safe API over raw asset storage. The requested Rust
    /// reference types must match the schema field types exposed by the asset
    /// plugin. A wrong type can create invalid references.
    pub fn asset_query_loaded<'scene, Q>(
        &'scene self,
        asset_type: &str,
        data_string: &str,
        fields: &[&str],
    ) -> Result<Q, SceneError>
    where
        Q: SceneQuery<'scene>,
    {
        if fields.len() != Q::FIELD_COUNT {
            return Err(SceneError::AssetError(AssetError::FieldParsing));
        }

        let field_ptrs = unsafe { self.get_asset_field_ptrs(asset_type, data_string, fields)? };
        unsafe { Q::fetch(&field_ptrs) }
    }

    /// Loads an asset if needed, then borrows its fields as typed shared references.
    ///
    /// SAFETY: this has the same raw-storage type requirements as
    /// `asset_query_loaded`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let (content,) = scene.asset_query::<(&String,)>("TextAsset", "story.txt", &["content"])?;
    /// ```
    pub fn asset_query<'scene, Q>(
        &'scene mut self,
        asset_type: &str,
        data_string: &str,
        fields: &[&str],
    ) -> Result<Q, SceneError>
    where
        Q: SceneQuery<'scene>,
    {
        self.ensure_asset_loaded(asset_type, data_string)?;
        self.asset_query_loaded(asset_type, data_string, fields)
    }

    /// Returns sorted field ids registered for a component on an entity.
    pub fn get_component_fields(
        &self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<Vec<String>, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component field list lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for field list lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        let mut fields = component.get_fields();
        fields.sort();
        Ok(fields)
    }

    /// Returns the id of the plugin that provided a component on an entity.
    pub fn get_entity_component_plugin_id(
        &self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<&str, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component plugin lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for plugin lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        Ok(component.get_plugin_id())
    }

    /// Returns the runtime type hint for a component field.
    pub fn get_component_field_type(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<FieldType, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component field type lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for field type lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .get_field_type(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    /// Returns whether a component field can be borrowed mutably.
    pub fn is_component_field_mutable(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<bool, SceneError> {
        let component = self.get_component(entity_id, component_id)?;

        component
            .is_field_mutable(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    /// Returns whether a component field can be edited from a string.
    pub fn is_component_field_string_parsable(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<bool, SceneError> {
        let component = self.get_component(entity_id, component_id)?;

        component
            .is_field_string_parsable(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    /// Renders a component field as a string for UI or debugging.
    ///
    /// SAFETY: this calls the component's raw getter internally. The component
    /// schema must report the real field type.
    pub fn render_field(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<String, SceneError> {
        let component = self.get_component(entity_id, component_id)?;

        component
            .render_field(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    /// Parses a string and writes it into a component field.
    ///
    /// The field must have a getter, must not be `Blob`, and the input must
    /// parse as the field's runtime type.
    ///
    /// SAFETY: this calls the component's raw getter internally. The component
    /// schema must report the real field type.
    pub fn parse_field(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
        input: &str,
    ) -> Result<(), SceneError> {
        let component = self.get_component(entity_id, component_id)?;

        component
            .parse_field(field_id, input)
            .map_err(SceneError::ComponentFieldError)
    }

    /// Requests that the next `tick` return `false`.
    pub fn should_exit(&mut self) {
        self.should_exit = true;
    }
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]

    use super::*;
    use crate::{
        error::ComponentError,
        scene::{
            component::{FieldType, Getter, Schema, SerializedBytes},
            serialization::{ComponentData, FieldData, SceneData},
        },
    };
    use std::sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    #[repr(C)]
    struct SceneCounter {
        value: i64,
    }

    #[repr(C)]
    struct ScenePair {
        a: i64,
        b: i64,
    }

    #[repr(C)]
    struct SceneOctet {
        values: [i64; 8],
    }

    #[derive(Default, Debug, PartialEq, Eq)]
    struct SceneOwnedValue {
        value: String,
    }

    #[repr(C)]
    struct SceneOwner {
        value: SceneOwnedValue,
    }

    struct SceneOrderComponent;

    struct SceneOrderAsset;

    unsafe extern "C" fn scene_counter_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut SceneCounter)).value as *mut i64 as *mut c_void }
    }

    unsafe extern "C" fn scene_pair_a_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut ScenePair)).a as *mut i64 as *mut c_void }
    }

    unsafe extern "C" fn scene_pair_b_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut ScenePair)).b as *mut i64 as *mut c_void }
    }

    macro_rules! scene_octet_getter {
        ($name:ident, $index:expr) => {
            unsafe extern "C" fn $name(data: *mut c_void) -> *mut c_void {
                unsafe {
                    &mut (*(data as *mut SceneOctet)).values[$index] as *mut i64 as *mut c_void
                }
            }
        };
    }

    scene_octet_getter!(scene_octet_a_getter, 0);
    scene_octet_getter!(scene_octet_b_getter, 1);
    scene_octet_getter!(scene_octet_c_getter, 2);
    scene_octet_getter!(scene_octet_d_getter, 3);
    scene_octet_getter!(scene_octet_e_getter, 4);
    scene_octet_getter!(scene_octet_f_getter, 5);
    scene_octet_getter!(scene_octet_g_getter, 6);
    scene_octet_getter!(scene_octet_h_getter, 7);

    unsafe extern "C" fn scene_counter_serializer(data: *const c_void) -> SerializedBytes {
        unsafe {
            SerializedBytes::from_vec(
                (*(data as *const SceneCounter))
                    .value
                    .to_le_bytes()
                    .to_vec(),
            )
        }
    }

    unsafe extern "C" fn scene_counter_deserializer(data: *mut c_void, value: SerializedBytes) {
        unsafe {
            let bytes = value.into_vec();
            if let Ok(bytes) = <[u8; 8]>::try_from(bytes.as_slice()) {
                (*(data as *mut SceneCounter)).value = i64::from_le_bytes(bytes);
            }
        }
    }

    unsafe extern "C" fn scene_owner_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut SceneOwner)).value as *mut SceneOwnedValue as *mut c_void }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_counter(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(SceneCounter { value: 1 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_pair(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(ScenePair { a: 1, b: 2 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_octet(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(SceneOctet {
            values: [1, 2, 3, 4, 5, 6, 7, 8],
        })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut SceneCounter));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_pair(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut ScenePair));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_octet(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut SceneOctet));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_owner(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(SceneOwner {
            value: SceneOwnedValue::default(),
        })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_owner(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut SceneOwner));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_order_component(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(SceneOrderComponent)) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_order_component(data: *mut c_void) {
        SCENE_UNLOAD_ORDER.lock().unwrap().push("component");
        unsafe {
            drop(Box::from_raw(data as *mut SceneOrderComponent));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_scene_counter(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::I64,
                Some(scene_counter_getter),
                true,
                Some(scene_counter_serializer),
                Some(scene_counter_deserializer),
            );
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_scene_pair(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "a".to_owned(),
                FieldType::I64,
                Some(scene_pair_a_getter),
                true,
                None,
                None,
            );
            (*schema).add_field(
                "b".to_owned(),
                FieldType::I64,
                Some(scene_pair_b_getter),
                false,
                None,
                None,
            );
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_scene_octet(schema: *mut Schema) {
        let fields: [(&str, Getter); 8] = [
            ("a", scene_octet_a_getter),
            ("b", scene_octet_b_getter),
            ("c", scene_octet_c_getter),
            ("d", scene_octet_d_getter),
            ("e", scene_octet_e_getter),
            ("f", scene_octet_f_getter),
            ("g", scene_octet_g_getter),
            ("h", scene_octet_h_getter),
        ];

        unsafe {
            for (id, getter) in fields {
                (*schema).add_field(
                    id.to_owned(),
                    FieldType::I64,
                    Some(getter),
                    false,
                    None,
                    None,
                );
            }
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_scene_owner(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::Blob,
                Some(scene_owner_getter),
                false,
                None,
                None,
            );
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_create_scene_order_asset(
        _scene: *mut Scene,
        _data_string: *const std::ffi::c_char,
    ) -> *mut c_void {
        Box::into_raw(Box::new(SceneOrderAsset)) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_schema_scene_order_asset(
        _schema: *mut crate::scene::assets::Schema,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_asset_destroy_scene_order_asset(
        _scene: *mut Scene,
        data: *mut c_void,
    ) {
        SCENE_UNLOAD_ORDER.lock().unwrap().push("asset");
        unsafe {
            drop(Box::from_raw(data as *mut SceneOrderAsset));
        }
    }

    static SCENE_ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_RELOAD_ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_RELOAD_DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_DEFERRED_RELOAD_ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_DEFERRED_RELOAD_DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_DEFERRED_REMOVE_TARGET_PRESENT: AtomicBool = AtomicBool::new(false);
    static SCENE_DEFERRED_UNLOAD_PLUGIN_PRESENT: AtomicBool = AtomicBool::new(false);
    static SCENE_DEFERRED_LOAD_PATH: LazyLock<Mutex<Option<std::path::PathBuf>>> =
        LazyLock::new(|| Mutex::new(None));
    static SCENE_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static SCENE_RELOAD_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static SCENE_DEFERRED_REMOVE_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static SCENE_UNLOAD_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

    fn set_scene_counter(scene: &mut Scene, entity: Uuid, value: i64) {
        let (field,) = scene
            .query_mut::<(&mut i64,)>(entity, "scene_counter", &["value"])
            .unwrap();
        *field = value;
    }

    fn get_scene_counter(scene: &Scene, entity: Uuid) -> i64 {
        let (field,) = scene
            .query::<(&i64,)>(entity, "scene_counter", &["value"])
            .unwrap();
        *field
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_attach_system(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_attach_scene_attach_system(_scene: *mut Scene) {
        SCENE_ATTACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_detach_scene_attach_system(_scene: *mut Scene) {
        SCENE_DETACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_cleanup_system(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_reload_counted_system(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_attach_scene_reload_counted_system(_scene: *mut Scene) {
        SCENE_RELOAD_ATTACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_detach_scene_reload_counted_system(_scene: *mut Scene) {
        SCENE_RELOAD_DETACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_deferred_reload_counted_system(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_attach_scene_deferred_reload_counted_system(_scene: *mut Scene) {
        SCENE_DEFERRED_RELOAD_ATTACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_detach_scene_deferred_reload_counted_system(_scene: *mut Scene) {
        SCENE_DEFERRED_RELOAD_DETACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_deferred_reload_low_priority(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        SCENE_RELOAD_TICK_ORDER.lock().unwrap().push("low");
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_reload_request(
        scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        unsafe {
            (&mut *scene).reload().unwrap();
        }
        SCENE_RELOAD_TICK_ORDER
            .lock()
            .unwrap()
            .push("reload-request");
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_load_request(
        scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        let path = SCENE_DEFERRED_LOAD_PATH.lock().unwrap().clone().unwrap();
        unsafe {
            (&mut *scene).load(path).unwrap();
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_loaded_marker(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_deferred_remove_request(
        scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        let scene = unsafe { &mut *scene };
        scene.remove_system("scene_deferred_remove_target").unwrap();
        SCENE_DEFERRED_REMOVE_TARGET_PRESENT.store(
            scene
                .get_systems()
                .contains(&"scene_deferred_remove_target".to_owned()),
            Ordering::SeqCst,
        );
        SCENE_DEFERRED_REMOVE_TICK_ORDER
            .lock()
            .unwrap()
            .push("remove-request");
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_deferred_remove_target(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        SCENE_DEFERRED_REMOVE_TICK_ORDER
            .lock()
            .unwrap()
            .push("remove-target");
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_deferred_unload_request(
        scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        let scene = unsafe { &mut *scene };
        scene.unload_plugin("dynamic_deferred").unwrap();
        SCENE_DEFERRED_UNLOAD_PLUGIN_PRESENT.store(
            scene.get_plugins().contains(&"dynamic_deferred".to_owned()),
            Ordering::SeqCst,
        );
    }

    #[unsafe(no_mangle)]
    static WXR_GROUPS_SCENE_LOW_PRIORITY: usize = 1;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_low_priority(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        SCENE_TICK_ORDER.lock().unwrap().push("low");
    }

    #[unsafe(no_mangle)]
    static WXR_GROUPS_SCENE_HIGH_PRIORITY: usize = 1;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_scene_high_priority(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        SCENE_TICK_ORDER.lock().unwrap().push("high");
    }

    #[test]
    fn scene_add_entity() {
        let mut scene = Scene::new();

        let entity = scene.add_entity();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "");
    }

    #[test]
    fn scene_set_entity_name_for_existing_entity() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        scene.set_entity_name(entity, "Player".to_owned()).unwrap();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "Player");
    }

    #[test]
    fn scene_get_entities() {
        let mut scene = Scene::new();
        let first_entity = scene.add_entity();
        let second_entity = scene.add_entity();

        let mut entities = vec![first_entity, second_entity];
        entities.sort();

        assert_eq!(scene.get_entities(), entities);
    }

    #[test]
    fn scene_remove_entity_for_existing_entity() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        scene.remove_entity(entity).unwrap();

        assert_eq!(
            scene.get_entity_name(entity),
            Err(SceneError::EntityNotFound)
        );
    }

    #[test]
    fn scene_add_component_for_existing_entity() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        assert_eq!(
            scene.add_component(entity, "scene_counter".to_owned()),
            Ok(())
        );
    }

    #[test]
    fn scene_get_entity_components() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.get_entity_components(entity).unwrap(),
            vec!["scene_counter"]
        );
    }

    #[test]
    fn scene_get_entity_with_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.get_entity_with_component("scene_counter"),
            Some(entity)
        );
        assert_eq!(scene.get_entity_with_component("missing"), None);
    }

    #[test]
    fn scene_query_gets_shared_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        let (value,) = scene
            .query::<(&i64,)>(entity, "scene_counter", &["value"])
            .unwrap();

        assert_eq!(*value, 1);
    }

    #[test]
    fn scene_query_gets_mutable_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        {
            let (value,) = scene
                .query_mut::<(&mut i64,)>(entity, "scene_counter", &["value"])
                .unwrap();
            *value = 17;
        }

        let (value,) = scene
            .query::<(&i64,)>(entity, "scene_counter", &["value"])
            .unwrap();
        assert_eq!(*value, 17);
    }

    #[test]
    fn scene_query_gets_mutable_and_shared_fields() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        {
            let (a, b) = scene
                .query_mut::<(&mut i64, &i64)>(entity, "scene_pair", &["a", "b"])
                .unwrap();
            assert_eq!(*b, 2);
            *a = 5;
        }

        let (a, b) = scene
            .query::<(&i64, &i64)>(entity, "scene_pair", &["a", "b"])
            .unwrap();
        assert_eq!((*a, *b), (5, 2));
    }

    #[test]
    fn scene_query_allows_multiple_shared_queries() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        let (a, b) = scene
            .query::<(&i64, &i64)>(entity, "scene_pair", &["a", "b"])
            .unwrap();
        let (c, d) = scene
            .query::<(&i64, &i64)>(entity, "scene_pair", &["a", "b"])
            .unwrap();

        assert_eq!((*a, *b, *c, *d), (1, 2, 1, 2));
    }

    #[test]
    fn scene_query_gets_eight_shared_fields() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_octet".to_owned())
            .unwrap();

        let (a, b, c, d, e, f, g, h) = scene
            .query::<(&i64, &i64, &i64, &i64, &i64, &i64, &i64, &i64)>(
                entity,
                "scene_octet",
                &["a", "b", "c", "d", "e", "f", "g", "h"],
            )
            .unwrap();

        assert_eq!((*a, *b, *c, *d, *e, *f, *g, *h), (1, 2, 3, 4, 5, 6, 7, 8));
    }

    #[test]
    fn scene_query_rejects_duplicate_fields() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        assert_eq!(
            scene.query_mut::<(&mut i64, &i64)>(entity, "scene_pair", &["a", "a"]),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldParsing
            ))
        );
    }

    #[test]
    fn scene_query_rejects_wrong_field_count() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        assert_eq!(
            scene.query::<(&i64, &i64)>(entity, "scene_pair", &["a"]),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldParsing
            ))
        );
    }

    #[test]
    fn scene_query_mut_rejects_wrong_field_count() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        assert_eq!(
            scene.query_mut::<(&i64, &i64)>(entity, "scene_pair", &["a"]),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldParsing
            ))
        );
    }

    #[test]
    fn scene_query_mut_rejects_non_mutable_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        assert_eq!(
            scene.query_mut::<(&mut SceneOwnedValue,)>(entity, "scene_owner", &["value"]),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotMutable
            ))
        );
    }

    #[test]
    fn scene_query_mut_allows_shared_non_mutable_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        let (value,) = scene
            .query_mut::<(&SceneOwnedValue,)>(entity, "scene_owner", &["value"])
            .unwrap();

        assert_eq!(value, &SceneOwnedValue::default());
    }

    #[test]
    fn scene_add_component_for_duplicate_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.add_component(entity, "scene_counter".to_owned()),
            Err(SceneError::ComponentAlreadyExists)
        );
    }

    #[test]
    fn scene_remove_component_for_existing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        scene.remove_component(entity, "scene_counter").unwrap();

        assert_eq!(
            scene.query::<(&i64,)>(entity, "scene_counter", &["value"]),
            Err(SceneError::ComponentNotFound)
        );
    }

    #[test]
    fn scene_get_component_fields() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.get_component_fields(entity, "scene_counter").unwrap(),
            vec!["value"]
        );
    }

    #[test]
    fn scene_get_entity_component_plugin_id() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.get_entity_component_plugin_id(entity, "scene_counter"),
            Ok("")
        );
    }

    #[test]
    fn scene_get_entity_component_plugin_id_rejects_missing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        assert_eq!(
            scene.get_entity_component_plugin_id(entity, "missing"),
            Err(SceneError::ComponentNotFound)
        );
    }

    #[test]
    fn scene_get_component_field_type() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene
                .get_component_field_type(entity, "scene_counter", "value")
                .unwrap(),
            FieldType::I64
        );
    }

    #[test]
    fn scene_is_component_field_mutable() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        assert_eq!(
            scene.is_component_field_mutable(entity, "scene_pair", "a"),
            Ok(true)
        );
        assert_eq!(
            scene.is_component_field_mutable(entity, "scene_pair", "b"),
            Ok(false)
        );
    }

    #[test]
    fn scene_is_component_field_string_parsable() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        assert_eq!(
            scene.is_component_field_string_parsable(entity, "scene_pair", "a"),
            Ok(true)
        );
        assert_eq!(
            scene.is_component_field_string_parsable(entity, "scene_owner", "value"),
            Ok(false)
        );
    }

    #[test]
    fn scene_component_field_predicates_reject_missing_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.is_component_field_mutable(entity, "scene_counter", "missing"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotFound
            ))
        );
        assert_eq!(
            scene.is_component_field_string_parsable(entity, "scene_counter", "missing"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotFound
            ))
        );
    }

    #[test]
    fn scene_render_field_returns_rendered_value() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene
                .render_field(entity, "scene_counter", "value")
                .unwrap(),
            "1"
        );
    }

    #[test]
    fn scene_parse_field_updates_mutable_value() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        scene
            .parse_field(entity, "scene_counter", "value", "42")
            .unwrap();

        assert_eq!(get_scene_counter(&scene, entity), 42);
    }

    #[test]
    fn scene_parse_field_rejects_invalid_value() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.parse_field(entity, "scene_counter", "value", "invalid"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldValueParsing
            ))
        );
        assert_eq!(get_scene_counter(&scene, entity), 1);
    }

    #[test]
    fn scene_parse_field_rejects_immutable_field() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_pair".to_owned())
            .unwrap();

        assert_eq!(
            scene.parse_field(entity, "scene_pair", "b", "42"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotMutable
            ))
        );
    }

    #[test]
    fn scene_render_field_renders_blob_pointer() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        assert!(
            scene
                .render_field(entity, "scene_owner", "value")
                .unwrap()
                .starts_with("0x")
        );
    }

    #[test]
    fn scene_add_system_for_existing_symbol() {
        SCENE_ATTACH_COUNT.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();

        scene
            .add_system("scene_attach_system".to_owned(), 1)
            .unwrap();

        assert_eq!(SCENE_ATTACH_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scene_get_systems() {
        let mut scene = Scene::new();
        scene
            .add_system("scene_cleanup_system".to_owned(), 1)
            .unwrap();

        assert_eq!(scene.get_systems(), vec!["scene_cleanup_system"]);
    }

    #[test]
    fn scene_get_system_priority() {
        let mut scene = Scene::new();
        scene
            .add_system("scene_cleanup_system".to_owned(), 7)
            .unwrap();

        assert_eq!(scene.get_system_priority("scene_cleanup_system"), Ok(7));
    }

    #[test]
    fn scene_get_system_plugin_id() {
        let mut scene = Scene::new();
        scene
            .add_system("scene_cleanup_system".to_owned(), 7)
            .unwrap();

        assert_eq!(scene.get_system_plugin_id("scene_cleanup_system"), Ok(""));
    }

    #[test]
    fn scene_get_system_plugin_id_rejects_missing_system() {
        let scene = Scene::new();

        assert_eq!(
            scene.get_system_plugin_id("missing"),
            Err(SceneError::SystemNotFound)
        );
    }

    #[test]
    fn scene_get_plugins() {
        let mut scene = Scene::new();
        scene.plugins.insert(
            "dynamic_a".to_owned(),
            Plugin::new_test_dynamic("dynamic_a".to_owned()),
        );

        assert_eq!(scene.get_plugins(), vec!["dynamic_a"]);
    }

    #[test]
    fn scene_plugin_lookup_prioritizes_dynamic_plugins_before_static() {
        let mut scene = Scene::new();
        scene.plugins.insert(
            "dynamic_a".to_owned(),
            Plugin::new_test_dynamic("dynamic_a".to_owned()),
        );
        scene.plugins.insert(
            "dynamic_b".to_owned(),
            Plugin::new_test_dynamic("dynamic_b".to_owned()),
        );

        let plugin_ids: Vec<&str> = scene
            .plugins_dynamic_first()
            .map(|plugin| plugin.get_id())
            .collect();

        assert_eq!(plugin_ids.len(), 3);
        assert!(plugin_ids[..2].contains(&"dynamic_a"));
        assert!(plugin_ids[..2].contains(&"dynamic_b"));
        assert_eq!(plugin_ids[2], "");
    }

    #[test]
    fn scene_plugin_lookup_uses_static_when_no_dynamic_plugins_exist() {
        let scene = Scene::new();

        let plugin_ids: Vec<&str> = scene
            .plugins_dynamic_first()
            .map(|plugin| plugin.get_id())
            .collect();

        assert_eq!(plugin_ids, vec![""]);
    }

    #[test]
    fn scene_remove_system_for_existing_system() {
        SCENE_DETACH_COUNT.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();
        scene
            .add_system("scene_attach_system".to_owned(), 1)
            .unwrap();

        scene.remove_system("scene_attach_system").unwrap();

        assert_eq!(SCENE_DETACH_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scene_reset_with_runtime_state() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        scene
            .add_system("scene_cleanup_system".to_owned(), 1)
            .unwrap();

        scene.reset().unwrap();

        assert_eq!(
            scene.get_entity_name(entity),
            Err(SceneError::EntityNotFound)
        );
        assert_eq!(
            scene.remove_system("scene_cleanup_system"),
            Err(SceneError::SystemNotFound)
        );
    }

    #[test]
    fn scene_unload_plugin_rejects_static_plugin() {
        let mut scene = Scene::new();

        assert_eq!(scene.unload_plugin(""), Err(SceneError::StaticPluginUnload));
    }

    #[test]
    fn scene_unload_plugin_for_static_plugin() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        scene
            .add_system("scene_cleanup_system".to_owned(), 1)
            .unwrap();

        assert_eq!(scene.unload_plugin(""), Err(SceneError::StaticPluginUnload));

        assert_eq!(get_scene_counter(&scene, entity), 1);
        scene.remove_system("scene_cleanup_system").unwrap();
    }

    #[test]
    fn scene_unload_plugin_destroys_assets_before_components() {
        SCENE_UNLOAD_ORDER.lock().unwrap().clear();
        let mut scene = Scene::new();
        scene.plugins.insert(
            "dynamic_order".to_owned(),
            Plugin::new_test_dynamic("dynamic_order".to_owned()),
        );
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_order_component".to_owned())
            .unwrap();
        scene
            .ensure_asset_loaded("scene_order_asset", "asset")
            .unwrap();

        scene.unload_plugin("dynamic_order").unwrap();

        assert_eq!(
            *SCENE_UNLOAD_ORDER.lock().unwrap(),
            vec!["asset", "component"]
        );
    }

    #[test]
    fn scene_tick_with_multiple_systems() {
        SCENE_TICK_ORDER.lock().unwrap().clear();
        let mut scene = Scene::new();
        scene.add_entity();
        scene
            .add_system("scene_high_priority".to_owned(), 2)
            .unwrap();
        scene
            .add_system("scene_low_priority".to_owned(), 1)
            .unwrap();

        scene.tick();

        assert_eq!(*SCENE_TICK_ORDER.lock().unwrap(), vec!["high", "low"]);
    }

    #[test]
    fn scene_tick_defers_remove_system_until_current_runner_finishes() {
        SCENE_DEFERRED_REMOVE_TARGET_PRESENT.store(false, Ordering::SeqCst);
        SCENE_DEFERRED_REMOVE_TICK_ORDER.lock().unwrap().clear();
        let mut scene = Scene::new();
        scene.add_entity();
        scene
            .add_system("scene_deferred_remove_target".to_owned(), 1)
            .unwrap();
        scene
            .add_system("scene_deferred_remove_request".to_owned(), 2)
            .unwrap();

        assert!(scene.tick());

        assert!(SCENE_DEFERRED_REMOVE_TARGET_PRESENT.load(Ordering::SeqCst));
        assert_eq!(
            *SCENE_DEFERRED_REMOVE_TICK_ORDER.lock().unwrap(),
            vec!["remove-request"]
        );
        assert_eq!(
            scene.remove_system("scene_deferred_remove_target"),
            Err(SceneError::SystemNotFound)
        );
        assert_eq!(scene.remove_system("scene_deferred_remove_request"), Ok(()));
    }

    #[test]
    fn scene_tick_defers_unload_plugin_until_current_runner_finishes() {
        SCENE_DEFERRED_UNLOAD_PLUGIN_PRESENT.store(false, Ordering::SeqCst);
        let mut scene = Scene::new();
        scene.add_entity();
        scene.plugins.insert(
            "dynamic_deferred".to_owned(),
            Plugin::new_test_dynamic("dynamic_deferred".to_owned()),
        );
        scene
            .add_system("scene_deferred_unload_request".to_owned(), 1)
            .unwrap();

        assert!(scene.tick());

        assert!(SCENE_DEFERRED_UNLOAD_PLUGIN_PRESENT.load(Ordering::SeqCst));
        assert!(!scene.get_plugins().contains(&"dynamic_deferred".to_owned()));
    }

    #[test]
    fn scene_reload_reinstantiates_scene_objects() {
        SCENE_RELOAD_ATTACH_COUNT.store(0, Ordering::SeqCst);
        SCENE_RELOAD_DETACH_COUNT.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .set_entity_name(entity, "Reloaded".to_owned())
            .unwrap();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        set_scene_counter(&mut scene, entity, 42);
        scene
            .add_system("scene_reload_counted_system".to_owned(), 1)
            .unwrap();

        scene.reload().unwrap();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "Reloaded");
        assert_eq!(get_scene_counter(&scene, entity), 42);
        assert_eq!(SCENE_RELOAD_ATTACH_COUNT.load(Ordering::SeqCst), 2);
        assert_eq!(SCENE_RELOAD_DETACH_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(scene.remove_system("scene_reload_counted_system"), Ok(()));
    }

    #[test]
    fn scene_tick_reloads_when_requested_by_system() {
        SCENE_DEFERRED_RELOAD_ATTACH_COUNT.store(0, Ordering::SeqCst);
        SCENE_DEFERRED_RELOAD_DETACH_COUNT.store(0, Ordering::SeqCst);
        SCENE_RELOAD_TICK_ORDER.lock().unwrap().clear();
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .set_entity_name(entity, "Deferred".to_owned())
            .unwrap();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        set_scene_counter(&mut scene, entity, 7);
        scene
            .add_system("scene_deferred_reload_counted_system".to_owned(), 1)
            .unwrap();
        scene
            .add_system("scene_deferred_reload_low_priority".to_owned(), 1)
            .unwrap();
        scene
            .add_system("scene_reload_request".to_owned(), 2)
            .unwrap();

        assert!(scene.tick());

        let tick_order = SCENE_RELOAD_TICK_ORDER.lock().unwrap();
        assert_eq!(tick_order[0], "reload-request");
        assert!(tick_order.contains(&"low"));
        drop(tick_order);
        assert_eq!(scene.get_entity_name(entity).unwrap(), "Deferred");
        assert_eq!(get_scene_counter(&scene, entity), 7);
        assert_eq!(SCENE_DEFERRED_RELOAD_ATTACH_COUNT.load(Ordering::SeqCst), 2);
        assert_eq!(SCENE_DEFERRED_RELOAD_DETACH_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(
            scene.remove_system("scene_deferred_reload_counted_system"),
            Ok(())
        );
        assert_eq!(
            scene.remove_system("scene_deferred_reload_low_priority"),
            Ok(())
        );
        assert_eq!(scene.remove_system("scene_reload_request"), Ok(()));
    }

    #[test]
    fn scene_tick_loads_when_requested_by_system() {
        let mut saved = Scene::new();
        let entity = saved.add_entity();
        saved
            .set_entity_name(entity, "Imported".to_owned())
            .unwrap();
        saved
            .add_system("scene_loaded_marker".to_owned(), 1)
            .unwrap();

        let path = std::env::temp_dir().join(format!("wasserxr-load-{}.scene", Uuid::now_v7()));
        saved.save(&path).unwrap();
        *SCENE_DEFERRED_LOAD_PATH.lock().unwrap() = Some(path.clone());

        let mut scene = Scene::new();
        scene
            .add_system("scene_load_request".to_owned(), 1)
            .unwrap();

        assert!(scene.tick());
        let _ = std::fs::remove_file(&path);
        *SCENE_DEFERRED_LOAD_PATH.lock().unwrap() = None;

        assert_eq!(scene.get_entity_name(entity).unwrap(), "Imported");
        assert!(
            scene
                .get_systems()
                .contains(&"scene_loaded_marker".to_owned())
        );
        assert!(
            !scene
                .get_systems()
                .contains(&"scene_load_request".to_owned())
        );
    }

    #[test]
    fn scene_deserialize_round_trip() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene.set_entity_name(entity, "Player".to_owned()).unwrap();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        set_scene_counter(&mut scene, entity, 42);
        scene
            .add_system("scene_cleanup_system".to_owned(), 3)
            .unwrap();

        let serialized = scene.serialize().unwrap();
        let mut loaded = Scene::new();
        loaded.deserialize(&serialized).unwrap();

        assert_eq!(loaded.get_entity_name(entity).unwrap(), "Player");
        assert_eq!(get_scene_counter(&loaded, entity), 42);
        assert_eq!(loaded.remove_system("scene_cleanup_system"), Ok(()));
    }

    #[test]
    fn scene_deserialize_missing_and_extra_component_fields() {
        let entity = Entity::new();
        let entity_data = entity.serialize();
        let data = SceneData {
            entities: vec![entity_data.clone()],
            systems: Vec::new(),
            components: vec![ComponentData {
                id: "scene_counter".to_owned(),
                entity_id: entity_data.id,
                fields: vec![FieldData {
                    name: "extra".to_owned(),
                    value: vec![1, 2, 3],
                }],
            }],
        };

        let mut scene = Scene::new();
        scene.deserialize(&data.encode().unwrap()).unwrap();

        assert_eq!(get_scene_counter(&scene, entity_data.id), 1);
    }

    #[test]
    fn scene_deserialize_skips_missing_systems_and_components() {
        let entity = Entity::new();
        let entity_data = entity.serialize();
        let data = SceneData {
            entities: vec![entity_data.clone()],
            systems: vec![SystemData {
                id: "missing_system".to_owned(),
                priority: 1,
            }],
            components: vec![ComponentData {
                id: "missing_component".to_owned(),
                entity_id: entity_data.id,
                fields: Vec::new(),
            }],
        };

        let mut scene = Scene::new();
        scene.deserialize(&data.encode().unwrap()).unwrap();

        assert_eq!(scene.get_entity_name(entity_data.id).unwrap(), "");
        assert!(!scene.has_component(entity_data.id, "missing_component"));
    }

    #[test]
    fn scene_save_and_load_round_trip() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene.set_entity_name(entity, "Saved".to_owned()).unwrap();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        set_scene_counter(&mut scene, entity, 11);

        let path = std::env::temp_dir().join(format!("wasserxr-{}.scene", Uuid::now_v7()));
        scene.save(&path).unwrap();

        let mut loaded = Scene::new();
        loaded.load(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded.get_entity_name(entity).unwrap(), "Saved");
        assert_eq!(get_scene_counter(&loaded, entity), 11);
    }

    #[test]
    fn scene_load_missing_file() {
        let path = std::env::temp_dir().join(format!("wasserxr-missing-{}.scene", Uuid::now_v7()));
        let mut scene = Scene::new();

        assert!(matches!(scene.load(path), Err(SceneError::FileIo(_))));
    }
}
