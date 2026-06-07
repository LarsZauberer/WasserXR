use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    entity::Entity,
    error::WXRError,
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
}
