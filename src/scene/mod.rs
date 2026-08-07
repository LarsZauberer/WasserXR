//! Scene runtime, ECS storage, plugin loading, and typed query APIs.

macro_rules! ecs_invariant_message {
    ($detail:literal) => {
        concat!(
            "internal ECS invariant violated; this is a WasserXR bug: ",
            $detail
        )
    };
}

/// Asset type schemas and asset field query support.
pub mod assets;
/// Component schemas, field metadata, and serialization byte buffers.
pub mod component;
/// Entity storage and errors.
pub mod entity;
mod error;
/// Scene logging types, callbacks, and exported log macros.
pub mod logging;
/// Dynamic and static plugin loading errors.
pub mod plugin;
pub mod query;
/// Type-erased scene resources.
pub mod resource;
/// Serialized scene data and decoding errors.
pub mod serialization;
/// System lifecycle errors.
pub mod system;

pub use error::SceneError;

use assets::{Asset, AssetType};
use component::{Component, ComponentDefinition, FieldType};
use entity::Entity;
use plugin::Plugin;
use query::{SceneQuery, SceneQueryMut};
use resource::Resource;
use system::{System, SystemDefinition};

use crate::bindings::scene::WXREntity;
use crate::scene::logging::LogManager;
use crate::scene::serialization::{ComponentData, SceneData, SystemData};
use crate::scene::system::{Runner, SelectionGroup};
use assets::AssetError;
use component::ComponentError;
use entity::EntityError;
use plugin::PluginError;
use system::SystemError;

