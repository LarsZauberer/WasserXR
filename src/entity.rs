use std::collections::HashMap;
use std::rc::Rc;

use log::warn;
use uuid::Uuid;

use crate::{component::Component, error::EntityError};

pub struct Entity {
    id: Uuid,
    name: String,
    components: HashMap<String, Rc<Component>>,
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

    pub fn add_component(&mut self, component: Rc<Component>) -> Result<(), EntityError> {
        if self.components.contains_key(component.get_id()) {
            Err(EntityError::ComponentAlreadyExists)
        } else {
            self.components
                .insert(component.get_id().to_owned(), component);
            Ok(())
        }
    }

    pub fn remove_component(&mut self, component: &Component) -> Result<(), EntityError> {
        if self.components.remove(component.get_id()).is_some() {
            Ok(())
        } else {
            Err(EntityError::ComponentNotFound)
        }
    }

    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id).map(|rc| rc.as_ref())
    }

    pub fn get_component_mut(&mut self, id: &str) -> Option<&mut Component> {
        self.components.get_mut(id).and_then(|rc| Rc::get_mut(rc))
    }

    /// Get a reference-counted handle to a stored component, cloning the `Rc`.
    ///
    /// Use this when you need to hold a reference independently of the entity's
    /// borrow — for example, to pass a `&Component` reference to
    /// [`remove_component`] without conflicting with a mutable borrow.
    pub fn get_component_rc(&self, id: &str) -> Option<Rc<Component>> {
        self.components.get(id).cloned()
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

    fn make_component(id: &str) -> Rc<Component> {
        let plugin = Rc::new(Plugin::new_static());
        Rc::new(Component::new(id.to_owned(), plugin).unwrap())
    }

    // -- add_component ---------------------------------------------------

    #[test]
    fn test_add_component_rc_stores_component() {
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

        // Remove before drop to satisfy Entity::drop invariant
        let rc = entity.get_component_rc("position").unwrap();
        entity.remove_component(&rc).unwrap();
    }

    #[test]
    fn test_add_component_rc_multiple_components() {
        let mut entity = Entity::new();
        let pos = make_component("position");
        let name = make_component("name");

        entity.add_component(pos).expect("add_component position");
        entity.add_component(name).expect("add_component name");

        assert!(entity.component_exists("position"));
        assert!(entity.component_exists("name"));
        assert_eq!(entity.get_components().len(), 2);

        // Remove before drop to satisfy Entity::drop invariant
        let rc1 = entity.get_component_rc("position").unwrap();
        entity.remove_component(&rc1).unwrap();
        let rc2 = entity.get_component_rc("name").unwrap();
        entity.remove_component(&rc2).unwrap();
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
        assert_eq!(result, Err(EntityError::ComponentAlreadyExists));

        // Remove before drop to satisfy Entity::drop invariant
        let rc = entity.get_component_rc("position").unwrap();
        entity.remove_component(&rc).unwrap();
    }

    // -- remove_component ------------------------------------------------

    #[test]
    fn test_remove_component_removes_by_component_ref() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity.add_component(component.clone()).unwrap();
        assert!(entity.component_exists("position"));

        // Remove by passing a reference to the component
        entity
            .remove_component(&component)
            .expect("remove_component should succeed");
        assert!(!entity.component_exists("position"));
    }

    #[test]
    fn test_remove_component_nonexistent_returns_error() {
        let mut entity = Entity::new();
        let component = make_component("position");

        // Component was never added
        let result = entity.remove_component(&component);
        assert_eq!(result, Err(EntityError::ComponentNotFound));
    }

    // -- get_component & get_component_mut --------------------------------

    #[test]
    fn test_get_component_returns_stored_component() {
        let mut entity = Entity::new();
        let component = make_component("position");

        entity.add_component(component.clone()).unwrap();

        let retrieved = entity.get_component("position");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get_id(), "position");

        // Remove before drop to satisfy Entity::drop invariant
        let rc = entity.get_component_rc("position").unwrap();
        entity.remove_component(&rc).unwrap();
    }

    #[test]
    fn test_get_component_nonexistent_returns_none() {
        let entity = Entity::new();
        assert!(entity.get_component("nonexistent").is_none());
    }
}
