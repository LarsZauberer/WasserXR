use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use petgraph::{algo::is_cyclic_directed, graphmap::DiGraphMap};

use crate::{
    definitions::systems::Runner,
    errors::{SceneError, SystemError},
    ids::{SystemID, SystemSlot, SystemTypeID, TypeID},
    private::{
        id_store::IDStore,
        manifests::systems::SystemManifest,
        system::{AttacherData, System},
    },
};

type Execution = (Runner, Arc<[TypeID]>);

/// Immutable scheduling data shared by consecutive ticks.
#[derive(Debug)]
struct SystemSchedule {
    executions: HashMap<SystemID, Execution>,
    dependents: HashMap<SystemID, Vec<SystemID>>,
    dependency_counts: HashMap<SystemID, usize>,
}

/// Per-tick execution state backed by a cached schedule.
pub(crate) struct SystemStorageSnapshot {
    schedule: Arc<SystemSchedule>,
    remaining_dependencies: HashMap<SystemID, usize>,
}

/// Concrete systems and their validated dependency relationships.
#[derive(Debug, Default)]
pub(crate) struct SystemStorage {
    systems: IDStore<SystemTypeID, SystemSlot, System>,
    schedule: Option<Arc<SystemSchedule>>,
}

impl SystemStorage {
    pub(crate) fn snapshot(&mut self) -> SystemStorageSnapshot {
        if self.schedule.is_none() {
            self.schedule = Some(Arc::new(SystemSchedule::new(self)));
        }
        SystemStorageSnapshot::new(Arc::clone(
            self.schedule.as_ref().expect("schedule was just created"),
        ))
    }

    pub(crate) fn resolve_id(&self, key: &SystemTypeID) -> Option<SystemID> {
        self.systems
            .resolve_id(key)
            .map(|slot| SystemID(key.0, key.1, slot))
    }

    pub(crate) fn add_system(
        &mut self,
        key: SystemTypeID,
        manifest: &SystemManifest,
        type_ids: Vec<TypeID>,
        requires: Vec<SystemTypeID>,
        wanted_by: Vec<SystemTypeID>,
    ) -> Result<(SystemID, Option<AttacherData>), SceneError> {
        // The candidate has no concrete SystemID until validation succeeds, so
        // graph nodes use its already-stable plugin and system-type key instead.
        // It shouldn't make too much of a performance impact since both are ID's
        for (name, dependency) in manifest.requires.iter().zip(&requires) {
            if !self.systems.contains_name(dependency) {
                return Err(SystemError::DependencyNotFound(name.clone()).into());
            }
        }

        let mut graph: DiGraphMap<SystemTypeID, ()> = DiGraphMap::new();
        for system in self.systems.values() {
            let system_key = system.get_system_type_id();
            graph.add_node(system_key);
            for dependency in system.get_requires() {
                graph.add_edge(*dependency, system_key, ());
            }
            for dependent in system.get_wanted_by() {
                graph.add_edge(system_key, *dependent, ());
            }
        }
        graph.add_node(key);
        for dependency in &requires {
            graph.add_edge(*dependency, key, ());
        }
        for dependent in &wanted_by {
            graph.add_edge(key, *dependent, ());
        }
        if is_cyclic_directed(&graph) {
            return Err(SystemError::DependencyCycle.into());
        }

        let system = System::new(key, manifest, type_ids, requires, wanted_by);
        let attachment = system.attacher();
        let system_id = SystemID(key.0, key.1, self.systems.insert_named(key, system));
        self.schedule = None;
        Ok((system_id, attachment))
    }

    pub(crate) fn remove(&mut self, id: SystemID) -> Result<System, SceneError> {
        let system = self.systems.get(id.2).ok_or(SceneError::SystemNotFound)?;
        let key = system.get_system_type_id();
        if self
            .systems
            .values()
            .any(|system| system.get_requires().contains(&key))
        {
            return Err(SystemError::DependencyInUse.into());
        }
        let system = self.systems.remove(id.2).expect("system was just resolved");
        self.schedule = None;
        Ok(system)
    }

    pub(crate) fn drain(&mut self) -> impl Iterator<Item = System> + '_ {
        self.schedule = None;
        self.systems.drain()
    }
}

impl SystemSchedule {
    fn new(storage: &SystemStorage) -> Self {
        let mut executions = HashMap::new();
        let mut edges = HashSet::new();
        for (slot, system) in storage.systems.iter() {
            let system_type = system.get_system_type_id();
            let id = SystemID(system_type.0, system_type.1, slot);
            let (runner, type_ids) = system.execution();
            executions.insert(id, (runner, Arc::from(type_ids)));
            for dependency in system.get_requires() {
                edges.insert((
                    storage
                        .resolve_id(dependency)
                        .expect("required system missing"),
                    id,
                ));
            }
            for dependent in system.get_wanted_by() {
                if let Some(dependent) = storage.resolve_id(dependent) {
                    edges.insert((id, dependent));
                }
            }
        }

        let mut dependents: HashMap<_, Vec<_>> = HashMap::new();
        let mut dependency_counts: HashMap<_, _> = executions.keys().map(|id| (*id, 0)).collect();
        for (dependency, dependent) in edges {
            dependents.entry(dependency).or_default().push(dependent);
            *dependency_counts
                .get_mut(&dependent)
                .expect("dependent system missing") += 1;
        }

        Self {
            executions,
            dependents,
            dependency_counts,
        }
    }
}

impl SystemStorageSnapshot {
    fn new(schedule: Arc<SystemSchedule>) -> Self {
        let remaining_dependencies = schedule.dependency_counts.clone();
        Self {
            schedule,
            remaining_dependencies,
        }
    }

    /// Returns the length of all the systems that still need to be executed
    pub(crate) fn len(&self) -> usize {
        self.schedule.executions.len()
    }

    /// Returns every system with no predecessors.
    pub(crate) fn take_ready(&self) -> Vec<(SystemID, Execution)> {
        self.schedule
            .executions
            .iter()
            .filter(|(id, _)| self.remaining_dependencies[id] == 0)
            .map(|(id, execution)| (*id, execution.clone()))
            .collect()
    }

    /// Marks a system finished and returns newly unblocked systems.
    pub(crate) fn complete(&mut self, id: SystemID) -> Vec<(SystemID, Execution)> {
        let mut ready = Vec::new();
        for dependent in self.schedule.dependents.get(&id).into_iter().flatten() {
            let remaining = self
                .remaining_dependencies
                .get_mut(dependent)
                .expect("dependent system missing");
            *remaining -= 1;
            if *remaining == 0 {
                ready.push((
                    *dependent,
                    self.schedule
                        .executions
                        .get(dependent)
                        .expect("ready system missing")
                        .clone(),
                ));
            }
        }
        ready
    }
}
