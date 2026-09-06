use std::os::raw::c_void;

use slotmap::SlotMap;

use crate::{errors::EntityError, private::components::Component, scene::ComponentID};

/// The entity struct corresponds to the actual entity data. It stores the
/// components it is carrying.
#[derive(Debug, Default)]
pub(crate) struct Entity {
    components: SlotMap<ComponentID, Component>,
}

impl Entity {
    /// Create a new entity
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add a new component to the entity. The function will reject the add, if
    /// a component of that type already exists.
    pub(crate) fn add_component(
        &mut self,
        component: Component,
    ) -> Result<ComponentID, EntityError> {
        if self
            .components
            .values()
            .find(|c| c.get_name() == component.get_name())
            .is_some()
        {
            return Err(EntityError::ComponentAlreadyExists);
        }
        Ok(self.components.insert(component))
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&mut self, id: ComponentID) -> Result<Component, EntityError> {
        self.components
            .remove(id)
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Get all currently attached component id's from this entity
    pub(crate) fn get_components(&self) -> Vec<ComponentID> {
        self.components.keys().collect()
    }

    /// Resolve from component name to component id. If there is no component
    /// with this type, it will return None.
    pub(crate) fn resolve_component_id(&self, name: &str) -> Result<ComponentID, EntityError> {
        self.components
            .iter()
            .find(|(_, c)| c.get_name() == name)
            .map(|(i, _)| i)
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Returns the field pointer of a component field from a specific
    /// component.
    pub(crate) fn get_component_field(
        &self,
        id: ComponentID,
        name: &str,
    ) -> Result<*const c_void, EntityError> {
        todo!()
    }

    /// Same as [`Self::get_component_field`] but instead provides a mutable
    /// pointer
    pub(crate) fn get_mut_component_field(
        &self,
        id: ComponentID,
        name: &str,
    ) -> Result<*mut c_void, EntityError> {
        todo!()
    }

    /// A wrapper around quickly getting a field pointer from a [`Component`]
    /// while not having a [`ComponentID`]
    pub(crate) fn resolve_and_get_component_field(
        &self,
        id: &str,
        name: &str,
    ) -> Result<*const c_void, EntityError> {
        let id = self.resolve_component_id(id)?;
        self.get_component_field(id, name)
    }

    /// Same as [`Self::resolve_and_get_component_field`] but it returns a
    /// mutable pointer
    pub(crate) fn resolve_and_get_mut_component_field(
        &self,
        id: &str,
        name: &str,
    ) -> Result<*mut c_void, EntityError> {
        let id = self.resolve_component_id(id)?;
        self.get_mut_component_field(id, name)
    }

    /// Get the name of a [`Component`] from a [`ComponentID`]
    pub(crate) fn get_component_name(&self, id: ComponentID) -> Result<&str, EntityError> {
        self.components
            .get(id)
            .ok_or(EntityError::ComponentNotFound)
            .map(|c| c.get_name())
    }
}
