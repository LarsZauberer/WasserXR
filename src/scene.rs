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
        if let Some(plugin) = plugin::Plugin::new(path) {
            self.plugins.push(plugin);
            true
        } else {
            false
        }
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

    pub fn tick(&mut self) -> bool {
        // Get all the systems from all the plugins and sort them in the priority order (highest
        // priority first)
        let mut plugins = std::mem::take(&mut self.plugins);
        let mut systems: Vec<_> = plugins
            .iter_mut()
            .flat_map(|plugin| plugin.get_systems_mut())
            .collect();

        systems.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for system in systems.into_iter().rev() {
            system.run(self);
        }

        self.plugins = plugins;
        true
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TICK_TEST_SYSTEM_RUNS: AtomicUsize = AtomicUsize::new(0);

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_tick_test_system(
        _scene: *mut Scene,
        _entities: *const *const *mut Entity,
        _groups: *const usize,
    ) {
        TICK_TEST_SYSTEM_RUNS.fetch_add(1, Ordering::SeqCst);
    }

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

    #[test]
    fn tick_runs_system() {
        TICK_TEST_SYSTEM_RUNS.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();

        assert!(scene.add_system("tick_test_system", 100));
        assert!(scene.tick());

        assert_eq!(TICK_TEST_SYSTEM_RUNS.load(Ordering::SeqCst), 1);
    }
}
