use std::collections::{HashMap, HashSet};

use crate::{
    definitions::systems::Runner,
    ids::{SystemID, TypeID},
    private::system_storage::SystemStorage,
};

type Execution = (Runner, Vec<TypeID>);

/// A dependency-aware copy of the systems present at the start of a tick.
pub(crate) struct SystemStorageSnapshot {
    executions: HashMap<SystemID, Execution>,
    dependents: HashMap<SystemID, Vec<SystemID>>,
    remaining_dependencies: HashMap<SystemID, usize>,
}

impl SystemStorageSnapshot {
    pub(crate) fn new(storage: &SystemStorage) -> Self {
        let mut executions = HashMap::new();
        let mut edges = HashSet::new();
        for (slot, system) in storage.systems.iter() {
            let system_type = system.get_system_type_id();
            let id = SystemID(system_type.0, system_type.1, slot);
            let (runner, type_ids) = system.execution();
            executions.insert(id, (runner, type_ids.to_vec()));
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
        let mut remaining_dependencies: HashMap<_, _> =
            executions.keys().map(|id| (*id, 0)).collect();
        for (dependency, dependent) in edges {
            dependents.entry(dependency).or_default().push(dependent);
            *remaining_dependencies
                .get_mut(&dependent)
                .expect("dependent system missing") += 1;
        }

        Self {
            executions,
            dependents,
            remaining_dependencies,
        }
    }

    /// Returns the length of all the systems that still need to be executed
    pub(crate) fn len(&self) -> usize {
        self.executions.len()
    }

    /// Takes every system whose predecessors have finished.
    pub(crate) fn take_ready(&mut self) -> Vec<(SystemID, Execution)> {
        let ready = self
            .executions
            .keys()
            .filter(|id| self.remaining_dependencies[id] == 0)
            .copied()
            .collect::<Vec<_>>();
        ready
            .into_iter()
            .map(|id| {
                let execution = self.executions.remove(&id).expect("ready system missing");
                (id, execution)
            })
            .collect()
    }

    /// Marks a system finished and returns newly unblocked systems.
    pub(crate) fn complete(&mut self, id: SystemID) -> Vec<(SystemID, Execution)> {
        let mut ready = Vec::new();
        for dependent in self.dependents.get(&id).into_iter().flatten() {
            let remaining = self
                .remaining_dependencies
                .get_mut(dependent)
                .expect("dependent system missing");
            *remaining -= 1;
            if *remaining == 0 {
                ready.push((
                    *dependent,
                    self.executions
                        .remove(dependent)
                        .expect("ready system missing"),
                ));
            }
        }
        ready
    }
}
