//! C-compatible system declarations used by plugin manifests.

use std::{collections::HashSet, ffi::c_char};

use crate::{
    bindings::scene::WXREntity,
    scene::{
        Scene,
        plugin::manifest::{ManifestError, copy_name, descriptor_slice, missing_callback},
    },
};

use super::{SelectionGroup, SystemDefinition};

pub type Runner =
    unsafe extern "C" fn(*mut Scene, f32, *const *const WXREntity, *const usize, usize);
pub type Attacher = unsafe extern "C" fn(*mut Scene);
pub type Detacher = unsafe extern "C" fn(*mut Scene);

/// C-compatible declaration of one entity selection group.
#[repr(C)]
pub struct WXRSystemEntityGroupDescriptor {
    pub components: *const *const c_char,
    pub component_count: usize,
}

/// C-compatible declaration of one plugin-provided system type.
#[repr(C)]
pub struct WXRSystemDescriptor {
    pub name: *const c_char,
    // Expanded for cbindgen; aliases inside Option emit incomplete C types.
    pub runner:
        Option<unsafe extern "C" fn(*mut Scene, f32, *const *const WXREntity, *const usize, usize)>,
    pub attach: Option<unsafe extern "C" fn(*mut Scene)>,
    pub detach: Option<unsafe extern "C" fn(*mut Scene)>,
    pub entity_groups: *const WXRSystemEntityGroupDescriptor,
    pub entity_group_count: usize,
}

// Descriptors are immutable process-lifetime declarations. Loading their raw
// pointers remains unsafe and validation copies all data used by the host.
unsafe impl Sync for WXRSystemEntityGroupDescriptor {}
unsafe impl Sync for WXRSystemDescriptor {}

impl WXRSystemDescriptor {
    pub(crate) unsafe fn validate(&self, plugin: &str) -> Result<SystemDefinition, ManifestError> {
        let name = unsafe { copy_name(self.name, "system") }?;
        let runner = self
            .runner
            .ok_or_else(|| missing_callback("system", &name, "runner"))?;
        let groups = unsafe {
            descriptor_slice(
                self.entity_groups,
                self.entity_group_count,
                "system entity groups",
            )
        }?;
        let mut unique_groups = HashSet::new();
        let mut validated_groups = Vec::with_capacity(groups.len());
        for group in groups {
            let components = unsafe {
                descriptor_slice(
                    group.components,
                    group.component_count,
                    "entity group components",
                )
            }?;
            let mut component_names = Vec::with_capacity(components.len());
            for component in components {
                component_names.push(unsafe { copy_name(*component, "entity group component") }?);
            }
            let group = SelectionGroup::new(component_names);
            if let Some(names) = group
                .components()
                .windows(2)
                .find(|names| names[0] == names[1])
            {
                return Err(ManifestError::DuplicateName {
                    kind: "entity group component",
                    name: names[0].clone(),
                });
            }
            if !unique_groups.insert(group.clone()) {
                return Err(ManifestError::DuplicateEntityGroup(name));
            }
            validated_groups.push(group);
        }
        Ok(SystemDefinition::new(
            name,
            plugin.to_owned(),
            runner,
            validated_groups,
            self.attach,
            self.detach,
        ))
    }
}
