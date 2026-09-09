use std::{collections::HashMap, os::raw::c_void, sync::RwLock};

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
    components: RwLock<ComponentStorage>,
}

#[derive(Debug, Default)]
struct ComponentStorage {
    components: SlotMap<ComponentID, Component>,
    component_ids: HashMap<String, ComponentID>,
}

impl Entity {
    /// Create a new entity
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Runs an action with a component while holding the component collection's
    /// read lock, preventing the component from being removed during the
    /// action.
    fn with_component<T>(
        &self,
        id: ComponentID,
        action: impl FnOnce(&Component) -> Result<T, EntityError>,
    ) -> Result<T, EntityError> {
        let components = self
            .components
            .read()
            .expect("entity component lock poisoned");
        let component = components
            .components
            .get(id)
            .ok_or(EntityError::ComponentNotFound)?;
        action(component)
    }

    /// Add a new component to the entity. The function will reject the add, if
    /// a component of that type already exists.
    pub(crate) fn add_component(
        &self,
        plugin_id: PluginID,
        manifest: &ComponentManifest,
    ) -> Result<ComponentID, EntityError> {
        let component = Component::new(manifest, plugin_id);
        let name = component.get_name().to_owned();
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        if components.component_ids.contains_key(&name) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        let id = components.components.insert(component);
        components.component_ids.insert(name, id);
        Ok(id)
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&self, id: ComponentID) -> Result<(), EntityError> {
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        let component = components
            .components
            .remove(id)
            .ok_or(EntityError::ComponentNotFound)?;
        let name = component.get_name().to_owned();
        components.component_ids.remove(&name);
        drop(components);
        drop(component);
        Ok(())
    }

    /// Get all currently attached component id's from this entity
    pub(crate) fn get_components(&self) -> Vec<ComponentID> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .components
            .keys()
            .collect()
    }

    /// Resolve from component name to component id. If there is no component
    /// with this type, it will return None.
    pub(crate) fn resolve_component_id(&self, name: &str) -> Result<ComponentID, EntityError> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .component_ids
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
        self.with_component(component_id, |component| {
            component.resolve_field_id(name).map_err(EntityError::from)
        })
    }

    /// Returns the field pointer of a component field from a specific
    /// component.
    pub(crate) fn get_component_field(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<*const c_void, EntityError> {
        self.with_component(component_id, |component| {
            component.get_field_ptr(field_id).map_err(EntityError::from)
        })
    }

    /// Same as [`Self::get_component_field`] but instead provides a mutable
    /// pointer
    pub(crate) fn get_mut_component_field(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<*mut c_void, EntityError> {
        let components = self
            .components
            .read()
            .expect("entity component lock poisoned");
        let component = components
            .components
            .get(component_id)
            .ok_or(EntityError::ComponentNotFound)?;
        component
            .get_field_mut_ptr(field_id)
            .map_err(EntityError::from)
    }

    /// Get the name of a [`Component`] from a [`ComponentID`]
    pub(crate) fn get_component_name(&self, id: ComponentID) -> Result<String, EntityError> {
        self.with_component(id, |component| Ok(component.get_name().to_owned()))
    }

    /// Get the [`Field`] name of a [`Component`]
    pub(crate) fn get_field_name(
        &self,
        component_id: ComponentID,
        field_id: FieldID,
    ) -> Result<String, EntityError> {
        self.with_component(component_id, |component| {
            component
                .get_field_name(field_id)
                .map(str::to_owned)
                .map_err(EntityError::from)
        })
    }
}
