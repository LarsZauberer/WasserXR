use slotmap::{SlotMap, new_key_type};

use crate::{components::Component, errors::EntityError};

new_key_type! {
    /// The component id is a cheap copyable handle for a component. It is unique inside of an
    /// entity. It is **not unique** from entity to entity.
    pub struct ComponentID;
}

/// The entity struct corresponds to the actual entity data. It stores the
/// components it is carrying.
#[derive(Debug, Default)]
pub struct Entity {
    components: SlotMap<ComponentID, Component>,
}

impl Entity {
    /// Create a new entity
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new component to the entity
    pub fn add_component(&mut self, component: Component) -> ComponentID {
        todo!()
    }

    /// Remove a component from the entity
    pub fn remove_component(&mut self, id: ComponentID) -> Result<(), EntityError> {
        todo!()
    }

    /// Gets the actual component object from the component id
    pub fn get_component(&self, id: ComponentID) -> Option<&Component> {
        todo!()
    }

    /// Get a component id of the component with a certain type
    pub fn get_component_type(&self, component_type: &str) -> Option<ComponentID> {
        todo!()
    }

    /// Get all currently attached component id's from this entity
    pub fn get_components(&self) -> Vec<ComponentID> {
        todo!()
    }
}
