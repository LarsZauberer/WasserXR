use petgraph::{algo::toposort, graphmap::DiGraphMap};

use crate::{
    definitions::systems::Runner,
    ids::{SystemID, TypeID},
    private::system_storage::SystemStorage,
};

/// A dependency-ordered copy of the systems present at the start of a tick.
pub(crate) struct SystemStorageSnapshot {
    executions: Vec<(Runner, Vec<TypeID>)>,
}

impl SystemStorageSnapshot {
    pub(crate) fn new(storage: &SystemStorage) -> Self {
        let mut graph: DiGraphMap<SystemID, ()> = DiGraphMap::new();
        for (id, system) in storage.systems.iter() {
            graph.add_node(id);
            for dependency in system.get_requires() {
                graph.add_edge(
                    storage
                        .systems
                        .resolve_id(dependency)
                        .expect("required system missing"),
                    id,
                    (),
                );
            }
            for dependent in system.get_wanted_by() {
                if let Some(dependent) = storage.systems.resolve_id(dependent) {
                    graph.add_edge(id, dependent, ());
                }
            }
        }

        let mut executions = toposort(&graph, None)
            .expect("validated system dependencies contain a cycle")
            .into_iter()
            .map(|id| {
                let (runner, type_ids) = storage
                    .systems
                    .get(id)
                    .expect("scheduled system missing")
                    .execution();
                (runner, type_ids.to_vec())
            })
            .collect::<Vec<_>>();
        executions.reverse();
        Self { executions }
    }

    pub(crate) fn next(&mut self) -> Option<(Runner, Vec<TypeID>)> {
        self.executions.pop()
    }
}
