use crate::{
    definitions::systems::{Attacher, Detacher, Runner},
    ids::{SystemTypeID, TypeID},
    private::manifests::systems::SystemManifest,
    scene::Scene,
};

pub(crate) type AttacherData = (Attacher, Vec<TypeID>);

/// A concrete system created from a plugin's system manifest.
#[derive(Debug)]
pub(crate) struct System {
    system_type_id: SystemTypeID,
    requires: Vec<SystemTypeID>,
    wanted_by: Vec<SystemTypeID>,
    runner: Runner,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
    type_ids: Vec<TypeID>,
}

impl System {
    pub(crate) fn new(
        system_type_id: SystemTypeID,
        manifest: &SystemManifest,
        type_ids: Vec<TypeID>,
        requires: Vec<SystemTypeID>,
        wanted_by: Vec<SystemTypeID>,
    ) -> Self {
        Self {
            system_type_id,
            requires,
            wanted_by,
            runner: manifest.runner,
            attacher: manifest.attacher,
            detacher: manifest.detacher,
            type_ids,
        }
    }

    pub(crate) fn get_system_type_id(&self) -> SystemTypeID {
        self.system_type_id
    }

    pub(crate) fn get_requires(&self) -> &[SystemTypeID] {
        &self.requires
    }

    pub(crate) fn get_wanted_by(&self) -> &[SystemTypeID] {
        &self.wanted_by
    }

    pub(crate) fn execution(&self) -> (Runner, &[TypeID]) {
        (self.runner, &self.type_ids)
    }

    pub(crate) fn attacher(&self) -> Option<AttacherData> {
        self.attacher
            .map(|attacher| (attacher, self.type_ids.clone()))
    }

    pub(crate) fn detach(self, scene: &Scene) {
        if let Some(detacher) = self.detacher {
            unsafe { detacher(scene, self.type_ids.as_ptr(), self.type_ids.len()) };
        }
    }
}
