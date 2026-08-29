//! Mockable boundaries for the I/O operations used while loading plugins.
//!
//! Production implementations perform the real filesystem and dynamic-library operations, while
//! tests can replace them with controlled implementations.

use std::{error::Error, path::Path};

use crate::{definitions::plugins::PluginDefinition, errors::PluginError};

pub(crate) mod filesystem;

/// Plugin IO operations like acquiring the [`PluginDefinition`] from some path (e.g. shared object
/// file).
pub(crate) trait PluginIO {
    /// An error encountered while opening the library or resolving its plugin definition.
    type Error: Error;

    /// Opens the plugin at `src` and returns its exported definition.
    unsafe fn get_plugin_definition(src: &Path) -> Result<PluginDefinition, Self::Error>;
}

/// Filesystem operations required while preparing a plugin for loading.
pub(crate) trait FileIO {
    /// An error encountered during a filesystem operation.
    type Error: Error;

    /// Copies the file at `src` to `dst`.
    fn copy(src: &Path, dst: &Path) -> Result<(), Self::Error>;
}
