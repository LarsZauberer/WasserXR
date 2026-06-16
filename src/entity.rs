use log::warn;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{component::Component, error::EntityError};

pub struct Entity {
    id: Uuid,
    name: String,
    components: HashMap<String, Component>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "".to_owned(),
            components: HashMap::new(),
        }
    }

    pub fn new_with_name(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            components: HashMap::new(),
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_components(&mut self) -> Vec<&String> {
        self.components.keys().collect()
    }

    pub fn component_exists(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }

    pub fn add_component(&mut self, component: Component) -> Result<&Component, EntityError> {
        let component_id = component.get_id().to_owned();
        if self.components.contains_key(&component_id) {
            Err(EntityError::ComponentAlreadyExists)
        } else {
            self.components.insert(component_id.clone(), component);
            Ok(self
                .components
                .get(&component_id)
                .expect("component was just inserted"))
        }
    }

    pub fn remove_component(&mut self, component_id: &str) -> Result<Component, EntityError> {
        self.components
            .remove(component_id)
            .ok_or(EntityError::ComponentNotFound)
    }

    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut Component> {
        self.components.get_mut(id)
    }
}

impl Drop for Entity {
    fn drop(&mut self) {
        // Ensure that all owned components have been removed from the entity
        if !self.components.is_empty() {
            warn!("Entity dropped which still had some components attached to it. (This is a bug)");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_void, rc::Rc};

    use crate::plugin::Plugin;

    use super::*;

    // -- Test FFI symbols ------------------------------------------------

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_position() -> *mut c_void {
        Box::into_raw(Box::new([0.0f32; 3])) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_position(ptr: *mut c_void) {
        let _ = unsafe { Box::from_raw(ptr as *mut [f32; 3]) };
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_name() -> *mut c_void {
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_name(ptr: *mut c_void) {
        assert!(ptr.is_null());
    }

    fn make_component(id: &str) -> Component {
        let plugin = Rc::new(Plugin::new_static());
        Component::new(id.to_owned(), plugin).unwrap()
    }

    // -- add_component ---------------------------------------------------

    #[test]
    fn test_add_component_stores_owned_component() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity
            .add_component(component)
            .expect("add_component should succeed");

        assert!(entity.component_exists("position"));
        assert_eq!(
            entity.get_component("position").unwrap().get_id(),
            "position"
        );
    }

    #[test]
    fn test_add_component_multiple_owned_components() {
        let mut entity = Entity::new();
        let pos = make_component("position");
        let name = make_component("name");

        entity.add_component(pos).expect("add_component position");
        entity.add_component(name).expect("add_component name");

        assert!(entity.component_exists("position"));
        assert!(entity.component_exists("name"));
        assert_eq!(entity.get_components().len(), 2);

        entity.remove_component("position").unwrap();
        entity.remove_component("name").unwrap();
    }

    #[test]
    fn test_add_component_duplicate_returns_error() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity
            .add_component(component)
            .expect("first add should succeed");

        let duplicate = make_component("position");
        let result = entity.add_component(duplicate);
        assert!(matches!(result, Err(EntityError::ComponentAlreadyExists)));
    }

    // -- remove_component ------------------------------------------------

    #[test]
    fn test_remove_component_removes_by_component_id() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity.add_component(component).unwrap();
        assert!(entity.component_exists("position"));

        // Remove by passing the component id, transferring ownership out of the entity
        entity
            .remove_component("position")
            .expect("remove_component should succeed");
        assert!(!entity.component_exists("position"));
    }

    #[test]
    fn test_remove_component_nonexistent_returns_error() {
        let mut entity = Entity::new();
        let component = make_component("position");

        // Component was never added
        let result = entity.remove_component(component.get_id());
        assert!(matches!(result, Err(EntityError::ComponentNotFound)));
    }

    // -- get_component & get_component_mut --------------------------------

    #[test]
    fn test_get_component_returns_stored_component() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity.add_component(component).unwrap();

        let retrieved = entity.get_component("position");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get_id(), "position");
    }

    #[test]
    fn test_get_component_nonexistent_returns_none() {
        let entity = Entity::new();
        assert!(entity.get_component("nonexistent").is_none());
    }
}
