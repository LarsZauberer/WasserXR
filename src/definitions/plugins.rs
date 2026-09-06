//! This is the standard definition of a plugin.

use std::ffi::c_char;

use crate::{
    definitions::{
        Definition, assets::AssetDefinition, components::ComponentDefinition,
        error::PluginDefinitionError,
    },
    utils::ffi::validate_string,
    utils::version::Version,
};

/// The plugin definition defines a plugin for WasserXR. It contains all the raw
/// function pointers for the components, systems and other ECS objects that are
/// described in the plugin. It is the master struct containing all the
/// definitions for the plugin.
///
/// The plugin definition should be statically and globally written out. The
/// global plugin definition variable **requires** to be named `wxr_plugin`. Per
/// plugin, only one single plugin definition may exist.
///
/// ## Datagram
///
/// The plugin definition is always constructed in the same way
///
/// - `name` as a globally defined static string pointer
/// - `engine_version` a [`Version`] of WasserXR with which it was built.
///
/// The component array uses a C-compatible pointer/count pair. The pointer must
/// remain valid for the lifetime of the plugin definition.
/// The asset array follows the same convention.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PluginDefinition {
    pub name: *const c_char,
    pub engine_version: Version,

    pub components: *const ComponentDefinition,
    pub component_count: usize,

    pub assets: *const AssetDefinition,
    pub asset_count: usize,
}

impl Definition for PluginDefinition {
    type Error = PluginDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        // Get the current WasserXR version
        let current_version = Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or_default(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or_default(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or_default(),
        };

        // Check WasserXR version compatibility
        // If the major is 0 (meaning that the API is unstable) then also the minor
        // version has to match.
        // Otherwise, only the major version has to match
        let compatible = if current_version.major == 0 {
            self.engine_version.major == 0 && self.engine_version.minor == current_version.minor
        } else {
            self.engine_version.major == current_version.major
        };
        if !compatible {
            return Err(PluginDefinitionError::EngineVersionMismatch {
                name: name.clone(),
                expected: current_version,
                actual: self.engine_version,
            });
        }

        // Generate the component slice
        let components = if self.component_count == 0 {
            &[]
        } else {
            if self.components.is_null() {
                return Err(PluginDefinitionError::ComponentsIsNull(name.clone()));
            }
            unsafe { std::slice::from_raw_parts(self.components, self.component_count) }
        };

        for component in components {
            if let Err(error) = unsafe { component.validate() } {
                return Err((name.clone(), error).into());
            }
        }

        let assets = if self.asset_count == 0 {
            &[]
        } else {
            if self.assets.is_null() {
                return Err(PluginDefinitionError::AssetsIsNull(name.clone()));
            }
            unsafe { std::slice::from_raw_parts(self.assets, self.asset_count) }
        };

        for asset in assets {
            if let Err(error) = unsafe { asset.validate() } {
                return Err((name.clone(), error).into());
            }
        }

        Ok(())
    }
}

impl PluginDefinition {
    /// Returns the validated plugin name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, PluginDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
