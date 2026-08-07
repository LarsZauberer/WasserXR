//! Versioned top-level plugin descriptor ABI.

use std::{
    collections::{HashMap, HashSet},
    ffi::c_char,
    rc::Rc,
};

use crate::scene::{
    assets::WXRAssetDescriptor,
    component::WXRComponentDescriptor,
    plugin::manifest::{
        ManifestError, ValidatedManifest, copy_name, descriptor_slice, register_definition,
    },
    system::WXRSystemDescriptor,
};

/// Semantic WasserXR version carried by every plugin descriptor.
///
/// While the host major version is zero, plugins must match its major and
/// minor versions. From major version one onward, only the major version must
/// match. Patch versions never affect compatibility.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const CURRENT: Self = Self {
        major: parse_version_component(env!("CARGO_PKG_VERSION_MAJOR")),
        minor: parse_version_component(env!("CARGO_PKG_VERSION_MINOR")),
        patch: parse_version_component(env!("CARGO_PKG_VERSION_PATCH")),
    };

    pub const fn is_compatible(self, plugin: Self) -> bool {
        if self.major == 0 {
            plugin.major == 0 && plugin.minor == self.minor
        } else {
            plugin.major == self.major
        }
    }
}

const fn parse_version_component(value: &str) -> u32 {
    let bytes = value.as_bytes();
    let mut parsed = 0_u32;
    let mut index = 0;
    while index < bytes.len() {
        parsed = parsed * 10 + (bytes[index] - b'0') as u32;
        index += 1;
    }
    parsed
}

pub const WXR_VERSION_MAJOR: u32 = Version::CURRENT.major;
pub const WXR_VERSION_MINOR: u32 = Version::CURRENT.minor;
pub const WXR_VERSION_PATCH: u32 = Version::CURRENT.patch;

/// Static descriptor exported by a dynamic plugin as the global `wxr_plugin`.
///
/// This layout is frozen within one compatibility line: layout changes require
/// a minor bump during `0.x`, and a major bump from `1.0` onward. Every pointer
/// and count uses the canonical pair `(null, 0)` for an empty collection and a
/// non-null pointer with a positive count otherwise.
#[repr(C)]
pub struct WXRPluginDescriptor {
    pub version: Version,
    pub name: *const c_char,
    pub components: *const WXRComponentDescriptor,
    pub component_count: usize,
    pub assets: *const WXRAssetDescriptor,
    pub asset_count: usize,
    pub systems: *const WXRSystemDescriptor,
    pub system_count: usize,
}

// The descriptor is immutable process-lifetime data. Loading its raw pointer
// graph is unsafe and validation copies all data used by the host.
unsafe impl Sync for WXRPluginDescriptor {}

impl WXRPluginDescriptor {
    pub(crate) unsafe fn validate(&self) -> Result<ValidatedManifest, ManifestError> {
        if !Version::CURRENT.is_compatible(self.version) {
            return Err(ManifestError::IncompatibleVersion {
                host: Version::CURRENT,
                plugin: self.version,
            });
        }

        let name = unsafe { copy_name(self.name, "plugin") }?;
        let raw_components =
            unsafe { descriptor_slice(self.components, self.component_count, "components") }?;
        let raw_assets = unsafe { descriptor_slice(self.assets, self.asset_count, "assets") }?;
        let raw_systems = unsafe { descriptor_slice(self.systems, self.system_count, "systems") }?;

        let mut definitions = HashSet::new();
        let mut components = HashMap::with_capacity(raw_components.len());
        for descriptor in raw_components {
            let component = unsafe { descriptor.validate(&name) }?;
            register_definition(&mut definitions, component.get_id())?;
            components.insert(component.get_id().to_owned(), Rc::new(component));
        }

        let mut assets = HashMap::with_capacity(raw_assets.len());
        for descriptor in raw_assets {
            let asset = unsafe { descriptor.validate(&name) }?;
            register_definition(&mut definitions, asset.get_id())?;
            assets.insert(asset.get_id().to_owned(), Rc::new(asset));
        }

        let mut systems = HashMap::with_capacity(raw_systems.len());
        for descriptor in raw_systems {
            let system = unsafe { descriptor.validate(&name) }?;
            register_definition(&mut definitions, system.get_id())?;
            systems.insert(system.get_id().to_owned(), Rc::new(system));
        }

        Ok(ValidatedManifest::new(name, components, assets, systems))
    }
}
