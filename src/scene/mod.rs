pub mod component;
mod entity;
mod plugin;
mod system;

use component::Component;
use entity::Entity;
use plugin::Plugin;
use system::System;

use crate::error::SceneError;
use crate::scene::system::{Runner, Selector};

use std::collections::HashMap;

use uuid::Uuid;

pub struct Scene {
    entities: HashMap<Uuid, Entity>,
    plugins: HashMap<String, Plugin>,
    systems: HashMap<String, System>,
    components: HashMap<Uuid, HashMap<String, Component>>,
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
        Ok(())
    }

    pub fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        if self.plugins.contains_key(&path) {
            return Err(SceneError::PluginAlreadyLoaded);
        }

        self.plugins.insert(
            path.to_owned(),
            Plugin::new(path).map_err(SceneError::PluginLoading)?,
        );

        Ok(())
    }

    pub fn unload_plugin(&mut self, path: &str) -> Result<(), SceneError> {
        if !self.plugins.contains_key(path) {
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

        Ok(())
    }

    pub fn add_entity(&mut self) -> Uuid {
        let entity = Entity::new();
        let uuid = entity.get_id();
        self.entities.insert(uuid, entity);
        self.components.insert(uuid, HashMap::new());
        uuid
    }

    pub fn remove_entity(&mut self, id: Uuid) -> Result<(), SceneError> {
        let Some(_) = self.entities.remove(&id) else {
            return Err(SceneError::EntityNotFound);
        };
        self.components.remove(&id);

        let Some(components) = self.components.remove(&id) else {
            log::error!(
                "Entity `{}` had no components carrier associated. This is a bug. Please report it",
                id
            );
            return Ok(());
        };

        let components: Vec<&String> = components.keys().collect();

        for i in components {
            self.remove_component(id, i)?;
        }

        Ok(())
    }

    fn get_entity(&self, id: Uuid) -> Result<&Entity, SceneError> {
        match self.entities.get(&id) {
            Some(entity) => Ok(entity),
            None => Err(SceneError::EntityNotFound),
        }
    }

    fn get_entity_mut(&mut self, id: Uuid) -> Result<&mut Entity, SceneError> {
        match self.entities.get_mut(&id) {
            Some(entity) => Ok(entity),
            None => Err(SceneError::EntityNotFound),
        }
    }

    pub fn get_entity_name(&self, id: Uuid) -> Result<&str, SceneError> {
        let entity = self.get_entity(id)?;
        Ok(entity.get_name())
    }

    pub fn set_entity_name(&mut self, id: Uuid, name: String) -> Result<(), SceneError> {
        let entity = self.get_entity_mut(id)?;
        entity.set_name(name);
        Ok(())
    }

    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        if self.systems.contains_key(&id) {
            return Err(SceneError::SystemAlreadyExists);
        }

        let system: Option<System> = self
            .plugins
            .values()
            .find_map(|plugin| System::new(id.clone(), plugin, priority).ok());

        match system {
            Some(system) => {
                let attacher = system.get_attacher();
                unsafe {
                    attacher(self as *mut Scene);
                }
                self.systems.insert(id, system);
                Ok(())
            }
            None => Err(SceneError::SystemCreation),
        }
    }

    pub fn remove_system(&mut self, id: &str) -> Result<(), SceneError> {
        let Some(system) = self.systems.remove(id) else {
            return Err(SceneError::SystemNotFound);
        };

        let detacher = system.get_detacher();
        unsafe {
            detacher(self as *mut Scene);
        }
        Ok(())
    }

    pub fn add_component(
        &mut self,
        entity_id: Uuid,
        component_id: String,
    ) -> Result<(), SceneError> {
        // Check if entity exists
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        if entity_components.contains_key(&component_id) {
            return Err(SceneError::ComponentAlreadyExists);
        }

        // Build the plugin
        let Some(plugin) = self
            .plugins
            .values()
            .find_map(|plugin| Component::new(component_id.clone(), plugin).ok())
        else {
            return Err(SceneError::ComponentCreation);
        };

        entity_components.insert(component_id, plugin);

        Ok(())
    }

    pub fn remove_component(
        &mut self,
        entity_id: Uuid,
        component_id: &str,
    ) -> Result<(), SceneError> {
        // Check if entity exists
        let Some(entity_components) = self.components.get_mut(&entity_id) else {
            return Err(SceneError::EntityNotFound);
        };

        // Check if this component already exists
        let Some(_) = entity_components.remove(component_id) else {
            return Err(SceneError::ComponentAlreadyExists);
        };

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

    pub fn tick(&mut self) -> bool {
        let functions: Vec<(usize, Selector, Runner)> = self
            .systems
            .values()
            .map(|system| {
                (
                    system.get_groups(),
                    system.get_selector(),
                    system.get_runner(),
                )
            })
            .collect();

        for (groups, selector, runner) in functions {
            self.run_system(groups, selector, runner);
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
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get(component_id) else {
            return Err(SceneError::ComponentNotFound);
        };

        component
            .get(field_id)
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
            return Err(SceneError::EntityNotFound);
        };

        let Some(component) = entity_components.get_mut(component_id) else {
            return Err(SceneError::ComponentNotFound);
        };

        component
            .set(field_id, data)
            .map_err(SceneError::ComponentFieldError)
    }
}
