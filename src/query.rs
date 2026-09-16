//! Query request, result, resolution, and locking support.

use std::{collections::BTreeMap, ffi::c_void};

use crate::{
    errors::SceneError,
    field::FieldAccess,
    ids::{
        AssetFieldTypeID, AssetTypeID, ComponentID, ComponentTypeID, EntityID, FieldID,
        FieldTypeID, PluginID,
    },
    scene::EntityStorage,
};

/// One asset request: its plugin, type, data string, and requested field types.
pub type AssetQuery<'a> = (PluginID, AssetTypeID, &'a str, &'a [AssetFieldTypeID]);

/// One component request: its plugin, type, and requested field types/access.
pub type ComponentQuery<'a> = (PluginID, ComponentTypeID, &'a [(FieldTypeID, FieldAccess)]);

/// Results for one component query, grouped by matching entity and then by
/// concrete component ID. Field access is governed by its corresponding
/// [`ComponentQuery`] request, and each pointer is valid only during the query
/// callback.
pub type ComponentQueryResult = Vec<(EntityID, Vec<(ComponentID, Vec<(FieldID, *mut c_void)>)>)>;

/// A component request after its type IDs have been resolved to concrete IDs
/// for one entity.
pub(crate) type ResolvedComponent = (ComponentID, Vec<(FieldID, FieldAccess)>);

/// All resolved component requests for one entity.
pub(crate) type ResolvedComponentQuery = Vec<ResolvedComponent>;

/// One queried component and the locked fields returned for it.
pub(crate) type QueriedComponentFields = (ComponentID, Vec<(FieldID, *mut c_void)>);

/// Resolved component requests grouped by entity.
type ResolvedEntities = Vec<(EntityID, ResolvedComponentQuery)>;

/// Unique requested fields grouped into a stable component lock order. A write
/// request supersedes read requests for the same field.
type PendingComponentLocks =
    BTreeMap<EntityID, BTreeMap<ComponentID, BTreeMap<FieldID, FieldAccess>>>;

/// Stores each queried pointer by its concrete location. Duplicate logical
/// requests use the same entry.
type LockedComponentFields = BTreeMap<(EntityID, ComponentID, FieldID), *mut c_void>;

/// Finds matching entities and builds their unique, globally ordered lock plan.
fn plan_component_query(
    entities: &EntityStorage,
    requests: &[ComponentQuery<'_>],
) -> Result<(ResolvedEntities, ResolvedEntities), SceneError> {
    let mut matches = Vec::new();
    let mut pending = PendingComponentLocks::new();
    for (entity_id, entity) in entities.iter() {
        let Some(components) = entity.resolve_query(requests).map_err(SceneError::from)? else {
            continue;
        };
        for (component_id, fields) in &components {
            let pending_fields = pending
                .entry(entity_id)
                .or_default()
                .entry(*component_id)
                .or_default();
            for (field_id, access) in fields {
                pending_fields
                    .entry(*field_id)
                    .and_modify(|current| {
                        if *access == FieldAccess::Write {
                            *current = FieldAccess::Write;
                        }
                    })
                    .or_insert(*access);
            }
        }
        matches.push((entity_id, components));
    }

    let plan = pending
        .into_iter()
        .map(|(entity_id, components)| {
            let components = components
                .into_iter()
                .map(|(component_id, fields)| (component_id, fields.into_iter().collect()))
                .collect::<Vec<_>>();
            (entity_id, components)
        })
        .collect::<Vec<_>>();
    Ok((matches, plan))
}

/// Acquires the complete lock plan recursively, keeping every prior guard on
/// the stack until the final action returns.
fn with_locked_components<T>(
    entities: &EntityStorage,
    plans: &[(EntityID, ResolvedComponentQuery)],
    locked: &mut LockedComponentFields,
    action: impl FnOnce(&LockedComponentFields) -> T,
) -> Result<T, SceneError> {
    let Some(((entity_id, components), remaining)) = plans.split_first() else {
        return Ok(action(locked));
    };
    let entity = entities.get(*entity_id).ok_or(SceneError::EntityNotFound)?;
    entity
        .with_locked_components(components, |component_fields| {
            for (component_id, fields) in component_fields {
                for (field_id, pointer) in fields {
                    locked.insert((*entity_id, *component_id, *field_id), *pointer);
                }
            }
            with_locked_components(entities, remaining, locked, action)
        })
        .map_err(SceneError::from)?
}

/// Rebuilds the public entity/component shape in request order from the
/// uniquely locked physical fields.
fn component_query_results(
    matches: &ResolvedEntities,
    locked: &LockedComponentFields,
) -> ComponentQueryResult {
    matches
        .iter()
        .map(|(entity_id, components)| {
            let components = components
                .iter()
                .map(|(component_id, fields)| {
                    let fields = fields
                        .iter()
                        .map(|(field_id, _)| {
                            (*field_id, locked[&(*entity_id, *component_id, *field_id)])
                        })
                        .collect();
                    (*component_id, fields)
                })
                .collect();
            (*entity_id, components)
        })
        .collect()
}

/// Resolves, locks, and shapes one complete component query.
pub(crate) fn query_components<T>(
    entities: &EntityStorage,
    requests: &[ComponentQuery<'_>],
    action: impl FnOnce(&ComponentQueryResult) -> T,
) -> Result<T, SceneError> {
    let (matches, plans) = plan_component_query(entities, requests)?;
    debug_assert!(
        plans
            .windows(2)
            .all(|entities| entities[0].0 < entities[1].0)
    );
    with_locked_components(entities, &plans, &mut BTreeMap::new(), |locked| {
        action(&component_query_results(&matches, locked))
    })
}
