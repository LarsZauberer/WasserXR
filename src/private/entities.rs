use std::sync::RwLock;

use crate::{
    errors::EntityError,
    ids::{ComponentID, ComponentTypeID, FieldID, FieldTypeID, PluginID},
    private::{components::Component, id_store::IDStore, manifests::components::ComponentManifest},
    query::{ComponentQuery, QueriedComponentFields, ResolvedComponent, ResolvedComponentQuery},
};

type ComponentStorage = IDStore<(PluginID, ComponentTypeID), ComponentID, Component>;

/// Recursively locks each component's fields so earlier locks remain alive
/// while later components and the final action are processed.
fn lock_component_fields<T>(
    components: &ComponentStorage,
    requests: &[ResolvedComponent],
    fields: &mut Vec<QueriedComponentFields>,
    action: impl FnOnce(&[QueriedComponentFields]) -> T,
) -> Result<T, EntityError> {
    let Some(((component_id, requested_fields), remaining)) = requests.split_first() else {
        return Ok(action(fields));
    };
    let component = components
        .get(*component_id)
        .ok_or(EntityError::ComponentNotFound)?;
    component
        .query_fields(requested_fields, |component_fields| {
            fields.push((*component_id, component_fields.to_vec()));
            lock_component_fields(components, remaining, fields, action)
        })
        .map_err(EntityError::from)?
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
        id: ComponentID,
        action: impl FnOnce(&Component) -> Result<T, EntityError>,
    ) -> Result<T, EntityError> {
        let components = self
            .components
            .read()
            .expect("entity component lock poisoned");
        let component = components.get(id).ok_or(EntityError::ComponentNotFound)?;
        action(component)
    }

    /// Add a new component to the entity. The function will reject the add, if
    /// a component of that type already exists.
    pub(crate) fn add_component(
        &self,
        plugin_id: PluginID,
        component_type_id: ComponentTypeID,
        manifest: &ComponentManifest,
    ) -> Result<ComponentID, EntityError> {
        let component = Component::new(manifest, plugin_id);
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        let type_id = (plugin_id, component_type_id);
        if components.contains_name(&type_id) {
            return Err(EntityError::ComponentAlreadyExists);
        }
        Ok(components.insert_named(type_id, component))
    }

    /// Remove a component from the entity
    pub(crate) fn remove_component(&self, id: ComponentID) -> Result<(), EntityError> {
        let mut components = self
            .components
            .write()
            .expect("entity component lock poisoned");
        let component = components
            .remove(id)
            .ok_or(EntityError::ComponentNotFound)?;
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

    /// Resolve a component type to its component ID.
    pub(crate) fn resolve_component_id(
        &self,
        plugin_id: PluginID,
        component_type_id: ComponentTypeID,
    ) -> Result<ComponentID, EntityError> {
        self.components
            .read()
            .expect("entity component lock poisoned")
            .resolve_id(&(plugin_id, component_type_id))
            .ok_or(EntityError::ComponentNotFound)
    }

    /// Resolve the [`FieldID`] from its field type ID.
    pub(crate) fn resolve_field_id(
        &self,
        component_id: ComponentID,
        field_type_id: FieldTypeID,
    ) -> Result<FieldID, EntityError> {
        self.with_component(component_id, |component| {
            component
                .resolve_field_id(field_type_id)
                .map_err(EntityError::from)
        })
    }

    /// Resolves a component query for this entity, returning `None` when the
    /// entity does not contain every requested component.
    pub(crate) fn resolve_query(
        &self,
        requests: &[ComponentQuery<'_>],
    ) -> Result<Option<ResolvedComponentQuery>, EntityError> {
        let components = self
            .components
            .read()
            .expect("entity component lock poisoned");
        requests
            .iter()
            .map(|(plugin_id, component_type_id, fields)| {
                let Some(component_id) = components.resolve_id(&(*plugin_id, *component_type_id))
                else {
                    return Ok(None);
                };
                let component = components
                    .get(component_id)
                    .expect("resolved component missing");
                let fields = fields
                    .iter()
                    .map(|(field_type_id, access)| {
                        component
                            .resolve_field_id(*field_type_id)
                            .map(|field_id| (field_id, *access))
                            .map_err(EntityError::from)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some((component_id, fields)))
            })
            .collect::<Result<Option<Vec<_>>, _>>()
    }

    /// Locks fields across multiple components in the supplied component-ID
    /// order and holds every lock until `action` returns.
    pub(crate) fn query_components<T>(
        &self,
        requests: &[ResolvedComponent],
        action: impl FnOnce(&[QueriedComponentFields]) -> T,
    ) -> Result<T, EntityError> {
        let components = self
            .components
            .read()
            .expect("entity component lock poisoned");
        lock_component_fields(
            &components,
            requests,
            &mut Vec::with_capacity(requests.len()),
            action,
        )
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
