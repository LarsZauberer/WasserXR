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
            name: name,
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

    pub fn add_component(&mut self, component: Component) -> Result<(), EntityError> {
        if self.components.contains_key(component.get_id()) {
            Err(EntityError::ComponentAlreadyExists)
        } else {
            self.components
                .insert(component.get_id().to_owned(), component);
            Ok(())
        }
    }

    pub fn remove_component(&mut self, id: &str) -> Result<(), EntityError> {
        if self.components.remove(id).is_some() {
            Ok(())
        } else {
            Err(EntityError::ComponentNotFound)
        }
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
        // Ensure that all the components have been removed from the entity
        assert!(self.components.len() == 0);
    }
}
