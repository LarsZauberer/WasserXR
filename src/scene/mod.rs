pub mod component;
pub(crate) mod entity;
pub(crate) mod plugin;
pub(crate) mod serialization;
pub(crate) mod system;

use component::{Component, FieldType};
use entity::Entity;
use log::{error, warn};
use plugin::Plugin;
use system::System;

use crate::error::{PluginError, SceneError};
use crate::scene::serialization::{ComponentData, SceneData, SystemData};
use crate::scene::system::{Runner, Selector};

use std::{collections::HashMap, fs, path::Path};

use uuid::Uuid;

enum DeferredCall {
    Reload,
    UnloadPlugin(String),
    RemoveSystem(String),
}

pub struct Scene {
    entities: HashMap<Uuid, Entity>,
    plugins: HashMap<String, Plugin>,
    systems: HashMap<String, System>,
    components: HashMap<Uuid, HashMap<String, Component>>,

    deferred_calls: Vec<DeferredCall>,
    is_ticking: bool,
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
            deferred_calls: Vec::new(),
            is_ticking: false,
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) -> Result<(), SceneError> {
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
        log::info!("Scene reset");
        Ok(())
    }

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

    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), SceneError> {
        self.reset()?;

        let scene_data = SceneData::decode(data).map_err(SceneError::Deserialization)?;

        for entity_data in scene_data.entities {
            let entity_id = entity_data.id;
            if self.entities.contains_key(&entity_id) {
                log::warn!("Entity `{}` is duplicated in serialized scene", entity_id);
                continue;
            }

            self.entities
                .insert(entity_id, Entity::deserialize(entity_data));
            self.components.insert(entity_id, HashMap::new());
        }

        for system_data in scene_data.systems {
            let system_id = system_data.id.clone();
            if let Err(error) = self.add_system_from_data(system_data) {
                log::warn!(
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
                log::warn!(
                    "Component `{}` on entity `{}` could not be deserialized: {:?}",
                    component_id,
                    entity_id,
                    error
                );
            }
        }

        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SceneError> {
        let data = self.serialize()?;
        fs::write(path, data).map_err(|error| SceneError::FileIo(error.to_string()))
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SceneError> {
        let data = fs::read(path).map_err(|error| SceneError::FileIo(error.to_string()))?;
        self.deserialize(&data)
    }

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

    pub fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        if self.plugins.contains_key(&path) {
            log::warn!("Plugin `{}` is already loaded", path);
            return Err(SceneError::PluginAlreadyLoaded);
        }

        let plugin = Plugin::new(path.to_owned()).map_err(SceneError::PluginLoading)?;
        self.plugins.insert(path.to_owned(), plugin);

        log::info!("Plugin `{}` loaded", path);
        Ok(())
    }

    pub fn unload_plugin(&mut self, path: &str) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls
                .push(DeferredCall::UnloadPlugin(path.to_owned()));
            return Ok(());
        }

        if path.is_empty() {
            log::warn!("Static plugin cannot be unloaded");
            return Err(SceneError::StaticPluginUnload);
        }

        if !self.plugins.contains_key(path) {
            log::warn!("Plugin `{}` is not loaded", path);
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

        log::info!("Plugin `{}` unloaded", path);
        Ok(())
    }

    pub fn add_entity(&mut self) -> Uuid {
        let entity = Entity::new();
        let uuid = entity.get_id();
        self.entities.insert(uuid, entity);
        self.components.insert(uuid, HashMap::new());
        log::info!("Entity `{}` added", uuid);
        uuid
    }

    pub fn get_entities(&self) -> Vec<Uuid> {
        let mut entities: Vec<Uuid> = self.entities.keys().copied().collect();
        entities.sort();
        entities
    }

    pub fn remove_entity(&mut self, id: Uuid) -> Result<(), SceneError> {
        let Some(_) = self.entities.remove(&id) else {
            log::warn!("Entity `{}` was not found for removal", id);
            return Err(SceneError::EntityNotFound);
        };

        let Some(components) = self.components.remove(&id) else {
            log::error!(
                "Entity `{}` had no components carrier associated. This is a bug. Please report it",
                id
            );
            return Ok(());
        };

        for component_id in components.keys() {
            log::info!("Component `{}` removed from entity `{}`", component_id, id);
        }

        log::info!("Entity `{}` removed", id);
        Ok(())
    }

    fn get_entity(&self, id: Uuid) -> Result<&Entity, SceneError> {
        match self.entities.get(&id) {
            Some(entity) => Ok(entity),
            None => {
                log::warn!("Entity `{}` was not found", id);
                Err(SceneError::EntityNotFound)
            }
        }
    }

    fn get_entity_mut(&mut self, id: Uuid) -> Result<&mut Entity, SceneError> {
        match self.entities.get_mut(&id) {
            Some(entity) => Ok(entity),
            None => {
                log::warn!("Entity `{}` was not found", id);
                Err(SceneError::EntityNotFound)
            }
        }
    }

    pub fn get_entity_name(&self, id: Uuid) -> Result<&str, SceneError> {
        let entity = self.get_entity(id)?;
        Ok(entity.get_name())
    }

    pub fn set_entity_name(&mut self, id: Uuid, name: String) -> Result<(), SceneError> {
        let entity = self.get_entity_mut(id)?;
        entity.set_name(name);
        log::info!("Entity `{}` renamed", id);
        Ok(())
    }

    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        self.add_system_from_data(SystemData { id, priority })
    }

    fn add_system_from_data(&mut self, data: SystemData) -> Result<(), SceneError> {
        let id = data.id.clone();
        if self.systems.contains_key(&id) {
            log::warn!("System `{}` already exists", id);
            return Err(SceneError::SystemAlreadyExists);
        }

        let system: Option<System> = self
            .plugins_dynamic_first()
            .find_map(|plugin| System::deserialize(data.clone(), plugin).ok());

        match system {
            Some(system) => {
                let attacher = system.get_attacher();
                unsafe {
                    attacher(self as *mut Scene);
                }
                log::info!("System `{}` added", system.get_id());
                self.systems.insert(id, system);
                Ok(())
            }
            None => {
                log::warn!("System `{}` could not be created", id);
                Err(SceneError::SystemCreation)
            }
        }
    }

    pub fn remove_system(&mut self, id: &str) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls
                .push(DeferredCall::RemoveSystem(id.to_owned()));
            return Ok(());
        }

        let Some(system) = self.systems.remove(id) else {
            log::warn!("System `{}` was not found for removal", id);
            return Err(SceneError::SystemNotFound);
        };

        let detacher = system.get_detacher();
        unsafe {
            detacher(self as *mut Scene);
        }
        log::info!("System `{}` removed", id);
        Ok(())
    }

    pub fn get_systems(&self) -> Vec<String> {
        let mut systems: Vec<String> = self.systems.keys().cloned().collect();
        systems.sort();
        systems
    }

    pub fn get_system_priority(&self, system_id: &str) -> Result<usize, SceneError> {
        match self.systems.get(system_id) {
            Some(system) => Ok(system.get_priority()),
            None => {
                log::warn!("System `{}` was not found for priority lookup", system_id);
                Err(SceneError::SystemNotFound)
            }
        }
    }

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
            log::debug!(
                "Entity `{}` was not found for component addition",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        if entity_components.contains_key(&component_id) {
            log::debug!(
                "Component `{}` already exists on entity `{}`",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentAlreadyExists);
        }

        // Build the plugin
        let Some(component) = self
            .plugins_dynamic_first()
            .find_map(|plugin| Component::deserialize(component_id.clone(), plugin).ok())
        else {
            log::warn!("Component `{}` could not be created", component_id);
            return Err(SceneError::ComponentCreation);
        };

        let entity_components = self
            .components
            .get_mut(&entity_id)
            .expect("entity was checked before creating the component");
        let component_id_for_log = component_id.clone();
        entity_components.insert(component_id, component);

        if let Some(component) = entity_components.get_mut(&component_id_for_log) {
            component.deserialize_fields(data.fields);
        }

        log::info!(
            "Component `{}` added to entity `{}`",
            component_id_for_log,
            entity_id
        );
        Ok(())
    }

    pub fn remove_component(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<(), SceneError> {
        // Check if entity exists
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            log::warn!("Entity `{}` was not found for component removal", entity_id);
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        let Some(_) = entity_components.remove(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for removal",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentAlreadyExists);
        };

        log::info!(
            "Component `{}` removed from entity `{}`",
            component_id,
            entity_id
        );
        Ok(())
    }

    pub fn get_entity_components(&self, entity_id: Uuid) -> Result<Vec<String>, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component list lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let mut components: Vec<String> = entity_components.keys().cloned().collect();
        components.sort();
        Ok(components)
    }

    pub fn has_component(&self, entity_id: Uuid, component_id: &str) -> bool {
        self.components
            .get(&entity_id)
            .is_some_and(|components| components.contains_key(component_id))
    }

    /// This is reload the Scene immediately. It will serialize the current scene data, unload all
    /// the plugins and reload them in. At the end it will deserialize the scene again.
    pub fn reload(&mut self) -> Result<(), SceneError> {
        if self.is_ticking {
            self.deferred_calls.push(DeferredCall::Reload);
            return Ok(());
        }

        // Serialize the important data
        let plugins: Vec<String> = self
            .plugins
            .keys()
            .filter_map(|id| if id == "" { None } else { Some(id.to_owned()) })
            .collect();
        let serialization_data = self.serialize()?;

        // Reset the scene
        self.reset()?;

        // Remove the plugins
        for plugin in &plugins {
            if self.unload_plugin(plugin).is_err() {
                error!("Failed to unload the plugin: {}", plugin);
            }
        }

        // Reload the plugins
        for plugin in plugins {
            match self.load_plugin(plugin) {
                Err(SceneError::PluginLoading(PluginError::InvalidSymbol)) => {
                    error!(
                        "Symbol has a null byte during the plugin reloading! This is a bug! Please report this!"
                    );
                }
                Err(SceneError::PluginLoading(PluginError::LinkingError(err))) => {
                    error!("Plugin has a linking error while hotreloading: {}", err);
                }
                _ => {}
            }
        }

        // Deserialize the scene data
        self.deserialize(&serialization_data)?;

        Ok(())
    }

    fn run_system(&mut self, groups: usize, selector: Selector, runner: Runner) {
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
    }

    fn run_deferred_calls(&mut self) {
        let deferred_calls = std::mem::take(&mut self.deferred_calls);

        for deferred_call in deferred_calls {
            match deferred_call {
                DeferredCall::Reload => {
                    if self.reload().is_err() {
                        warn!("Reload Failed! The scene has maybe been partially been reloaded.");
                    }
                }
                DeferredCall::UnloadPlugin(plugin_id) => {
                    if self.unload_plugin(&plugin_id).is_err() {
                        warn!("Deferred unload of plugin `{}` failed", plugin_id);
                    }
                }
                DeferredCall::RemoveSystem(system_id) => {
                    if self.remove_system(&system_id).is_err() {
                        warn!("Deferred removal of system `{}` failed", system_id);
                    }
                }
            }
        }
    }

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
            self.run_system(groups, selector, runner);
            self.is_ticking = false;

            self.run_deferred_calls();
        }

        true
    }

    pub fn get<T>(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<&T, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field read",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for field read",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .get(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    pub fn get_mut<T>(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<&mut T, SceneError> {
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for mutable component field read",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get_mut(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for mutable field read",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .get_mut(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    pub fn set<T>(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
        data: &T,
    ) -> Result<(), SceneError> {
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field update",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get_mut(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for field update",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .set(field_id, data)
            .map_err(SceneError::ComponentFieldError)
    }

    pub fn r#move<T>(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
        data: T,
    ) -> Result<(), SceneError> {
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field move",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get_mut(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for field move",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .r#move(field_id, data)
            .map_err(SceneError::ComponentFieldError)
    }

    pub fn take<T>(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<T, SceneError> {
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field take",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get_mut(component_id) else {
            log::warn!(
                "Component `{}` was not found on entity `{}` for field take",
                component_id,
                entity_id
            );
            return Err(SceneError::ComponentNotFound);
        };

        component
            .take(field_id)
            .map_err(SceneError::ComponentFieldError)
    }

    pub fn get_component_fields(
        &self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<Vec<String>, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field list lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            log::warn!(
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

    pub fn get_component_field_type(
        &self,
        entity_id: Uuid,
        component_id: &str,
        field_id: &str,
    ) -> Result<FieldType, SceneError> {
        let Some(entity_components) = self.components.get(&entity_id) else {
            log::warn!(
                "Entity `{}` was not found for component field type lookup",
                entity_id
            );
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            log::warn!(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::ComponentError,
        scene::{
            component::{FieldType, Schema, SerializedBytes},
            serialization::{ComponentData, FieldData, SceneData},
        },
    };
    use std::{
        ffi::c_void,
        sync::{
            LazyLock, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
    };

    #[repr(C)]
    struct SceneCounter {
        value: i64,
    }

    #[derive(Default, Debug, PartialEq, Eq)]
    struct SceneOwnedValue {
        value: String,
    }

    #[repr(C)]
    struct SceneOwner {
        value: SceneOwnedValue,
    }

    unsafe extern "C" fn scene_counter_getter(data: *const c_void) -> *const c_void {
        unsafe { &(*(data as *const SceneCounter)).value as *const i64 as *const c_void }
    }

    unsafe extern "C" fn scene_counter_getter_mut(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut SceneCounter)).value as *mut i64 as *mut c_void }
    }

    unsafe extern "C" fn scene_counter_setter(data: *mut c_void, value: *const c_void) {
        unsafe {
            (*(data as *mut SceneCounter)).value = *(value as *const i64);
        }
    }

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

    unsafe extern "C" fn scene_owner_getter(data: *const c_void) -> *const c_void {
        unsafe { &(*(data as *const SceneOwner)).value as *const SceneOwnedValue as *const c_void }
    }

    unsafe extern "C" fn scene_owner_mover(data: *mut c_void, value: *mut c_void) {
        unsafe {
            (*(data as *mut SceneOwner)).value = *Box::from_raw(value as *mut SceneOwnedValue);
        }
    }

    unsafe extern "C" fn scene_owner_taker(data: *mut c_void) -> *mut c_void {
        unsafe {
            Box::into_raw(Box::new(std::mem::take(
                &mut (*(data as *mut SceneOwner)).value,
            ))) as *mut c_void
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_counter() -> *mut c_void {
        Box::into_raw(Box::new(SceneCounter { value: 1 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_scene_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut SceneCounter));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_scene_owner() -> *mut c_void {
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
    unsafe extern "C" fn wxr_schema_scene_counter(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::Long,
                Some(scene_counter_getter),
                Some(scene_counter_getter_mut),
                Some(scene_counter_setter),
                None,
                None,
                Some(scene_counter_serializer),
                Some(scene_counter_deserializer),
            );
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_scene_owner(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::Blob,
                Some(scene_owner_getter),
                None,
                None,
                Some(scene_owner_mover),
                Some(scene_owner_taker),
                None,
                None,
            );
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
    static SCENE_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static SCENE_RELOAD_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static SCENE_DEFERRED_REMOVE_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

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
    fn scene_set_component_field_for_existing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        scene.set(entity, "scene_counter", "value", &9_i64).unwrap();

        assert_eq!(
            *scene.get::<i64>(entity, "scene_counter", "value").unwrap(),
            9
        );
    }

    #[test]
    fn scene_get_mut_component_field_for_existing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        *scene
            .get_mut::<i64>(entity, "scene_counter", "value")
            .unwrap() = 17;

        assert_eq!(
            *scene.get::<i64>(entity, "scene_counter", "value").unwrap(),
            17
        );
    }

    #[test]
    fn scene_get_mut_component_field_without_getter_mut() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        assert_eq!(
            scene.get_mut::<SceneOwnedValue>(entity, "scene_owner", "value"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNoGetterMut
            ))
        );
    }

    #[test]
    fn scene_move_component_field_for_existing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();

        scene
            .r#move(
                entity,
                "scene_owner",
                "value",
                SceneOwnedValue {
                    value: "moved".to_owned(),
                },
            )
            .unwrap();

        assert_eq!(
            scene
                .get::<SceneOwnedValue>(entity, "scene_owner", "value")
                .unwrap(),
            &SceneOwnedValue {
                value: "moved".to_owned(),
            }
        );
    }

    #[test]
    fn scene_take_component_field_for_existing_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_owner".to_owned())
            .unwrap();
        scene
            .r#move(
                entity,
                "scene_owner",
                "value",
                SceneOwnedValue {
                    value: "taken".to_owned(),
                },
            )
            .unwrap();

        let value = scene
            .take::<SceneOwnedValue>(entity, "scene_owner", "value")
            .unwrap();

        assert_eq!(
            value,
            SceneOwnedValue {
                value: "taken".to_owned(),
            }
        );
        assert_eq!(
            scene
                .get::<SceneOwnedValue>(entity, "scene_owner", "value")
                .unwrap(),
            &SceneOwnedValue::default()
        );
    }

    #[test]
    fn scene_move_component_field_without_mover() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.r#move(entity, "scene_counter", "value", 2_i64),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNoMover
            ))
        );
    }

    #[test]
    fn scene_take_component_field_without_taker() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();

        assert_eq!(
            scene.take::<i64>(entity, "scene_counter", "value"),
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNoTaker
            ))
        );
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
            scene.get::<i64>(entity, "scene_counter", "value"),
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
            FieldType::Long
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

        assert_eq!(
            *scene.get::<i64>(entity, "scene_counter", "value").unwrap(),
            1
        );
        scene.remove_system("scene_cleanup_system").unwrap();
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
        scene
            .set(entity, "scene_counter", "value", &42_i64)
            .unwrap();
        scene
            .add_system("scene_reload_counted_system".to_owned(), 1)
            .unwrap();

        scene.reload().unwrap();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "Reloaded");
        assert_eq!(
            *scene.get::<i64>(entity, "scene_counter", "value").unwrap(),
            42
        );
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
        scene.set(entity, "scene_counter", "value", &7_i64).unwrap();
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
        assert_eq!(
            *scene.get::<i64>(entity, "scene_counter", "value").unwrap(),
            7
        );
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
    fn scene_deserialize_round_trip() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene.set_entity_name(entity, "Player".to_owned()).unwrap();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        scene
            .set(entity, "scene_counter", "value", &42_i64)
            .unwrap();
        scene
            .add_system("scene_cleanup_system".to_owned(), 3)
            .unwrap();

        let serialized = scene.serialize().unwrap();
        let mut loaded = Scene::new();
        loaded.deserialize(&serialized).unwrap();

        assert_eq!(loaded.get_entity_name(entity).unwrap(), "Player");
        assert_eq!(
            *loaded.get::<i64>(entity, "scene_counter", "value").unwrap(),
            42
        );
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

        assert_eq!(
            *scene
                .get::<i64>(entity_data.id, "scene_counter", "value")
                .unwrap(),
            1
        );
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
        scene
            .set(entity, "scene_counter", "value", &11_i64)
            .unwrap();

        let path = std::env::temp_dir().join(format!("wasserxr-{}.scene", Uuid::now_v7()));
        scene.save(&path).unwrap();

        let mut loaded = Scene::new();
        loaded.load(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded.get_entity_name(entity).unwrap(), "Saved");
        assert_eq!(
            *loaded.get::<i64>(entity, "scene_counter", "value").unwrap(),
            11
        );
    }

    #[test]
    fn scene_load_missing_file() {
        let path = std::env::temp_dir().join(format!("wasserxr-missing-{}.scene", Uuid::now_v7()));
        let mut scene = Scene::new();

        assert!(matches!(scene.load(path), Err(SceneError::FileIo(_))));
    }
}
