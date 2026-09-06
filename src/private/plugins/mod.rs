//! This module describes everything about the actual active representation of
//! plugins

use std::{
    ffi::{CStr, CString},
    path::Path,
};

use libc::{RTLD_LAZY, dlopen, dlsym};
use uuid::Uuid;

use crate::{
    definitions::plugins::PluginDefinition,
    errors::PluginError,
    private::manifests::{
        Manifest, assets::AssetManifest, components::ComponentManifest, plugins::PluginManifest,
    },
};

pub(crate) mod error;

const WXR_PLUGIN_SYMBOL_NAME: &CStr = c"wxr_plugin";

/// This trait defines what operations an active plugin that handles all the I/O
/// needs to satisfy
#[derive(Debug)]
pub(crate) struct Plugin {
    manifest: PluginManifest,
}

impl Plugin {
    /// Loads a dynamic shared object library and tries to tires to aquire the
    /// [`PluginDefinition`] which will then be
    /// turned into a [`PluginManifest`] and stored.
    ///
    /// # Safety
    ///
    /// The caller is required to ensure that the plugin that they are trying to
    /// load has a public variable that is of type [`PluginDefinition`] and
    /// therefore, implements the correct data structure format. Furthermore,
    /// the caller is required, that the PluginDefinition is memory safe and
    /// valid.
    pub(crate) unsafe fn load_shared(path: &Path) -> Result<Self, PluginError> {
        // Generate a unique and new path for the loaded plugin and copy it into the
        // OS's tmp directory.
        //
        // This is a deliberate choice, since the kernel will never freshly open the
        // with the `dlopen` call a plugin that is already open with the same
        // name. When we want to open the same file (which might have changed by
        // the user in the meantime), we need to still be able to open the new
        // plugin file. Hence, we trick the kernel into thinking this is a new
        // file by giving it a new name.
        let copied_path = std::env::temp_dir().join(Path::new(&Uuid::new_v4().to_string()));
        std::fs::copy(path, &copied_path).map_err(PluginError::from)?;

        // Load the plugin.
        //
        // You may notice that the plugin is kept open intentionally. There is no close
        // call. We do this deliberately here, since plugins could perform some
        // thread operations that would anyway prevent the kernel from closing
        // the plugin. Furthermore, there might be some data around that has
        // been allocated at some point but needs to be deallocated at some point
        // later. To ensure that this information stays, we never unload the plugin, so
        // that still old code can be called.
        let filename = CString::new(
            copied_path
                .to_str()
                .expect("Filepath needs to be a valid string")
                .to_owned(),
        )
        .expect("Filepath needs to be a valid CString");
        let handle = unsafe { dlopen(filename.as_ptr(), RTLD_LAZY) };
        if handle.is_null() {
            return Err(PluginError::FailedToOpenPlugin);
        }

        let symbol = unsafe { dlsym(handle, WXR_PLUGIN_SYMBOL_NAME.as_ptr()) };
        if symbol.is_null() {
            return Err(PluginError::FailedToFindPluginDefinition);
        }

        // Cast the null ptr to the PluginDefinition (highly unsafe, since there
        // are no type guarantees). The caller is required to ensure that the
        // plugin definitions are of the correct type and use therefore the correct data
        // structure.
        let definition = unsafe { symbol.cast::<PluginDefinition>().read() };
        let manifest =
            unsafe { Manifest::checked_convert(definition) }.map_err(PluginError::from)?;

        Ok(Self::load_static(manifest))
    }

    /// Loads a given [`PluginManifest`] directly and turns it into an active
    /// plugin.
    pub(crate) fn load_static(manifest: PluginManifest) -> Self {
        Self { manifest }
    }

    /// Returns the name of the plugin
    pub(crate) fn get_name(&self) -> &str {
        &self.manifest.name
    }

    /// Searches in the current [`PluginManifest`] for the defined components
    /// and tries to find the requested [`ComponentManifest`]
    pub(crate) fn get_component(&self, name: &str) -> Option<&ComponentManifest> {
        self.manifest.components.get(name)
    }

    /// Searches in the current [`PluginManifest`] for the defined assets and
    /// tries to find the requested [`AssetManifest`].
    pub(crate) fn get_asset(&self, name: &str) -> Option<&AssetManifest> {
        self.manifest.assets.get(name)
    }
}
