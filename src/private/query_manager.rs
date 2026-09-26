//! Exact component-query definitions, concrete matches, and mutation-driven
//! cache maintenance.

use std::{collections::BTreeMap, collections::HashMap, ffi::c_void, sync::RwLockReadGuard};

use crate::{
    errors::EntityError,
    field::AccessRequest,
    ids::{ComponentID, ComponentSlot, ComponentTypeID, EntitySlot, FieldSlot, FieldTypeID},
    private::{
        components::{Component, ComponentGuard},
        entities::{ComponentStorage, Entity},
    },
    scene::{ComponentQuery, EntityStorage},
};

/// Cache-owned counterpart of one borrowed [`ComponentQuery`] tuple.
///
/// The manager needs an owned form because public queries borrow their field
/// slices only for the duration of a call. Component order, access mode, field
/// order, and duplicates all remain part of the exact cache identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct QueryRequirement {
    component: ComponentTypeID,
    access: AccessRequest,
    fields: Vec<FieldTypeID>,
}

/// Complete owned identity of one submitted component query.
///
/// It is basically the total representation of a full Query for the
/// QueryManager.
///
/// The named type distinguishes an exact cache identity from an arbitrary
/// vector and owns every requirement needed after the public query is dropped.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct QueryKey {
    requirements: Vec<QueryRequirement>,
}

impl<'query> From<&[ComponentQuery<'query>]> for QueryKey {
    /// Copies a borrowed public query into its owned cache representation.
    fn from(query: &[ComponentQuery<'query>]) -> Self {
        let mut requirements = Vec::with_capacity(query.len());
        for &(component, access, fields) in query {
            requirements.push(QueryRequirement {
                component,
                access,
                fields: fields.to_vec(),
            });
        }
        Self { requirements }
    }
}

/// Concrete component access needed to execute one cached query requirement.
///
/// This named representation keeps component slot, access mode, and field
/// slots together. Without it, execution would have to correlate parallel
/// positions in the query key and cached match every time it locks a component
/// or resolves a field pointer.
#[derive(Clone, Debug)]
struct CachedRequirement {
    component: ComponentSlot,
    access: AccessRequest,
    fields: Vec<FieldSlot>,
}

/// Concrete slots through which one entity satisfies a [`QueryKey`].
///
/// Requirements remain in submitted query order, and their fields remain in
/// submitted field order. Raw plugin pointers are deliberately resolved only
/// during execution.
#[derive(Clone, Debug)]
pub(crate) struct CachedMatch {
    entity: EntitySlot,
    requirements: Vec<CachedRequirement>,
}

/// Owns successful exact queries and their deterministic entity matches.
///
/// The ordinary [`HashMap`] intentionally accepts an owned-key allocation on
/// lookup in exchange for standard collision handling and straightforward
/// ownership. Structural mutations incrementally maintain only affected
/// entries; explicit and entity resets clear the map. Every match vector
/// preserves the entity-slot ordering assumption described by
/// [`build_matches`]; component addition inserts at the corresponding ordered
/// position.
#[derive(Debug, Default)]
pub(crate) struct QueryManager {
    queries: HashMap<QueryKey, Vec<CachedMatch>>,
}

impl QueryManager {
    /// Returns the maintained matches for an identical owned query key.
    pub(crate) fn get(&self, key: &QueryKey) -> Option<&[CachedMatch]> {
        self.queries.get(key).map(Vec::as_slice)
    }

    /// Stores the successful result of a previously absent query.
    pub(crate) fn insert(&mut self, key: QueryKey, matches: Vec<CachedMatch>) {
        self.queries.insert(key, matches);
    }

    /// Evicts every cached query definition and concrete match.
    pub(crate) fn clear(&mut self) {
        self.queries.clear();
    }

