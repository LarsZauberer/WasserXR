//! Query request, result, resolution, and locking support.

use std::collections::HashMap;

use crate::{
    errors::SceneError,
    field::{AssetField, Field, FieldAccess},
    ids::{
        AssetFieldTypeID, AssetID, AssetTypeID, ComponentID, ComponentTypeID, EntityID, FieldID,
        FieldTypeID, PluginID,
    },
    scene::{AssetStorage, EntityStorage},
};

/// One asset request: its plugin, type, data string, and requested field types.
pub type AssetQuery<'a> = (PluginID, AssetTypeID, &'a str, &'a [AssetFieldTypeID]);

/// One component request: its plugin, type, and requested field types/access.
pub type ComponentQuery<'a> = (PluginID, ComponentTypeID, &'a [(FieldTypeID, FieldAccess)]);

/// Results for one query group, grouped by matching entity and then by
/// concrete component ID.
pub type ComponentQueryResult = Vec<(EntityID, Vec<(ComponentID, Vec<Field>)>)>;

/// A component request after its type IDs have been resolved to concrete IDs
/// for one entity.
pub(crate) type ResolvedComponent = (ComponentID, Vec<(FieldID, FieldAccess)>);

/// All resolved component requests for one entity.
pub(crate) type ResolvedComponentQuery = Vec<ResolvedComponent>;

/// One queried component and the locked fields returned for it.
pub(crate) type QueriedComponentFields = (ComponentID, Vec<Field>);

/// Resolved component requests grouped by query group and matching entity.
type ComponentQueryMatches = Vec<Vec<(EntityID, ResolvedComponentQuery)>>;

/// Unique component fields arranged in their stable global acquisition order.
type ComponentLockPlan = Vec<(EntityID, ResolvedComponentQuery)>;

/// Unique requested fields gathered before conversion to a stable lock order.
/// A write request supersedes read requests for the same field.
type PendingComponentLocks = HashMap<EntityID, HashMap<ComponentID, HashMap<FieldID, FieldAccess>>>;

/// Stores each physically locked field by its concrete location. Duplicate
/// logical requests use the same entry, so a field is never locked twice by
/// one query. It acts as a cache of locked fields within that query.
type LockedComponentFields = HashMap<(EntityID, ComponentID, FieldID), Field>;

fn add_component_locks(
    pending: &mut PendingComponentLocks,
    entity_id: EntityID,
    components: &ResolvedComponentQuery,
) {
    for (component_id, fields) in components {
        for (field_id, access) in fields {
            pending
                .entry(entity_id)
                .or_default()
                .entry(*component_id)
                .or_default()
                .entry(*field_id)
                .and_modify(|current| {
                    if *access == FieldAccess::Write {
                        *current = FieldAccess::Write;
                    }
                })
                .or_insert(*access);
        }
    }
}

