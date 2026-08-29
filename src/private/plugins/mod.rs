//! This module describes everything about the actual active representation of plugins

use std::path::Path;

use crate::{
    errors::PluginError,
    private::{
        io::{FileIO, PluginIO},
        manifests::{Manifest, components::ComponentManifest, plugins::PluginManifest},
    },
};

pub(crate) mod error;

#[cfg(test)]
mod tests;

/// This trait defines what operations an active plugin that handles all the I/O needs to satisfy
pub(crate) struct Plugin {
    manifest: PluginManifest,
}

impl Plugin {
    /// Loads a dynamic shared object library and tries to tires to aquire the
    /// [`wasserxr::definitions::plugins::PluginDefinition`] which will then be turned into a
    /// [`PluginManifest`] and stored.
    ///
    /// # Safety
    ///
    /// TODO: Add the safety text
    pub(crate) unsafe fn load_shared<T>(path: &Path) -> Result<Self, PluginError>
    where
        T: FileIO + PluginIO,
        <T as FileIO>::Error: Into<PluginError>,
        <T as PluginIO>::Error: Into<PluginError>,
    {
        let copied_path = std::env::temp_dir().join(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("wasserxr-plugin")),
        );
        T::copy(path, &copied_path).map_err(|error| -> PluginError { error.into() })?;

        let plugin_definition = unsafe { T::get_plugin_definition(&copied_path) }
            .map_err(|error| -> PluginError { error.into() })?;
        let manifest = unsafe { PluginManifest::checked_convert(plugin_definition) }
            .map_err(PluginError::from)?;

        Ok(Self::load_static(manifest))
    }

    /// Loads a given [`PluginManifest`] directly and turns it into an active plugin.
    pub(crate) fn load_static(manifest: PluginManifest) -> Self {
        Self { manifest }
    }

    /// Searches in the current [`PluginManifest`] for the defined components and tries to find the
    /// requested [`ComponentManifest`]
    pub(crate) fn get_component(&self, name: &str) -> Option<ComponentManifest> {
        todo!()
    }
}
