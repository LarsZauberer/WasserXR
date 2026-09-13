use crate::{
    definitions::systems::Detacher,
    ids::{PluginID, SystemTypeID, TypeID},
    private::manifests::systems::SystemManifest,
    scene::Scene,
};

/// A concrete system created from a plugin's system manifest.
#[derive(Debug)]
pub(crate) struct System {
    plugin_id: PluginID,
    system_type_id: SystemTypeID,
    requires: Vec<(PluginID, SystemTypeID)>,
    wanted_by: Vec<(PluginID, SystemTypeID)>,
    detacher: Option<Detacher>,
    type_ids: Vec<TypeID>,
}

impl System {
    pub(crate) fn new(
        scene: &Scene,
        plugin_id: PluginID,
        system_type_id: SystemTypeID,
        manifest: &SystemManifest,
        type_ids: Vec<TypeID>,
        requires: Vec<(PluginID, SystemTypeID)>,
        wanted_by: Vec<(PluginID, SystemTypeID)>,
    ) -> Self {
        if let Some(attacher) = manifest.attacher {
            unsafe { attacher(scene, type_ids.as_ptr(), type_ids.len()) };
        }
        Self {
            plugin_id,
            system_type_id,
            requires,
            wanted_by,
            detacher: manifest.detacher,
            type_ids,
        }
    }

    pub(crate) fn get_plugin_id(&self) -> PluginID {
        self.plugin_id
    }

    pub(crate) fn get_system_type_id(&self) -> SystemTypeID {
        self.system_type_id
    }

    pub(crate) fn get_requires(&self) -> &[(PluginID, SystemTypeID)] {
        &self.requires
    }

    pub(crate) fn get_wanted_by(&self) -> &[(PluginID, SystemTypeID)] {
        &self.wanted_by
    }

    pub(crate) fn detach(self, scene: &Scene) {
        if let Some(detacher) = self.detacher {
            unsafe { detacher(scene, self.type_ids.as_ptr(), self.type_ids.len()) };
        }
    }
}
