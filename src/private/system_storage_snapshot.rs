use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    definitions::systems::Runner,
    ids::{SystemID, TypeID},
    private::system_storage::SystemStorage,
};

pub(super) type Execution = (Runner, Arc<[TypeID]>);

/// Immutable scheduling data shared by consecutive ticks.
#[derive(Debug)]
pub(super) struct SystemSchedule {
    executions: HashMap<SystemID, Execution>,
    dependents: HashMap<SystemID, Vec<SystemID>>,
    dependency_counts: HashMap<SystemID, usize>,
}

/// Per-tick execution state backed by a cached schedule.
pub(crate) struct SystemStorageSnapshot {
    schedule: Arc<SystemSchedule>,
    remaining_dependencies: HashMap<SystemID, usize>,
}

impl SystemSchedule {
    pub(super) fn new(storage: &SystemStorage) -> Self {
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
    pub(super) fn new(schedule: Arc<SystemSchedule>) -> Self {
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
