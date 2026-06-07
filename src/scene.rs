use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    component::Component,
    entity::Entity,
    error::{PluginError, SceneError},
    plugin::Plugin,
    system::{System, SystemFunctions},
};

pub struct Scene {
    entities: HashMap<Uuid, Entity>,
    plugins: HashMap<String, Plugin>,
    systems: HashMap<String, System>,
}

impl Scene {
    pub fn new() -> Self {
        let mut plugins = HashMap::new();
        plugins.insert("".to_owned(), Plugin::new_static());
        Self {
            entities: HashMap::new(),
            plugins: plugins,
            systems: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        if self.plugins.contains_key(&path) {
            Err(SceneError::PluginAlreadyLoaded)
        } else {
            let plugin = Plugin::new(path.clone()).map_err(|error| match error {
                PluginError::LinkingError(msg) => {
                    log::error!("Linking Error: {}", msg);
                    SceneError::PluginLoading(PluginError::LinkingError(msg))
                }
                _ => SceneError::PluginLoading(error),
            })?;
            self.plugins.insert(path, plugin);
            Ok(())
        }
    }

    pub fn unload_plugin(&mut self, path: &str) -> Result<(), SceneError> {
        if let Some(_) = self.plugins.remove(path) {
            Ok(())
        } else {
            Err(SceneError::PluginNotFound)
        }
    }

    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        if self.systems.contains_key(&id) {
            Err(SceneError::SystemAlreadyExists)
        } else {
            let Some(system) = self
                .plugins
                .values()
                .find_map(|plugin| System::new(id.clone(), plugin, priority).ok())
            else {
                return Err(SceneError::SystemCreation);
            };
            system.get_functions().attach(self);
            self.systems.insert(id, system);
            Ok(())
        }
    }

    pub fn remove_system(&mut self, id: &str) -> Result<(), SceneError> {
        if let Some(system) = self.systems.remove(id) {
            system.get_functions().detach(self);
            Ok(())
        } else {
            Err(SceneError::SystemNotFound)
        }
    }

    pub fn tick(&mut self) {
        let mut systems: Vec<&System> = self.systems.values().collect();
        systems.sort();

        let functions: Vec<SystemFunctions> = systems
            .iter()
            .rev()
            .map(|system| system.get_functions().clone())
            .collect();

        for i in functions {
            i.run(self);
        }
    }

    pub fn add_entity(&mut self, name: Option<String>) -> Uuid {
        let entity = {
            if let Some(name) = name {
                Entity::new_with_name(name)
            } else {
                Entity::new()
            }
        };
        let uuid = entity.get_uuid();
        self.entities.insert(uuid, entity);
        uuid
    }

    pub fn remove_entity(&mut self, uuid: Uuid) -> Result<(), SceneError> {
        if let Some(_) = self.entities.remove(&uuid) {
            // TODO: Remove all the components associated
            Ok(())
        } else {
            Err(SceneError::EntityNotFound)
        }
    }

    pub fn get_entities(&self) -> Vec<&Uuid> {
        self.entities.keys().collect()
    }

    pub fn add_component(&mut self, uuid: Uuid, component_id: &str) -> Result<(), SceneError> {
        let Some(entity) = self.entities.get_mut(&uuid) else {
            return Err(SceneError::EntityNotFound);
        };
        if entity.component_exists(component_id) {
            return Err(SceneError::ComponentAlreadyExists);
        }

        let Some(component) = self
            .plugins
            .values()
            .find_map(|plugin| Component::new(component_id.to_owned(), plugin).ok())
        else {
            return Err(SceneError::ComponentCreation);
        };

        let res = entity.add_component(component);
        assert!(res.is_ok());

        Ok(())
    }

    pub fn remove_component(&mut self, uuid: Uuid, component_id: &str) -> Result<(), SceneError> {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            entity
                .remove_component(component_id)
                .map_err(|_| SceneError::ComponentNotFound)
        } else {
            Err(SceneError::EntityNotFound)
        }
    }

    pub fn get<T>(&self, uuid: Uuid, component_id: &str, field_id: &str) -> Result<T, SceneError> {
        let Some(entity) = self.entities.get(&uuid) else {
            return Err(SceneError::EntityNotFound);
        };
        let Some(component) = entity.get_component(component_id) else {
            return Err(SceneError::ComponentNotFound);
        };
        component
            .get(field_id)
            .map_err(|error| SceneError::ComponentFieldError(error))
    }

    pub fn set<T>(
        &mut self,
        uuid: Uuid,
        component_id: &str,
        field_id: &str,
        data: &T,
    ) -> Result<(), SceneError> {
        let Some(entity) = self.entities.get_mut(&uuid) else {
            return Err(SceneError::EntityNotFound);
        };
        let Some(component) = entity.get_component_mut(component_id) else {
            return Err(SceneError::ComponentNotFound);
        };
        component
            .set(field_id, data)
            .map_err(|error| SceneError::ComponentFieldError(error))
    }
}
