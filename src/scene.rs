use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    component::Component,
    entity::Entity,
    error::{ComponentError, SceneError, WXRError},
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

    pub fn load_plugin(&mut self, path: String) -> Result<(), WXRError> {
        if self.plugins.contains_key(&path) {
            Err(WXRError::PluginAlreadyLoaded)
        } else {
            let plugin = Plugin::new(path.clone())?;
            self.plugins.insert(path, plugin);
            Ok(())
        }
    }

    pub fn unload_plugin(&mut self, path: &str) -> Result<(), WXRError> {
        if let Some(plugin) = self.plugins.remove(path) {
            plugin.destroy();
            Ok(())
        } else {
            Err(WXRError::PluginNotFound)
        }
    }

    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), WXRError> {
        if self.systems.contains_key(&id) {
            Err(WXRError::SystemAlreadyLoaded)
        } else {
            let mut system: Option<System> = None;
            for (_, plugin) in self.plugins.iter() {
                if let Ok(current_system) = System::new(id.clone(), plugin, priority) {
                    system = Some(current_system);
                    break;
                }
                // TODO: Add else case with better errors
            }
            if let Some(system) = system {
                system.get_functions().attach(self);
                self.systems.insert(id, system);
                Ok(())
            } else {
                Err(WXRError::Other)
            }
        }
    }

    pub fn remove_system(&mut self, id: &str) -> Result<(), WXRError> {
        if let Some(system) = self.systems.remove(id) {
            system.get_functions().detach(self);
            Ok(())
        } else {
            Err(WXRError::SystemNotFound)
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

    pub fn remove_entity(&mut self, uuid: Uuid) -> Result<(), WXRError> {
        if let Some(entity) = self.entities.remove(&uuid) {
            // TODO: Remove all the components associated
            Ok(())
        } else {
            Err(WXRError::Other)
        }
    }

    pub fn add_component(
        &mut self,
        uuid: Uuid,
        component_id: &str,
    ) -> Result<(), SceneError<ComponentError>> {
        let Some(entity) = self.entities.get_mut(&uuid) else {
            return Err(SceneError::EntityNotFound);
        };
        if entity.component_exists(component_id) {
            return Err(SceneError::ComponentAlreadyExists);
        }

        let mut component: Option<Component> = None;
        for (_, plugin) in self.plugins.iter() {
            if let Ok(current_component) = Component::new(component_id.to_owned(), plugin) {
                component = Some(current_component);
                break;
            }
            // TODO: Provide better errors
        }

        let Some(component) = component else {
            return Err(SceneError::Other(ComponentError::Other));
        };

        let res = entity.add_component(component);
        assert!(res.is_ok());

        Ok(())
    }

    pub fn remove_component(&mut self, uuid: Uuid, component_id: &str) {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            entity.remove_component(component_id);
        }
    }

    pub fn get<T>(&self, uuid: Uuid, component_id: &str, field_id: &str) -> Option<T> {
        if let Some(entity) = self.entities.get(&uuid) {
            if let Some(component) = entity.get_component(component_id) {
                if let Ok(data) = component.get(field_id) {
                    Some(data)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set<T>(&mut self, uuid: Uuid, component_id: &str, field_id: &str, data: &T) -> bool {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            if let Some(component) = entity.get_component_mut(component_id) {
                if let Ok(_) = component.set(field_id, data) {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}
