use std::collections::{HashMap, hash_map::Entry};
use std::rc::Rc;

use log::error;
use uuid::Uuid;

use crate::{
    component::Component,
    entity::Entity,
    error::{ComponentError, PluginError, SceneError},
    plugin::Plugin,
    system::{System, SystemFunctions},
};

pub struct Scene {
    entities: HashMap<Uuid, Entity>,
    plugins: HashMap<String, Rc<Plugin>>,
    systems: HashMap<String, System>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut plugins = HashMap::new();
        plugins.insert("".to_owned(), Rc::new(Plugin::new_static()));
        Self {
            entities: HashMap::new(),
            plugins,
            systems: HashMap::new(),
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_plugin(&mut self, path: String) -> Result<(), SceneError> {
        match self.plugins.entry(path) {
            Entry::Occupied(_) => Err(SceneError::PluginAlreadyLoaded),
            Entry::Vacant(entry) => {
                let plugin = Rc::new(Plugin::new(entry.key().clone()).map_err(
                    |error| match error {
                        PluginError::LinkingError(msg) => {
                            log::error!("Linking Error: {}", msg);
                            SceneError::PluginLoading(PluginError::LinkingError(msg))
                        }
                        _ => SceneError::PluginLoading(error),
                    },
                )?);
                entry.insert(plugin);
                Ok(())
            }
        }
    }

    pub fn unload_plugin(&mut self, path: &str) -> Result<(), SceneError> {
        let Some(plugin) = self.plugins.remove(path) else {
            return Err(SceneError::PluginNotFound);
        };

        // Check if no references are still alive
        let Ok(_) = Rc::try_unwrap(plugin) else {
            error!(
                "There are still reference counted references to the plugin `{}` that should be unloaded.",
                path
            );
            error!(
                "There are dead references to the plugin that has been unloaded. Corrupted ECS state. Terminating Program!"
            );
            panic!(
                "There are dead references to the plugin that has been unloaded. Corrupted ECS state"
            );
        };

        Ok(())
    }

    pub fn add_system(&mut self, id: String, priority: usize) -> Result<(), SceneError> {
        if self.systems.contains_key(&id) {
            Err(SceneError::SystemAlreadyExists)
        } else {
            let Some(system) = self
                .plugins
                .values()
                .find_map(|plugin| System::new(id.clone(), plugin.clone(), priority).ok())
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
            .map(|system| system.get_functions())
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
        if self.entities.remove(&uuid).is_some() {
            // TODO: Remove all the components associated
            Ok(())
        } else {
            Err(SceneError::EntityNotFound)
        }
    }

    pub fn get_entities(&self) -> Vec<&Uuid> {
        self.entities.keys().collect()
    }

    pub fn add_component(
        &mut self,
        uuid: Uuid,
        component_id: &str,
    ) -> Result<Rc<Component>, SceneError> {
        let Some(entity) = self.entities.get_mut(&uuid) else {
            return Err(SceneError::EntityNotFound);
        };
        if entity.component_exists(component_id) {
            return Err(SceneError::ComponentAlreadyExists);
        }

        let mut last_error = ComponentError::NoCreator;
        let component = self
            .plugins
            .values()
            .find_map(
                |plugin| match Component::new(component_id.to_owned(), plugin.clone()) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        last_error = e;
                        None
                    }
                },
            )
            .ok_or(SceneError::ComponentCreation(last_error))?;

        let component_rc = Rc::new(component);
        let res = entity.add_component(component_rc.clone());
        assert!(res.is_ok());

        Ok(component_rc)
    }

    pub fn remove_component(&mut self, uuid: Uuid, component_id: &str) -> Result<(), SceneError> {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            let component_rc = entity
                .get_component_rc(component_id)
                .ok_or(SceneError::ComponentNotFound)?;
            entity
                .remove_component(&component_rc)
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
            .map_err(SceneError::ComponentFieldError)
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
            .map_err(SceneError::ComponentFieldError)
    }

    /// Get a reference to an entity by UUID.
    pub fn get_entity(&self, uuid: Uuid) -> Option<&Entity> {
        self.entities.get(&uuid)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{ComponentError, SceneError},
        scene::Scene,
    };

    fn scene_with_entity() -> (Scene, uuid::Uuid) {
        let mut scene = Scene::new();
        let uuid = scene.add_entity(None);
        (scene, uuid)
    }