/// Finds the entities matching each query group and gathers their unique field
/// locks. Matching order follows the scene's entity order.
fn resolve_component_queries(
    entities: &EntityStorage,
    groups: &[&[ComponentQuery<'_>]],
) -> Result<(ComponentQueryMatches, PendingComponentLocks), SceneError> {
    let mut matches = Vec::with_capacity(groups.len());
    let mut pending = PendingComponentLocks::new();
    for group in groups {
        let mut group_matches = Vec::new();
        for (entity_id, entity) in entities.iter() {
            let entity = entity.read().expect("entity lock poisoned");
            let Some(components) = entity.resolve_query(group).map_err(SceneError::from)? else {
                continue;
            };
            add_component_locks(&mut pending, entity_id, &components);
            group_matches.push((entity_id, components));
        }
        matches.push(group_matches);
    }
    Ok((matches, pending))
}

/// Converts the gathered locks to a stable global acquisition order. This
/// prevents two concurrent queries from deadlocking on reversed requests.
fn order_component_locks(pending: PendingComponentLocks) -> ComponentLockPlan {
    let mut entities = pending
        .into_iter()
        .map(|(entity_id, components)| {
            let mut components = components
                .into_iter()
                .map(|(component_id, fields)| {
                    let mut fields = fields.into_iter().collect::<Vec<_>>();
                    fields.sort_by_key(|(field_id, _)| *field_id);
                    (component_id, fields)
                })
                .collect::<Vec<_>>();
            components.sort_by_key(|(component_id, _)| *component_id);
            (entity_id, components)
        })
        .collect::<Vec<_>>();
    entities.sort_by_key(|(entity_id, _)| *entity_id);
    entities
}

/// Acquires the complete lock plan recursively, keeping every prior guard on
/// the stack until the final action returns.
fn with_locked_component_fields<T>(
    entities: &EntityStorage,
    plans: &[(EntityID, ResolvedComponentQuery)],
    locked: &mut LockedComponentFields,
    action: impl FnOnce(&LockedComponentFields) -> T,
) -> Result<T, SceneError> {
    let Some(((entity_id, components), remaining)) = plans.split_first() else {
        return Ok(action(locked));
    };
    let entity = entities
        .get(*entity_id)
        .ok_or(SceneError::EntityNotFound)?
        .read()
        .expect("entity lock poisoned");
    entity
        .query_components(components, |component_fields| {
            for (component_id, fields) in component_fields {
                for field in fields {
                    let field_id = match field {
                        Field::Read(id, _) | Field::Write(id, _) => *id,
                    };
                    locked.insert((*entity_id, *component_id, field_id), *field);
                }
            }
            with_locked_component_fields(entities, remaining, locked, action)
        })
        .map_err(SceneError::from)?
}

fn requested_field(access: FieldAccess, locked: Field) -> Field {
    match (access, locked) {
        (FieldAccess::Read, Field::Write(id, pointer)) => Field::Read(id, pointer.cast_const()),
        (_, field) => field,
    }
}

/// Rebuilds the public group/entity/component shape in request order from the
/// uniquely locked physical fields.
fn component_query_results(
    matches: &ComponentQueryMatches,
    locked: &LockedComponentFields,
) -> Vec<ComponentQueryResult> {
    matches
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|(entity_id, components)| {
                    let components = components
                        .iter()
                        .map(|(component_id, fields)| {
                            let fields = fields
                                .iter()
                                .map(|(field_id, access)| {
                                    requested_field(
                                        *access,
                                        locked[&(*entity_id, *component_id, *field_id)],
                                    )
                                })
                                .collect();
                            (*component_id, fields)
                        })
                        .collect();
                    (*entity_id, components)
                })
                .collect()
        })
        .collect()
}

/// Resolves, locks, and shapes one complete component query.
pub(crate) fn query_components<T>(
    entities: &EntityStorage,
    groups: &[&[ComponentQuery<'_>]],
    action: impl FnOnce(&[ComponentQueryResult]) -> T,
) -> Result<T, SceneError> {
    let (matches, pending) = resolve_component_queries(entities, groups)?;
    let plans = order_component_locks(pending);
    with_locked_component_fields(entities, &plans, &mut HashMap::new(), |locked| {
        action(&component_query_results(&matches, locked))
    })
}

/// Resolves the requested fields while the asset collection is read-locked.
pub(crate) fn asset_query_results(
    assets: &AssetStorage,
    requests: &[AssetQuery<'_>],
    asset_ids: &[AssetID],
) -> Result<Vec<Vec<AssetField>>, SceneError> {
    requests
        .iter()
        .zip(asset_ids)
        .map(|((_, _, _, field_type_ids), asset_id)| {
            let asset = assets.get(*asset_id).ok_or(SceneError::AssetNotFound)?;
            field_type_ids
                .iter()
                .map(|field_type_id| {
                    let field_id = asset
                        .resolve_field_id(*field_type_id)
                        .map_err(SceneError::from)?;
                    asset
                        .get_field(field_id)
                        .map(|pointer| AssetField::Read(field_id, pointer))
                        .map_err(SceneError::from)
                })
                .collect()
        })
        .collect()
}
