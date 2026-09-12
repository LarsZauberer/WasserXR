use crate::{
    definitions::systems::Detacher,
    ids::{PluginID, TypeID},
    private::manifests::systems::SystemManifest,
    scene::Scene,
};

/// A concrete system created from a plugin's system manifest.
#[derive(Debug)]
pub(crate) struct System {
    pub(crate) plugin_id: PluginID,
    detacher: Option<Detacher>,
    type_ids: Vec<TypeID>,
}

impl System {
    pub(crate) fn new(
        scene: &Scene,
        plugin_id: PluginID,
        manifest: &SystemManifest,
        type_ids: Vec<TypeID>,
    ) -> Self {
        if let Some(attacher) = manifest.attacher {
            unsafe { attacher(scene, type_ids.as_ptr(), type_ids.len()) };
        }
        Self {
            plugin_id,
            detacher: manifest.detacher,
            type_ids,
        }
    }

    pub(crate) fn detach(self, scene: &Scene) {
        if let Some(detacher) = self.detacher {
            unsafe { detacher(scene, self.type_ids.as_ptr(), self.type_ids.len()) };
        }
    }
}