    // -- add_component ---------------------------------------------------

    #[test]
    fn test_add_component_returns_component_rc() {
        let (mut scene, uuid) = scene_with_entity();

        let component = scene
            .add_component(uuid, "position")
            .expect("add_component should succeed");

        assert_eq!(component.get_id(), "position");
    }

    #[test]
    fn test_add_component_invalid_type_returns_descriptive_error() {
        let (mut scene, uuid) = scene_with_entity();

        let result = scene.add_component(uuid, "nonexistent_component");

        match result {
            Err(SceneError::ComponentCreation(err)) => match err {
                ComponentError::MissingSymbol(sym) => {
                    assert!(sym.contains("nonexistent_component"))
                }
                _ => panic!("expected MissingSymbol, got {:?}", err),
            },
            other => panic!(
                "expected Err(ComponentCreation(MissingSymbol)), got {:?}",
                other.map(|_| ())
            ),
        }
    }

    #[test]
    fn test_add_component_duplicate_returns_error() {
        let (mut scene, uuid) = scene_with_entity();

        scene
            .add_component(uuid, "position")
            .expect("first add_component should succeed");

        let result = scene.add_component(uuid, "position");
        assert!(matches!(result, Err(SceneError::ComponentAlreadyExists)));
    }

    #[test]
    fn test_add_component_nonexistent_entity_returns_error() {
        let mut scene = Scene::new();
        let bogus_uuid = uuid::Uuid::now_v7();

        let result = scene.add_component(bogus_uuid, "position");
        assert!(matches!(result, Err(SceneError::EntityNotFound)));
    }

    #[test]
    fn test_add_component_multiple_different_types() {
        let (mut scene, uuid) = scene_with_entity();

        let pos = scene.add_component(uuid, "position").expect("add position");
        assert_eq!(pos.get_id(), "position");

        let name = scene.add_component(uuid, "name").expect("add name");
        assert_eq!(name.get_id(), "name");

        // Both components are reachable via scene.remove
        assert!(scene.remove_component(uuid, "position").is_ok());
        assert!(scene.remove_component(uuid, "name").is_ok());
    }

    #[test]
    fn test_add_component_rc_lifetime_independent_of_scene() {
        let (mut scene, uuid) = scene_with_entity();

        let component = scene
            .add_component(uuid, "position")
            .expect("add_component should succeed");

        // Hold an Rc after removing the component from the scene
        scene
            .remove_component(uuid, "position")
            .expect("remove should succeed");

        // The component still exists because we hold the Rc
        assert_eq!(component.get_id(), "position");
    }

    // -- remove_component ---------------------------------------------------

    #[test]
    fn test_remove_component_removes_component() {
        let (mut scene, uuid) = scene_with_entity();

        scene
            .add_component(uuid, "position")
            .expect("add_component should succeed");

        assert!(scene.remove_component(uuid, "position").is_ok());
    }

    #[test]
    fn test_remove_component_nonexistent_returns_error() {
        let (mut scene, uuid) = scene_with_entity();

        let result = scene.remove_component(uuid, "position");
        assert_eq!(result, Err(SceneError::ComponentNotFound));
    }

