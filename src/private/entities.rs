use std::sync::RwLock;

use crate::{
    errors::EntityError,
    field::{Field, FieldAccess},
    private::{components::Component, id_store::IDStore, manifests::components::ComponentManifest},
    scene::{ComponentID, FieldID, PluginID},
};

type ComponentStorage = IDStore<String, ComponentID, RwLock<Component>>;

/// The entity struct corresponds to the actual entity data. It stores the
/// components it is carrying.
#[derive(Debug, Default)]
pub(crate) struct Entity {
    components: RwLock<ComponentStorage>,
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
        let component = components.get(id).ok_or(EntityError::ComponentNotFound)?;
        action(&component.read().expect("component lock poisoned"))
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
        if components.contains_name(&name) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        Ok(components.insert_named(name, RwLock::new(component)))
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&self, id: ComponentID) -> Result<(), EntityError> {
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        let component = components
            .remove(id)
            .ok_or(EntityError::ComponentNotFound)?
            .into_inner()
            .expect("component lock poisoned");
        drop(components);
        drop(component);
        Ok(())
    }

    /// Get all currently attached component id's from this entity
    pub(crate) fn get_components(&self) -> Vec<ComponentID> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .keys()
            .collect()
    }

    /// Resolve from component name to component id. If there is no component
    /// with this type, it will return None.
    pub(crate) fn resolve_component_id(&self, name: &str) -> Result<ComponentID, EntityError> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .resolve_id(name)
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

    /// Locks component fields in a consistent order.
    pub(crate) fn query_component_fields<T>(
        &self,
        component_id: ComponentID,
        requests: &[(FieldID, FieldAccess)],
        action: impl FnOnce(&[Field]) -> T,
    ) -> Result<T, EntityError> {
        self.with_component(component_id, |component| {
            component
                .query_fields(requests, action)
                .map_err(EntityError::from)
        })
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
                .map_err(EntityError::from)
        })
    }
}
