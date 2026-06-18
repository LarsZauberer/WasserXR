pub mod component;
pub(crate) mod entity;
pub(crate) mod plugin;
pub(crate) mod system;

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
        let mut systems_sorted: Vec<&System> = self.systems.values().collect();
        systems_sorted.sort_by_key(|a| a.get_priority());

        let functions: Vec<(usize, Selector, Runner)> = systems_sorted
            .iter()
            .rev()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::component::{FieldType, Schema};
    use std::{
        ffi::c_void,
        sync::{
            LazyLock, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    #[repr(C)]
    struct SceneCounter {
        value: i64,
    }

    unsafe extern "C" fn scene_counter_getter(data: *const c_void) -> *const c_void {
        unsafe { &(*(data as *const SceneCounter)).value as *const i64 as *const c_void }
    }

    unsafe extern "C" fn scene_counter_setter(data: *mut c_void, value: *const c_void) {
        unsafe {
            (*(data as *mut SceneCounter)).value = *(value as *const i64);
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
    unsafe extern "C" fn wxr_schema_scene_counter(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::Long,
                Some(scene_counter_getter),
                Some(scene_counter_setter),
            );
        }
    }

    static SCENE_ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SCENE_TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

    unsafe extern "C" fn scene_select_all(_scene: *const Scene, _entity: *const u8) -> i32 {
        0
    }

    unsafe extern "C" fn scene_noop_attacher_detacher(_scene: *mut Scene) {}

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
    fn scene_add_entity_creates_entity_with_empty_name() {
        let mut scene = Scene::new();

        let entity = scene.add_entity();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "");
    }

    #[test]
    fn scene_set_entity_name_for_existing_entity_updates_name() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        scene.set_entity_name(entity, "Player".to_owned()).unwrap();

        assert_eq!(scene.get_entity_name(entity).unwrap(), "Player");
    }

    #[test]
    fn scene_remove_entity_for_existing_entity_removes_entity() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        scene.remove_entity(entity).unwrap();

        assert_eq!(
            scene.get_entity_name(entity),
            Err(SceneError::EntityNotFound)
        );
    }

    #[test]
    fn scene_add_component_for_existing_entity_adds_component() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        assert_eq!(
            scene.add_component(entity, "scene_counter".to_owned()),
            Ok(())
        );
    }

    #[test]
    fn scene_set_component_field_for_existing_component_updates_value() {
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
    fn scene_add_component_for_duplicate_component_returns_component_already_exists() {
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
    fn scene_remove_component_for_existing_component_removes_component() {
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
    fn scene_add_system_for_existing_symbol_calls_attacher() {
        SCENE_ATTACH_COUNT.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();

        scene
            .add_system("scene_attach_system".to_owned(), 1)
            .unwrap();

        assert_eq!(SCENE_ATTACH_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scene_remove_system_for_existing_system_calls_detacher() {
        SCENE_DETACH_COUNT.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();
        scene.systems.insert(
            "scene_attach_system".to_owned(),
            System::new_test(
                "scene_attach_system".to_owned(),
                1,
                0,
                scene_select_all,
                wxr_system_scene_attach_system,
                scene_noop_attacher_detacher,
                wxr_detach_scene_attach_system,
            ),
        );

        scene.remove_system("scene_attach_system").unwrap();

        assert_eq!(SCENE_DETACH_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scene_reset_with_runtime_state_clears_entities_components_and_systems() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        scene.systems.insert(
            "scene_cleanup_system".to_owned(),
            System::new_test(
                "scene_cleanup_system".to_owned(),
                1,
                0,
                scene_select_all,
                wxr_system_scene_cleanup_system,
                scene_noop_attacher_detacher,
                scene_noop_attacher_detacher,
            ),
        );

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
    fn scene_unload_plugin_for_static_plugin_removes_owned_systems_and_components() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();
        scene
            .add_component(entity, "scene_counter".to_owned())
            .unwrap();
        scene.systems.insert(
            "scene_cleanup_system".to_owned(),
            System::new_test(
                "scene_cleanup_system".to_owned(),
                1,
                0,
                scene_select_all,
                wxr_system_scene_cleanup_system,
                scene_noop_attacher_detacher,
                scene_noop_attacher_detacher,
            ),
        );

        scene.unload_plugin("").unwrap();

        assert_eq!(
            scene.get::<i64>(entity, "scene_counter", "value"),
            Err(SceneError::ComponentNotFound)
        );
        assert_eq!(
            scene.remove_system("scene_cleanup_system"),
            Err(SceneError::SystemNotFound)
        );
    }

    #[test]
    fn scene_tick_with_multiple_systems_runs_higher_priority_first() {
        SCENE_TICK_ORDER.lock().unwrap().clear();
        let mut scene = Scene::new();
        scene.add_entity();
        scene.systems.insert(
            "scene_high_priority".to_owned(),
            System::new_test(
                "scene_high_priority".to_owned(),
                2,
                1,
                scene_select_all,
                wxr_system_scene_high_priority,
                scene_noop_attacher_detacher,
                scene_noop_attacher_detacher,
            ),
        );
        scene.systems.insert(
            "scene_low_priority".to_owned(),
            System::new_test(
                "scene_low_priority".to_owned(),
                1,
                1,
                scene_select_all,
                wxr_system_scene_low_priority,
                scene_noop_attacher_detacher,
                scene_noop_attacher_detacher,
            ),
        );

        scene.tick();

        assert_eq!(*SCENE_TICK_ORDER.lock().unwrap(), vec!["high", "low"]);
    }
}
