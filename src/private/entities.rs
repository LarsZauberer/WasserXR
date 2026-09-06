use std::{collections::HashMap, os::raw::c_void};

use slotmap::SlotMap;

use crate::{
    errors::EntityError,
    private::{components::Component, manifests::components::ComponentManifest},
    scene::{ComponentID, FieldID, PluginID},
};

/// The entity struct corresponds to the actual entity data. It stores the
/// components it is carrying.
#[derive(Debug, Default)]
pub(crate) struct Entity {
    components: SlotMap<ComponentID, Component>,
    component_ids: HashMap<String, ComponentID>,
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
        plugin_id: PluginID,
        manifest: &ComponentManifest,
    ) -> Result<ComponentID, EntityError> {
        let component = Component::new(manifest, plugin_id);
        if self.component_ids.contains_key(component.get_name()) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        let name = component.get_name().to_owned();
        let id = self.components.insert(component);
        self.component_ids.insert(name, id);
        Ok(id)
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&mut self, id: ComponentID) -> Result<(), EntityError> {
        let component = self
            .components
            .remove(id)
            .ok_or(EntityError::ComponentNotFound)?;
        self.component_ids.remove(component.get_name());
        Ok(())
    }

    /// Get all currently attached component id's from this entity
    pub(crate) fn get_components(&self) -> Vec<ComponentID> {
        self.components.keys().collect()
    }

    /// Get a component from its id.
    pub(crate) fn get_component(&self, id: ComponentID) -> Result<&Component, EntityError> {
        self.components
            .get(id)
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Resolve from component name to component id. If there is no component
    /// with this type, it will return None.
    pub(crate) fn resolve_component_id(&self, name: &str) -> Result<ComponentID, EntityError> {
        self.component_ids
            .get(name)
            .copied()
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Resolve the [`FieldID`] from the field name
    pub(crate) fn resolve_field_id(
        &self,
        component_id: ComponentID,
        name: &str,
    ) -> Result<FieldID, EntityError> {
        self.get_component(component_id)?
            .resolve_field_id(name)
            .map_err(EntityError::from)
    }

    /// Returns the field pointer of a component field from a specific
    /// component.
    pub(crate) fn get_component_field(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<*const c_void, EntityError> {
        todo!()
    }

    /// Same as [`Self::get_component_field`] but instead provides a mutable
    /// pointer
    pub(crate) fn get_mut_component_field(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<*mut c_void, EntityError> {
        todo!()
    }

    /// Get the name of a [`Component`] from a [`ComponentID`]
    pub(crate) fn get_component_name(&self, id: ComponentID) -> Result<&str, EntityError> {
        Ok(self.get_component(id)?.get_name())
    }

    /// Get the [`Field`] name of a [`Component`]
    pub(crate) fn get_field_name(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<&str, EntityError> {
        todo!()
    }
}
