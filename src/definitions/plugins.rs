//! This is the standard definition of a plugin.

use std::ffi::c_char;

use crate::{
    definitions::{Definition, components::ComponentDefinition, error::PluginDefinitionError},
    utils::version::Version,
};

/// The plugin definition defines a plugin for WasserXR. It contains all the raw function pointers
/// for the components, systems and other ECS objects that are described in the plugin. It is the
/// master struct containing all the definitions for the plugin.
///
/// The plugin definition should be statically and globally written out. The global plugin
/// definition variable **requires** to be named `wxr_plugin`. Per plugin, only one single plugin
/// definition may exist.
///
/// ## Datagram
///
/// The plugin definition is always constructed in the same way
///
/// - `name` as a globally defined static string pointer
/// - `engine_version` a [`wasserxr::utils::Version`] of WasserXR with which it was built.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PluginDefinition {
    name: *const c_char,
    engine_version: Version,

    components: &'static [ComponentDefinition],
}

impl Definition for PluginDefinition {
    type Error = PluginDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        // TODO: Check that the name is not null

        // TODO: Check that the name is valid

        // TODO: Convert the current crate version into a wasserxr::utils::Version

        // TODO: If the versions major number of the crate is `0`, then only plugin definitions with
        // also major number 0 and equal minor numbers are allowed. Otherwise, the major numbers
        // need to match.

        // TODO: Create for all of the previous errors a DefinitionError
        todo!()
    }
}
