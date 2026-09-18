use std::sync::{RwLock, RwLockReadGuard};

use crate::{
    errors::EntityError,
    ids::{ComponentSlot, ComponentTypeID, FieldSlot, FieldTypeID},
    private::{components::Component, id_store::IDStore, manifests::components::ComponentManifest},
};

pub(crate) type ComponentStorage = IDStore<ComponentTypeID, ComponentSlot, RwLock<Component>>;

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

    /// Keeps this entity's component membership stable during a scene query.
    ///
    /// # Design decisions
    ///
    /// The scene holds these guards before taking any component data locks.
    /// Borrowing the existing storage prevents removal without copying data
    /// or introducing shared ownership of individual components.
    pub(crate) fn lock_components(&self) -> RwLockReadGuard<'_, ComponentStorage> {
        self.components
            .read()
            .expect("entity component lock poisoned")
    }

    /// Runs an action with a component while holding the component collection's
    /// read lock, preventing the component from being removed during the
    /// action.
    fn with_component<T>(
        &self,
        id: ComponentSlot,
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
        component_type_id: ComponentTypeID,
        manifest: &ComponentManifest,
    ) -> Result<ComponentSlot, EntityError> {
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        if components.contains_name(&component_type_id) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        Ok(components.insert_named(
            component_type_id,
            RwLock::new(Component::new(manifest, component_type_id)),
        ))
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&self, id: ComponentSlot) -> Result<(), EntityError> {
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

    /// Get all currently attached component slots from this entity.
    pub(crate) fn get_component_slots(&self) -> Vec<ComponentSlot> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .keys()
            .collect()
    }

    /// Resolve a component type to its entity-local slot.
    pub(crate) fn resolve_component_slot(
        &self,
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentSlot, EntityError> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .resolve_id(&component_type_id)
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Resolve a field type to its component-local slot.
    pub(crate) fn resolve_field_slot(
        &self,
        component: ComponentSlot,
        field_type_id: FieldTypeID,
    ) -> Result<FieldSlot, EntityError> {
        self.with_component(component, |component| {
            component
                .resolve_field_slot(field_type_id)
                .map_err(EntityError::from)
        })
    }

    /// Get the name of a component from its entity-local slot.
    pub(crate) fn get_component_name(&self, id: ComponentSlot) -> Result<String, EntityError> {
        self.with_component(id, |component| Ok(component.get_name().to_owned()))
    }

    /// Get the [`Field`] name of a [`Component`]
    pub(crate) fn get_field_name(
        &self,
        component: ComponentSlot,
        field: FieldSlot,
    ) -> Result<String, EntityError> {
        self.with_component(component, |component| {
            component.get_field_name(field).map_err(EntityError::from)
        })
    }
}