    /// Re-evaluates entries that require a newly added component type.
    ///
    /// `entity_id` is the stable cached identity and ordering key; `entity`
    /// provides the component storage from which concrete slots are resolved.
    /// `matches` is ordered by entity slot: initial construction preserves the
    /// entity store's slot order, and this insertion preserves that invariant.
    pub(crate) fn component_added(
        &mut self,
        entity_id: EntitySlot,
        entity: &Entity,
        component_type: ComponentTypeID,
    ) {
        self.queries.retain(|key, matches| {
            // If the query doesn't ask for the added component, it is unrelated and just
            // continues to exist.
            if !key
                .requirements
                .iter()
                .any(|requirement| requirement.component == component_type)
            {
                return true;
            }

            // If the matches contain the entity that is changed => remove it
            //
            // We do this since, it is now stale and we need to recalculate if it still
            // matches.
            matches.retain(|item| item.entity != entity_id);
            // Recalculate if the entity still matches the query
            match build_match(entity_id, entity, key) {
                Ok(Some(found)) => {
                    let position = matches.partition_point(|item| item.entity < entity_id);
                    matches.insert(position, found);
                    true
                }
                Ok(None) => true,
                // The storage mutation remains valid. Evict only this cache
                // entry so the next query evaluates and reports the error.
                Err(_) => false,
            }
        });
    }

    /// Removes an entity from every cached query result.
    pub(crate) fn entity_removed(&mut self, entity: EntitySlot) {
        for matches in self.queries.values_mut() {
            matches.retain(|item| item.entity != entity);
        }
    }

    /// Removes matches that use a component being removed.
    pub(crate) fn component_removed(&mut self, component: ComponentID) {
        for matches in self.queries.values_mut() {
            matches.retain(|item| {
                if item.entity != component.0 {
                    return true;
                }

                for requirement in &item.requirements {
                    if requirement.component == component.1 {
                        return false;
                    }
                }
                true
            });
        }
    }
}

/// Resolves an owned key into deterministic concrete entity matches.
///
/// Empty queries retain an empty successful result. Field availability and
/// write access are validated from metadata without invoking plugin getters.
/// The resulting vector preserves `EntityStorage::iter` order. WasserXR assumes
/// the current `IDStore`/`SlotMap` implementation traverses live slot indices,
/// which agrees with the order of simultaneously live [`EntitySlot`] values.
/// `SlotMap` officially describes iteration order as arbitrary, so dependency
/// upgrades must revalidate this assumption. [`QueryManager::component_added`]
/// preserves the established order.
pub(crate) fn build_matches(
    entities: &EntityStorage,
    key: &QueryKey,
) -> Result<Vec<CachedMatch>, EntityError> {
    if key.requirements.is_empty() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    for (entity_id, entity) in entities.iter() {
        if let Some(found) = build_match(entity_id, entity, key)? {
            matches.push(found);
        }
    }
    Ok(matches)
}

/// Resolves one entity against an exact query key.
///
/// Returns `Ok(None)` when a required component is absent and an error when a
/// matching entity cannot satisfy a requested field or access mode.
fn build_match(
    entity_id: EntitySlot,
    entity: &Entity,
    key: &QueryKey,
) -> Result<Option<CachedMatch>, EntityError> {
    let components = entity
        .components()
        .read()
        .expect("entity component lock poisoned");

    // Resolve every required component before validating any fields. An entity
    // missing one component is not a query match and therefore must not report
    // field errors from another component that happens to be present.
    let mut component_slots = Vec::with_capacity(key.requirements.len());
    for request in &key.requirements {
        let Some(component) = components.resolve_id(&request.component) else {
            return Ok(None);
        };
        component_slots.push(component);
    }

    // The slot iterator has the same length and order as the query because the
    // first phase pushed exactly one slot for every requirement.
    let mut resolved_components = component_slots.into_iter();
    let mut requirements = Vec::with_capacity(key.requirements.len());
    for request in &key.requirements {
        let component = resolved_components
            .next()
            .expect("every query requirement has a resolved component");
        requirements.push(build_cached_requirement(&components, request, component)?);
    }
    Ok(Some(CachedMatch {
        entity: entity_id,
        requirements,
    }))
}

/// Resolves the fields for one already-matched component requirement.
///
/// This helper keeps [`build_match`] visibly split into its two semantic
/// phases: first prove that every component exists, then validate and cache
/// fields. Keeping those phases separate prevents non-matching entities from
/// producing unrelated field errors.
fn build_cached_requirement(
    components: &ComponentStorage,
    request: &QueryRequirement,
    component: ComponentSlot,
) -> Result<CachedRequirement, EntityError> {
    let record = components
        .get(component)
        .expect("resolved component exists");
    let record = record.read().expect("component lock poisoned");
    let mut fields = Vec::with_capacity(request.fields.len());
    for &field in &request.fields {
        fields.push(
            record
                .resolve_query_field_slot(field, request.access)
                .map_err(EntityError::from)?,
        );
    }
    Ok(CachedRequirement {
        component,
        access: request.access,
        fields,
    })
}

/// Locks every matched entity's component collection in deterministic order.
///
/// These guards keep cached component slots valid even when execution uses a
/// local match snapshot after releasing the query-manager lock. The manager
/// preserves the entity-slot ordering assumption documented on
/// [`build_matches`], so every query acquires multiple collection locks in the
/// same order and cannot create a lock-order cycle with another query.
pub(crate) fn lock_collections<'a>(
    entities: &'a EntityStorage,
    matches: &[CachedMatch],
) -> Vec<RwLockReadGuard<'a, ComponentStorage>> {
    let mut collections = Vec::with_capacity(matches.len());
    for matched_entity in matches {
        let entity = entities
            .get(matched_entity.entity)
            .expect("cached entity exists");
        collections.push(
            entity
                .components()
                .read()
                .expect("entity component lock poisoned"),
        );
    }
    collections
}