use std::ffi::c_void;
use std::rc::Rc;
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
        Self {
            entities: HashMap::new(),
            plugins: HashMap::new(),
            systems: HashMap::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
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
    /// Creates an empty scene with no plugins registered.
    pub fn new() -> Self {
        Self::default()
    }

    fn component_definition(&self, id: &str) -> Option<Rc<ComponentDefinition>> {
        self.plugins
            .values()
            .find_map(|plugin| plugin.component_definition(id))
    }

    fn asset_definition(&self, id: &str) -> Option<Rc<AssetType>> {
        self.plugins
            .values()
            .find_map(|plugin| plugin.asset_definition(id))
    }

    fn system_definition(&self, id: &str) -> Option<Rc<SystemDefinition>> {
        self.plugins
            .values()
            .find_map(|plugin| plugin.system_definition(id))
    }

    fn debug_assert_consistent(&self) {
        // Each entity owns exactly one component store, with no orphan stores.
        debug_assert!(
            self.entities
                .keys()
                .all(|entity| self.components.contains_key(entity))
                && self
                    .components
                    .keys()
                    .all(|entity| self.entities.contains_key(entity)),
            ecs_invariant_message!("entities and component storage must have identical keys"),
        );
        // Every storage key is the canonical ID of the object stored under it.
        debug_assert!(
            self.entities
                .iter()
                .all(|(id, entity)| id == &entity.get_id())
                && self
                    .systems
                    .iter()
                    .all(|(id, system)| id == system.get_id())
                && self.components.values().all(|components| components
                    .iter()
                    .all(|(id, component)| id == component.get_id()))
                && self
                    .plugins
                    .iter()
                    .all(|(id, plugin)| id == plugin.get_id()),
            ecs_invariant_message!("storage keys must match their stored object IDs"),
        );
        debug_assert!(
            self.plugins.values().all(Plugin::is_consistent),
            ecs_invariant_message!("every manifest definition must reference its owning plugin"),
        );
        // Objects holding plugin function pointers never outlive their provider.
        debug_assert!(
            self.systems
                .values()
                .all(|system| self.plugins.contains_key(system.get_plugin_id()))
                && self.components.values().all(|components| components
                    .values()
                    .all(|component| self.plugins.contains_key(component.get_plugin_id()))),
            ecs_invariant_message!("every plugin-owned object must reference a loaded plugin"),
        );
        // Every loaded asset is owned by a currently registered asset type.
        debug_assert!(
            self.assets
                .keys()
                .all(|asset_type| self.asset_definition(asset_type).is_some()),
            ecs_invariant_message!("every loaded asset must have a registered asset type"),
        );
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
        self.debug_assert_consistent();
        Ok(())
    }

    /// Serializes entities, systems, and components into WasserXR scene bytes.
    ///
    /// Assets, resources, and loaded plugin handles are not serialized.
    pub fn serialize(&self) -> Result<Vec<u8>, SceneError> {
        self.debug_assert_consistent();
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

        let bytes = SceneData {
            entities,
            systems,
            components,
        }
        .encode()?;
        Ok(bytes)
    }

    /// Replaces the scene state with serialized WasserXR scene bytes.
    ///
    /// This calls `reset` first, then recreates entities, systems, and
    /// components from the decoded scene data. Invalid individual systems or
    /// components are skipped with a warning.
    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), SceneError> {
        self.reset()?;

        let scene_data = SceneData::decode(data)?;

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

        self.debug_assert_consistent();
        Ok(())
    }

    /// Writes serialized scene bytes to `path`.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SceneError> {
        let data = self.serialize()?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Loads serialized scene bytes from `path`.
    ///
    /// If called while a system is ticking, the load is deferred until the
    /// current system runner returns.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SceneError> {
        let data = fs::read(path)?;
        if self.is_ticking {
            self.deferred_calls.push(DeferredCall::Load(data));
            return Ok(());
        }

        self.deserialize(&data)
    }

    /// Returns all loaded plugin manifest names in sorted order.
    pub fn get_plugins(&self) -> Vec<String> {
        let mut plugins: Vec<String> = self.plugins.keys().cloned().collect();
        plugins.sort();
        plugins
    }

    /// Loads, validates, and atomically registers a dynamic plugin.
    ///
    /// # Safety
    /// The library must export an immutable process-lifetime `wxr_plugin`
    /// descriptor graph. Every pointer/count pair must reference readable,
    /// correctly aligned storage of the declared length, and every name must
    /// reference a readable NUL-terminated string. All callbacks must use their
    /// declared signatures, preserve Rust aliasing rules for every supplied
    /// pointer, obey the documented allocation and ownership rules, return
    /// valid ABI values with the required lifetimes, and never unwind across
    /// the C boundary. A callback must not remove its currently executing
    /// component, asset, or system, destroy the scene, or unload the plugin
    /// whose code is executing. Registration from a callback is unsupported.
    pub unsafe fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        let plugin = unsafe { Plugin::load_dynamic(path) }?;
        self.install_plugin(plugin)
    }

    /// Validates and atomically registers a statically linked plugin.
    ///
    /// # Safety
    /// The descriptor graph and callbacks must satisfy the same contract as
    /// [`Self::load_plugin`].
    pub unsafe fn load_static_plugin(
        &mut self,
        descriptor: &'static plugin::WXRPluginDescriptor,
    ) -> Result<(), SceneError> {
        let plugin = unsafe { Plugin::load_static(descriptor) }?;
        self.install_plugin(plugin)
    }

    fn install_plugin(&mut self, plugin: Plugin) -> Result<(), SceneError> {
        let name = plugin.get_id().to_owned();
        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name).into());
        }
        // Check if there are any Plugin Definition Collisions (no two plugins should create the
        // same definition)
        for definition in plugin.definition_names() {
            if self
                .plugins
                .values()
                .any(|loaded| loaded.has_definition(definition))
            {
                return Err(PluginError::DefinitionCollision(definition.to_owned()).into());
            }
        }

        self.plugins.insert(name.clone(), plugin);

        crate::info!(self, "Plugin `{}` loaded", name);
        self.debug_assert_consistent();
        Ok(())
    }

    /// Unregisters a plugin by manifest name and removes its live objects.
    ///
    /// If called while a system is ticking, the unload is deferred until the
    /// current system runner returns.
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls
                .push(DeferredCall::UnloadPlugin(name.to_owned()));
            return Ok(());
        }

        if !self.plugins.contains_key(name) {
            return Err(PluginError::NotLoaded.into());
        }

        // Unload all systems that are still loaded with this plugin
        let systems: Vec<String> = self
            .systems
            .values()
            .filter(|system| system.get_plugin_id() == name)
            .map(|system| system.get_id().to_owned())
            .collect();

        for system_id in systems {
            self.remove_system(&system_id)?;
        }

        // Unload all the asset types that are still loaded with this plugin
        let asset_names: Vec<String> = self.plugins[name]
            .asset_names()
            .map(str::to_owned)
            .collect();

        for asset_type in asset_names {
            self.remove_assets_for_type(&asset_type);
        }

        // Unload all the components that are still loaded with this plugin
        let components: Vec<(Uuid, String)> = self
            .components
            .iter()
            .flat_map(|(entity, components)| {
                components
                    .values()
                    .filter(|component| component.get_plugin_id() == name)
                    .map(|component| (*entity, component.get_id().to_owned()))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (entity, component_id) in components {
            self.remove_component(entity, &component_id)?;
        }

        assert!(
            self.systems
                .values()
                .all(|system| system.get_plugin_id() != name)
                && self.components.values().all(|components| components
                    .values()
                    .all(|component| component.get_plugin_id() != name))
                && self.plugins[name]
                    .asset_names()
                    .all(|asset_type| !self.assets.contains_key(asset_type)),
            ecs_invariant_message!(
                "every plugin-owned object must be removed before unloading its function pointers"
            ),
        );

        self.plugins.remove(name);

        crate::info!(self, "Plugin `{}` unloaded", name);
        self.debug_assert_consistent();
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
        self.debug_assert_consistent();
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        let components = self.components.remove(&id).expect(ecs_invariant_message!(
            "every stored entity must have component storage"
        ));

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
        self.debug_assert_consistent();
        Ok(())
    }

    fn get_entity(&self, id: Uuid) -> Result<&Entity, SceneError> {
        match self.entities.get(&id) {
            Some(entity) => Ok(entity),
            None => {
                crate::warn!(self, "Entity `{}` was not found", id);
                Err(SceneError::Entity(EntityError::NotFound))
            }
        }
    }

    fn get_entity_mut(&mut self, id: Uuid) -> Result<&mut Entity, SceneError> {
        if !self.entities.contains_key(&id) {
            crate::warn!(self, "Entity `{}` was not found", id);
            return Err(SceneError::Entity(EntityError::NotFound));
        }

        Ok(self.entities.get_mut(&id).expect(ecs_invariant_message!(
            "an entity checked as present must remain present during mutable lookup"
        )))
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
    /// The system must be declared by a loaded plugin descriptor. Higher
    /// priorities run earlier in `tick`.
    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        self.add_system_from_data(SystemData { id, priority })
    }

    fn add_system_from_data(&mut self, data: SystemData) -> Result<(), SceneError> {
        let id = data.id.clone();
        if self.systems.contains_key(&id) {
            crate::warn!(self, "System `{}` already exists", id);
            return Err(SceneError::System(SystemError::AlreadyExists));
        }

        let definition = self
            .system_definition(&id)
            .ok_or(SceneError::System(SystemError::TypeNotFound))?;
        let system = System::new(Rc::clone(&definition), data.priority);
        let system_id = system.get_id().to_owned();
        self.set_logger(system_id.clone());
        if let Some(attacher) = system.get_attacher() {
            unsafe { attacher(self as *mut Scene) };
        }
        self.reset_logger();
        if !self
            .system_definition(&id)
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, &definition))
        {
            return Err(SceneError::System(SystemError::TypeNotFound));
        }
        crate::info!(self, "System `{}` added", system_id);
        self.systems.insert(id, system);
        Ok(())
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
            return Err(SceneError::System(SystemError::NotFound));
        };

        self.set_logger(id.to_owned());
        if let Some(detacher) = system.get_detacher() {
            unsafe { detacher(self as *mut Scene) };
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
                Err(SceneError::System(SystemError::NotFound))
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
                Err(SceneError::System(SystemError::NotFound))
            }
        }
    }

    /// Adds a component to an entity by component id.
    ///
    /// The component must be declared by a loaded plugin descriptor.
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        // Check if this component already exists
        if entity_components.contains_key(&component_id) {
            crate::debug!(
                self,
                "Component `{}` already exists on entity `{}`",
                component_id,
                entity_id
            );
            return Err(SceneError::Component(ComponentError::AlreadyExists));
        }

        let Some(definition) = self.component_definition(&component_id) else {
            crate::warn!(self, "Component `{}` could not be created", component_id);
            return Err(SceneError::Component(ComponentError::TypeNotFound));
        };

        let Some(mut component) = Component::create(Rc::clone(&definition), self) else {
            crate::warn!(self, "Component creator for `{}` failed", component_id);
            return Err(SceneError::Component(ComponentError::CreatorFailed));
        };

        component.deserialize_fields(data.fields);

        if !self
            .component_definition(&component_id)
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, &definition))
        {
            return Err(SceneError::Component(ComponentError::TypeNotFound));
        }

        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            return Err(SceneError::Entity(EntityError::NotFound));
        };
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        // Check if this component already exists
        let Some(component) = entity_components.remove(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for removal",
                component_id,
                entity_id
            );
            return Err(SceneError::Component(ComponentError::AlreadyExists));
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
            return Err(SceneError::Entity(EntityError::NotFound));
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
    ///
    /// # Safety
    /// Every dynamically loaded plugin must continue to export a valid static
    /// descriptor graph and uphold all callback contracts described by
    /// [`Self::load_plugin`] when its source path is loaded again.
    pub unsafe fn reload(&mut self) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls.push(DeferredCall::Reload);
            return Ok(());
        }

        // Serialize the important data
        let plugins: Vec<(String, String)> = self
            .plugins
            .iter()
            .filter_map(|(name, plugin)| {
                plugin
                    .source_path()
                    .map(|path| (name.clone(), path.to_owned()))
            })
            .collect();
        let serialization_data = self.serialize().inspect_err(|error| {
            crate::error!(self, "Failed to serialize the scene: {error}");
        })?;

        // Remove the plugins
        for (name, _) in &plugins {
            if self.unload_plugin(name).is_err() {
                crate::error!(self, "Failed to unload the plugin: {}", name);
            }
        }

        // Reload the plugins
        for (_, path) in plugins {
            if let Err(SceneError::Plugin(PluginError::Linking(err))) =
                unsafe { self.load_plugin(path) }
            {
                crate::error!(
                    self,
                    "Plugin has a linking error while hotreloading: {}",
                    err
                );
            }
        }

        // Deserialize the scene data (Scene reset also happens in here)
        self.deserialize(&serialization_data).inspect_err(|error| {
            crate::error!(self, "Failed to deserialize the scene: {error}");
        })?;

        Ok(())
    }

    fn run_system(
        &mut self,
        system_id: &str,
        groups: &[SelectionGroup],
        runner: Runner,
        delta: f32,
    ) {
        debug_assert!(
            self.is_ticking,
            ecs_invariant_message!("systems may only run during a scene tick"),
        );
        self.set_logger(system_id.to_owned());
        let mut entities: Vec<Vec<WXREntity>> = Vec::with_capacity(groups.len());
        for group in groups {
            let mut selected = Vec::new();
            for entity in self.entities.keys().copied() {
                let entity_components = &self.components[&entity];
                if group.matches(|component| entity_components.contains_key(component)) {
                    selected.push(WXREntity::from_uuid(entity));
                }
            }
            entities.push(selected);
        }

        let sizes: Vec<usize> = entities.iter().map(|group| group.len()).collect();
        let sizes_ptr = if sizes.is_empty() {
            std::ptr::null()
        } else {
            sizes.as_ptr()
        };

        let entity_pointers: Vec<*const WXREntity> = entities
            .iter()
            .map(|group| {
                if group.is_empty() {
                    std::ptr::null()
                } else {
                    group.as_ptr()
                }
            })
            .collect();
        let entities_ptr = if entity_pointers.is_empty() {
            std::ptr::null()
        } else {
            entity_pointers.as_ptr()
        };

        unsafe {
            runner(
                self as *mut Scene,
                delta,
                entities_ptr,
                sizes_ptr,
                groups.len(),
            );
        }
        self.reset_logger();
    }

    fn run_deferred_calls(&mut self) {
        debug_assert!(
            !self.is_ticking,
            ecs_invariant_message!(
                "deferred scene mutations must run after the active system returns"
            ),
        );
        let deferred_calls = std::mem::take(&mut self.deferred_calls);

        for deferred_call in deferred_calls {
            match deferred_call {
                DeferredCall::Reload => {
                    if unsafe { self.reload() }.is_err() {
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
            let Some(system) = self.systems.get_mut(&system_id) else {
                continue;
            };

            let definition = system.definition();
            let delta = system.tick_delta();

            self.is_ticking = true;
            self.run_system(&system_id, definition.groups(), definition.runner(), delta);
            self.is_ticking = false;

            self.run_deferred_calls();
        }

        let should_exit = self.should_exit;
        self.should_exit = false;
        self.debug_assert_consistent();
        !should_exit
    }

    fn get_component(&self, entity_id: Uuid, component_id: &str) -> Result<&Component, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            crate::warn!(
                self,
                "Entity `{}` was not found for component lookup",
                entity_id
            );
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        let Some(component) = entity_components.get(component_id) else {
            return Err(SceneError::Component(ComponentError::NotFound));
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
            .map(|field| -> Result<_, SceneError> {
                let ptr = unsafe { component.get_field_ptr(field) }?;
                assert!(
                    !ptr.is_null(),
                    "internal ECS invariant violated; this is a component implementation bug: component field getters must return non-null pointers",
                );
                Ok(ptr)
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
            return Err(SceneError::Component(
                crate::scene::component::ComponentError::FieldParsing,
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
            return Err(SceneError::Component(
                crate::scene::component::ComponentError::FieldParsing,
            ));
        }

        // Check if there are no duplicates in the fields
        // This is not a thorough check but at least a good error prevention
        for index in 0..fields.len() {
            if fields[index + 1..].contains(&fields[index]) {
                return Err(SceneError::Component(
                    crate::scene::component::ComponentError::FieldParsing,
                ));
            }
        }

        let component = self.get_component(entity_id, component_id)?;
        Q::for_each_mutable_field(fields, |field| {
            if component
                .is_field_mutable(field)
                .map_err(SceneError::Component)?
            {
                Ok(())
            } else {
                Err(SceneError::Component(
                    crate::scene::component::ComponentError::FieldNotMutable,
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

    fn remove_assets_for_type(&mut self, asset_type: &str) {
        if let Some(assets) = self.assets.remove(asset_type) {
            for asset in assets.into_values() {
                self.set_logger(asset_type.to_owned());
                asset.destroy(self);
                self.reset_logger();
            }
        }

        crate::info!(self, "Assets of type `{}` removed", asset_type);
    }

    /// Ensures an asset of `asset_type` with `data_string` is loaded.
    ///
    /// If the asset already exists, this is a no-op. Otherwise its
    /// manifest-registered asset creator is called.
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

        let asset_type_data = self
            .asset_definition(asset_type)
            .ok_or(SceneError::Asset(AssetError::AssetTypeNotFound))?;
        let asset = asset_type_data
            .create_asset(self, data_string)
            .map_err(SceneError::Asset)?;

        if !self
            .asset_definition(asset_type)
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, &asset_type_data))
        {
            asset.destroy(self);
            return Err(SceneError::Asset(AssetError::AssetTypeNotFound));
        }

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
            return Err(SceneError::Asset(AssetError::AssetTypeNotFound));
        };

        let Some(asset) = assets.get(data_string) else {
            crate::debug!(
                self,
                "Asset `{}` with data `{}` was not found",
                asset_type,
                data_string
            );
            return Err(SceneError::Asset(AssetError::InvalidAsset));
        };

        Ok(asset)
    }

    pub(crate) unsafe fn get_asset_field_ptrs(
        &self,
        asset_type: &str,
        data_string: &str,
        fields: &[&str],
    ) -> Result<Vec<*mut c_void>, SceneError> {
        let Some(asset_type_data) = self.asset_definition(asset_type) else {
            crate::debug!(
                self,
                "Asset type `{}` was not found for field lookup",
                asset_type
            );
            return Err(SceneError::Asset(AssetError::AssetTypeNotFound));
        };
        let asset = self.get_asset(asset_type, data_string)?;

        fields
            .iter()
            .map(|field| {
                unsafe { asset_type_data.get_field_ptr(asset, field) }.map_err(SceneError::Asset)
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
            return Err(SceneError::Asset(AssetError::FieldParsing));
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for field list lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::Component(ComponentError::NotFound));
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for plugin lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::Component(ComponentError::NotFound));
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
            return Err(SceneError::Entity(EntityError::NotFound));
        };

        let Some(component) = entity_components.get(component_id) else {
            crate::warn!(
                self,
                "Component `{}` was not found on entity `{}` for field type lookup",
                component_id,
                entity_id
            );
            return Err(SceneError::Component(ComponentError::NotFound));
        };

        component
            .get_field_type(field_id)
            .map_err(SceneError::Component)
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
            .map_err(SceneError::Component)
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
            .map_err(SceneError::Component)
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
            .map_err(SceneError::Component)
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
            .map_err(SceneError::Component)
    }

    /// Requests that the next `tick` return `false`.
    pub fn should_exit(&mut self) {
        self.should_exit = true;
    }
}