    #[test]
    fn test_remove_component_nonexistent_entity_returns_error() {
        let mut scene = Scene::new();
        let bogus_uuid = uuid::Uuid::now_v7();

        let result = scene.remove_component(bogus_uuid, "position");
        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn test_remove_component_then_re_add() {
        let (mut scene, uuid) = scene_with_entity();

        scene.add_component(uuid, "position").expect("first add");
        scene.remove_component(uuid, "position").expect("remove");

        let component = scene
            .add_component(uuid, "position")
            .expect("re-add after removal should succeed");
        assert_eq!(component.get_id(), "position");
    }

    // -- get() ----------------------------------------------------------------

    #[test]
    fn test_get_field_value_from_component() {
        let (mut scene, uuid) = scene_with_entity();

        // position component has [f32; 3] data, no schema — well-defined get
        scene.add_component(uuid, "position").expect("add position");

        // position has no schema — get any field returns FieldNotFound
        let result: Result<*const std::ffi::c_void, SceneError> = scene.get(uuid, "position", "x");
        assert!(matches!(
            result,
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotFound
            ))
        ));
    }

    #[test]
    fn test_get_nonexistent_entity_returns_not_found() {
        let scene = Scene::new();
        let bogus_uuid = uuid::Uuid::now_v7();
        let result: Result<*const std::ffi::c_void, SceneError> =
            scene.get(bogus_uuid, "position", "x");
        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn test_get_nonexistent_component_returns_not_found() {
        let (scene, uuid) = scene_with_entity();
        let result: Result<*const std::ffi::c_void, SceneError> =
            scene.get(uuid, "nonexistent", "x");
        assert_eq!(result, Err(SceneError::ComponentNotFound));
    }

    #[test]
    fn test_get_nonexistent_field_returns_field_error() {
        let (mut scene, uuid) = scene_with_entity();
        scene.add_component(uuid, "position").expect("add position");

        let result: Result<*const std::ffi::c_void, SceneError> =
            scene.get(uuid, "position", "nonexistent_field");
        assert!(matches!(
            result,
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotFound
            ))
        ));
    }

    // -- set() ----------------------------------------------------------------

    #[test]
    fn test_set_nonexistent_entity_returns_not_found() {
        let mut scene = Scene::new();
        let val: f32 = 1.0;
        let result = scene.set(uuid::Uuid::now_v7(), "position", "x", &val);
        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn test_set_nonexistent_component_returns_not_found() {
        let (mut scene, uuid) = scene_with_entity();
        let val: f32 = 1.0;
        let result = scene.set(uuid, "nonexistent", "x", &val);
        assert_eq!(result, Err(SceneError::ComponentNotFound));
    }

    #[test]
    fn test_set_nonexistent_field_returns_field_error() {
        let (mut scene, uuid) = scene_with_entity();
        scene.add_component(uuid, "position").expect("add position");

        let val: f32 = 1.0;
        let result = scene.set(uuid, "position", "nonexistent_field", &val);
        assert!(matches!(
            result,
            Err(SceneError::ComponentFieldError(
                ComponentError::FieldNotFound
            ))
        ));
    }

    // -- get_entity -----------------------------------------------------------

    #[test]
    fn test_get_entity_returns_existing_entity() {
        let (scene, uuid) = scene_with_entity();

        let entity = scene.get_entity(uuid);
        assert!(entity.is_some());
    }

    #[test]
    fn test_get_entity_nonexistent_returns_none() {
        let scene = Scene::new();
        let entity = scene.get_entity(uuid::Uuid::now_v7());
        assert!(entity.is_none());
    }

    // -- new() / default ------------------------------------------------------

    #[test]
    fn test_new_scene_has_expected_default_state() {
        let scene = Scene::new();
        assert!(scene.get_entities().is_empty());
        assert!(scene.get_entity(uuid::Uuid::now_v7()).is_none());
    }

    // -- add_entity  ----------------------------------------------------------

    #[test]
    fn test_add_entity_with_name_stores_named_entity() {
        let mut scene = Scene::new();
        let uuid = scene.add_entity(Some("player".to_owned()));

        let entity = scene.get_entity(uuid).expect("entity should exist");
        assert_eq!(entity.get_name(), "player");
    }

    #[test]
    fn test_add_entity_anonymous_returns_different_uuids() {
        let mut scene = Scene::new();
        let u1 = scene.add_entity(None);
        let u2 = scene.add_entity(None);
        assert_ne!(u1, u2);
        assert_eq!(scene.get_entities().len(), 2);
    }

    // -- get_entities ---------------------------------------------------------

    #[test]
    fn test_get_entities_empty_on_new_scene() {
        let scene = Scene::new();
        assert!(scene.get_entities().is_empty());
    }

    #[test]
    fn test_get_entities_contains_all_added() {
        let mut scene = Scene::new();
        let ids: Vec<uuid::Uuid> = (0..3).map(|_| scene.add_entity(None)).collect();

        let entities = scene.get_entities();
        for id in &ids {
            assert!(entities.contains(&id), "entity {id} should be present");
        }
        assert_eq!(entities.len(), 3);
    }

    // -- remove_entity --------------------------------------------------------

    #[test]
    fn test_remove_entity_removes_existing() {
        let mut scene = Scene::new();
        let uuid = scene.add_entity(None);
        assert_eq!(scene.get_entities().len(), 1);

        scene.remove_entity(uuid).expect("remove should succeed");
        assert!(scene.get_entities().is_empty());
    }

    #[test]
    fn test_remove_entity_nonexistent_returns_error() {
        let mut scene = Scene::new();
        let result = scene.remove_entity(uuid::Uuid::nil());
        assert_eq!(result, Err(SceneError::EntityNotFound));
    }

    #[test]
    fn test_remove_entity_then_re_add_allows_new_entity() {
        let mut scene = Scene::new();
        let uuid = scene.add_entity(None);
        scene.remove_entity(uuid).unwrap();
        assert!(scene.get_entities().is_empty());

        // Re-adding an entity should work (different UUID)
        let uuid2 = scene.add_entity(None);
        assert_ne!(uuid, uuid2);
        assert_eq!(scene.get_entities().len(), 1);
    }

    // -- load_plugin ----------------------------------------------------------

    #[test]
    fn test_load_plugin_already_loaded_returns_error() {
        let mut scene = Scene::new();
        // Default scene has "" plugin loaded
        let result = scene.load_plugin("".to_owned());
        assert_eq!(result, Err(SceneError::PluginAlreadyLoaded));
    }

    #[test]
    fn test_load_plugin_nonexistent_path_returns_linking_error() {
        let mut scene = Scene::new();
        let result = scene.load_plugin("/nonexistent/plugin.so".to_owned());
        assert!(matches!(result, Err(SceneError::PluginLoading(_))));
    }

    // -- unload_plugin --------------------------------------------------------

    #[test]
    fn test_unload_plugin_removes_existing_plugin() {
        let mut scene = Scene::new();
        // Default "" plugin is loaded
        scene.unload_plugin("").expect("unload should succeed");
        // Verify the plugin is removed (no-op on get_symbol — won't crash)
        assert!(scene.plugins.is_empty());
    }

    #[test]
    fn test_unload_plugin_nonexistent_returns_not_found() {
        let mut scene = Scene::new();
        let result = scene.unload_plugin("nonexistent");
        assert_eq!(result, Err(SceneError::PluginNotFound));
    }

    // -- add_system -----------------------------------------------------------

    #[test]
    fn test_add_system_adds_to_scene() {
        let mut scene = Scene::new();
        scene
            .add_system("with_runner".to_owned(), 50)
            .expect("add_system should succeed");
        assert!(scene.systems.contains_key("with_runner"));
    }

    #[test]
    fn test_add_system_nonexistent_id_returns_creation_error() {
        let mut scene = Scene::new();
        let result = scene.add_system("nonexistent_system".to_owned(), 100);
        assert_eq!(result, Err(SceneError::SystemCreation));
    }

    #[test]
    fn test_add_system_duplicate_id_returns_already_exists() {
        let mut scene = Scene::new();
        // "with_runner" is defined in system.rs tests as a no_mangle symbol
        scene
            .add_system("with_runner".to_owned(), 50)
            .expect("first add should succeed");

        let result = scene.add_system("with_runner".to_owned(), 100);
        assert_eq!(result, Err(SceneError::SystemAlreadyExists));
    }

    // -- remove_system --------------------------------------------------------

    #[test]
    fn test_remove_system_removes_added_system() {
        let mut scene = Scene::new();
        scene
            .add_system("with_runner".to_owned(), 50)
            .expect("add_system");

        scene
            .remove_system("with_runner")
            .expect("remove_system should succeed");
        assert!(!scene.systems.contains_key("with_runner"));
    }

    #[test]
    fn test_remove_system_nonexistent_returns_not_found() {
        let mut scene = Scene::new();
        let result = scene.remove_system("nonexistent");
        assert_eq!(result, Err(SceneError::SystemNotFound));
    }

    // -- tick() ---------------------------------------------------------------

    #[test]
    fn test_tick_empty_scene_no_panic() {
        let mut scene = Scene::new();
        scene.tick();
        // Should not crash
    }

    #[test]
    fn test_tick_with_system_no_panic() {
        let mut scene = Scene::new();
        scene
            .add_system("with_runner".to_owned(), 50)
            .expect("add_system");
        scene.tick();
        // Should not crash — the FFI system call happens successfully
    }

    // -- error consistency ----------------------------------------------------

    #[test]
    fn test_component_already_exists_error_matches_entity_error() {
        let (mut scene, uuid) = scene_with_entity();

        scene.add_component(uuid, "position").unwrap();
        let result = scene.add_component(uuid, "position");

        assert!(matches!(result, Err(SceneError::ComponentAlreadyExists)));
    }
}
