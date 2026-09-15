use petgraph::{algo::is_cyclic_directed, graphmap::DiGraphMap};

use crate::{
    errors::{SceneError, SystemError},
    ids::{PluginID, SystemID, SystemTypeID, TypeID},
    private::{id_store::IDStore, manifests::systems::SystemManifest, system::System},
    scene::Scene,
};

type SystemKey = (PluginID, SystemTypeID);

/// Concrete systems and their validated dependency relationships.
#[derive(Debug, Default)]
pub(crate) struct SystemStorage {
    pub(super) systems: IDStore<SystemKey, SystemID, System>,
}

impl SystemStorage {
    pub(crate) fn resolve_id(&self, key: &SystemKey) -> Option<SystemID> {
        self.systems.resolve_id(key)
    }

    pub(crate) fn add_system(
        &mut self,
        scene: &Scene,
        key: SystemKey,
        manifest: &SystemManifest,
        type_ids: Vec<TypeID>,
        requires: Vec<SystemKey>,
        wanted_by: Vec<SystemKey>,
    ) -> Result<SystemID, SceneError> {
        // The candidate has no concrete SystemID until validation succeeds, so
        // graph nodes use its already-stable plugin and system-type key instead.
        // It shouldn't make too much of a performance impact since both are ID's
        for (name, dependency) in manifest.requires.iter().zip(&requires) {
            if !self.systems.contains_name(dependency) {
                return Err(SystemError::DependencyNotFound(name.clone()).into());
            }
        }

        let mut graph: DiGraphMap<SystemKey, ()> = DiGraphMap::new();
        for system in self.systems.values() {
            let system_key = (system.get_plugin_id(), system.get_system_type_id());
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

        let system = System::new(scene, key.0, key.1, manifest, type_ids, requires, wanted_by);
        Ok(self.systems.insert_named(key, system))
    }

    pub(crate) fn remove(&mut self, id: SystemID) -> Result<System, SceneError> {
        let system = self.systems.get(id).ok_or(SceneError::SystemNotFound)?;
        let key = (system.get_plugin_id(), system.get_system_type_id());
        if self
            .systems
            .values()
            .any(|system| system.get_requires().contains(&key))
        {
            return Err(SystemError::DependencyInUse.into());
        }
        Ok(self.systems.remove(id).expect("system was just resolved"))
    }

    pub(crate) fn into_values(self) -> impl Iterator<Item = System> {
        self.systems.into_values()
    }
}
