use std::{collections::BTreeMap, ffi::c_void, sync::RwLock};

use crate::{
    errors::EntityError,
    field::AccessRequest,
    ids::{ComponentID, ComponentSlot, ComponentTypeID, EntitySlot, FieldSlot, FieldTypeID},
    private::{
        components::{Component, ComponentGuard},
        id_store::IDStore,
        manifests::components::ComponentManifest,
    },
};

type ComponentStorage = IDStore<ComponentTypeID, ComponentSlot, RwLock<Component>>;

/// Calls `action` with fields from every entity matching every query entry.
pub(crate) fn with_component_fields<'a>(
    entities: impl Iterator<Item = (EntitySlot, &'a Entity)>,
    query: &[(ComponentTypeID, AccessRequest, &[FieldTypeID])],
    action: impl FnOnce(&[*mut c_void]),
) -> Result<(), EntityError> {
    // Acquire every collection guard before any component data guard.
    let ordered_entities: BTreeMap<_, _> = entities.collect();
    let collections: Vec<_> = ordered_entities
        .iter()
        .map(|(&id, entity)| {
            (
                id,
                entity
                    .components
                    .read()
                    .expect("entity component lock poisoned"),
            )
        })
        .collect();

    // Lock each component once in global order while retaining request order.
    let mut locks = BTreeMap::new();
    let mut requests = Vec::new();
    for (entity_id, components) in &collections {
        let ids: Option<Vec<_>> = query
            .iter()
            .map(|(component_type, _, _)| components.resolve_id(component_type))
            .collect();
        let Some(ids) = ids else { continue };

        for (slot, &(_, access, fields)) in ids.into_iter().zip(query) {
            let id = ComponentID(*entity_id, slot);
            let component = components.get(slot).expect("resolved component exists");
            let (_, lock_access) = locks.entry(id).or_insert((component, access));
            if access == AccessRequest::Write {
                *lock_access = AccessRequest::Write;
            }
            requests.push((id, access, fields));
        }
    }

    let guards: BTreeMap<_, _> = locks
        .into_iter()
        .map(|(key, (component, access))| (key, ComponentGuard::lock(component, access)))
        .collect();
    let mut pointers = Vec::new();
    for (key, access, fields) in requests {
        for &field in fields {
            pointers.push(
                guards[&key]
                    .component()
                    .query_field(field, access)
                    .map_err(EntityError::from)?,
            );
        }
    }
    action(&pointers);
    Ok(())
}

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