/// Executes cached matches while holding every supplied collection guard and
/// every acquired component guard until the callback returns.
///
/// Locks are acquired in global component-ID order while pointers retain
/// entity, query-entry, and field order.
pub(crate) fn execute(
    matches: &[CachedMatch],
    collections: &[RwLockReadGuard<'_, ComponentStorage>],
    action: impl FnOnce(&[*mut c_void]),
) -> Result<(), EntityError> {
    // Phase 1: describe every component lock the query needs. Collections and
    // matches have the same order and length because lock_collections created
    // one guard for each match immediately before this call.
    let mut locks: BTreeMap<ComponentID, (&std::sync::RwLock<Component>, AccessRequest)> =
        BTreeMap::new();
    for (match_index, matched_entity) in matches.iter().enumerate() {
        let components = &collections[match_index];
        for requirement in &matched_entity.requirements {
            let component = components
                .get(requirement.component)
                .expect("cached component exists");
            let (_, access) = locks
                .entry(ComponentID(matched_entity.entity, requirement.component))
                .or_insert((component, requirement.access));
            if requirement.access == AccessRequest::Write {
                *access = AccessRequest::Write;
            }
        }
    }

    // Phase 2: BTreeMap iteration is sorted by ComponentID, so every query
    // acquires component locks in the same global order. Duplicate requests
    // share one guard using the strongest requested access mode.
    let mut guards = BTreeMap::new();
    for (component_id, (component, access)) in locks {
        guards.insert(component_id, ComponentGuard::lock(component, access));
    }

    // Phase 3: walk the cached representation in its original entity,
    // requirement, and field order. Lock ordering is deliberately independent
    // of pointer ordering, so the callback observes the submitted query order.
    let mut pointers = Vec::new();
    for matched_entity in matches {
        for requirement in &matched_entity.requirements {
            let component_id = ComponentID(matched_entity.entity, requirement.component);
            let component = guards[&component_id].component();
            for &field in &requirement.fields {
                pointers.push(
                    component
                        .query_field_slot(field, requirement.access)
                        .map_err(EntityError::from)?,
                );
            }
        }
    }
    action(&pointers);
    Ok(())
}
