use crate::scene::plugin::Plugin;

mod plugin;
mod system;

pub type Entity = usize;

pub struct Scene {
    entity_counter: Entity,
    entities: Vec<Entity>,
    plugins: Vec<Plugin>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entities: Vec::new(),
            plugins: vec![Plugin::new_static()],
        }
    }

    pub fn add_entity(&mut self) -> Entity {
        let entity = self.entity_counter;
        self.entity_counter += 1;
        self.entities.push(entity);
        entity
    }

    pub fn get_entities_count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn load_plugin(&mut self, path: &str) -> bool {
        let plugin = plugin::Plugin::new(path);
        if plugin.is_none() {
            return false;
        }
        self.plugins.push(plugin.unwrap());
        true
    }

    pub fn add_system(&mut self, id: &str, priority: usize) -> bool {
        // We iterate in reverse order because we want to have the newest plugins that have been loaded
        // be prioritized and especially the static linked plugin be used at last
        let mut plugins = std::mem::take(&mut self.plugins);
        let added = plugins
            .iter_mut()
            .rev()
            .any(|plugin| plugin.add_system(self, id, priority));

        self.plugins = plugins;
        added
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_entities() {
        let mut scene = Scene::new();

        let first = scene.add_entity();
        let second = scene.add_entity();

        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(scene.get_entities_count(), 2);
        assert_eq!(scene.get_entities(), &[first, second]);
    }
}
