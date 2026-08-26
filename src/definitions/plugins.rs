//! This is the standard definition of a plugin.

use std::ffi::c_char;

use crate::{
    definitions::{Definition, components::ComponentDefinition, error::PluginDefinitionError},
    utils::ffi::validate_string,
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
///
/// The component array uses a C-compatible pointer/count pair. The pointer must remain valid for
/// the lifetime of the plugin definition.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PluginDefinition {
    name: *const c_char,
    engine_version: Version,

    components: *const ComponentDefinition,
    component_count: usize,
}

impl Definition for PluginDefinition {
    type Error = PluginDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        // Get the current WasserXR version
        let current_version = Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or_default(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or_default(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or_default(),
        };

        // Check WasserXR version compatibility
        // If the major is 0 (meaning that the API is unstable) then also the minor version has to
        // match.
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
                return Err((name, error).into());
            }
        }

        Ok(())
    }
}

impl PluginDefinition {
    unsafe fn name(&self) -> Result<String, PluginDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn validates_plugin_name_and_engine_version() {
        let name = CString::new("example").unwrap();
        let compatible_version = Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        };
        let plugin = PluginDefinition {
            name: name.as_ptr(),
            engine_version: compatible_version,
            components: std::ptr::null(),
            component_count: 0,
        };
        assert!(unsafe { plugin.validate() }.is_ok());

        let incompatible_version = if compatible_version.major == 0 {
            Version {
                minor: compatible_version.minor + 1,
                ..compatible_version
            }
        } else {
            Version {
                major: compatible_version.major + 1,
                ..compatible_version
            }
        };
        let plugin = PluginDefinition {
            engine_version: incompatible_version,
            ..plugin
        };
        assert_eq!(
            unsafe { plugin.validate() },
            Err(PluginDefinitionError::EngineVersionMismatch {
                name: "example".to_owned(),
                expected: compatible_version,
                actual: incompatible_version,
            })
        );

        let plugin = PluginDefinition {
            engine_version: compatible_version,
            components: std::ptr::null(),
            component_count: 1,
            ..plugin
        };
        assert_eq!(
            unsafe { plugin.validate() },
            Err(PluginDefinitionError::ComponentsIsNull(
                "example".to_owned()
            ))
        );
    }
}
