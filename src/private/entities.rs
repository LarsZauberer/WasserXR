use std::collections::HashMap;

use crate::{errors::EntityError, private::components::Component};

/// The entity struct corresponds to the actual entity data. It stores the
/// components it is carrying.
#[derive(Debug, Default)]
pub(crate) struct Entity {
    components: HashMap<String, Component>,
}

impl Entity {
    /// Create a new entity
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add a new component to the entity. The function will reject the add, if
    /// a component of that type already exists.
    pub(crate) fn add_component(&mut self, component: Component) -> Result<(), EntityError> {
        let id = component.name().to_owned();
        if self.components.contains_key(&id) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        self.components.insert(id, component);
        Ok(())
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&mut self, id: &str) -> Result<(), EntityError> {
        self.components
            .remove(id)
            .map(|_| ())
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Return a reference to the component
    pub(crate) fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    /// Get all currently attached component id's from this entity
    pub(crate) fn get_components(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
}
