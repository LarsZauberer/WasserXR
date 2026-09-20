use std::sync::Arc;

use petgraph::{algo::is_cyclic_directed, graphmap::DiGraphMap};

use crate::{
    errors::{SceneError, SystemError},
    ids::{SystemID, SystemSlot, SystemTypeID, TypeID},
    private::{
        id_store::IDStore,
        manifests::systems::SystemManifest,
        system::{AttacherData, System},
        system_storage_snapshot::{SystemSchedule, SystemStorageSnapshot},
    },
};

/// Concrete systems and their validated dependency relationships.
#[derive(Debug, Default)]
pub(crate) struct SystemStorage {
    pub(super) systems: IDStore<SystemTypeID, SystemSlot, System>,
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
