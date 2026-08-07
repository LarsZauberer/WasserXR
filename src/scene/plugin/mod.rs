//! Dynamic-library origins and the versioned plugin descriptor ABI.

use std::{
    ffi::{CStr, CString, c_void},
    ptr,
    rc::Rc,
};

use uuid::Uuid;

use crate::scene::{assets::AssetType, component::ComponentDefinition, system::SystemDefinition};

/// Versioned top-level plugin descriptor ABI.
pub mod descriptor;
mod error;
pub(crate) mod manifest;

pub use descriptor::{
    Version, WXR_VERSION_MAJOR, WXR_VERSION_MINOR, WXR_VERSION_PATCH, WXRPluginDescriptor,
};
pub use error::PluginError;
pub use manifest::ManifestError;
use manifest::ValidatedManifest;

pub(crate) struct Plugin {
    manifest: ValidatedManifest,
    source_path: Option<String>,
    // Deliberately never passed to dlclose. Plugin code remains resident.
    _fd: *mut c_void,
}

impl Plugin {
    pub(crate) unsafe fn load_dynamic(path: String) -> Result<Self, PluginError> {
        let copied_path = std::env::temp_dir().join(Uuid::now_v7().to_string());
        if let Err(error) = std::fs::copy(&path, &copied_path) {
            let _ = std::fs::remove_file(&copied_path);
            return Err(PluginError::LoadIo(error));
        }
        let copied_path_c = match CString::new(copied_path.to_string_lossy().as_bytes()) {
            Ok(path) => path,
            Err(error) => {
                let _ = std::fs::remove_file(&copied_path);
                return Err(PluginError::InvalidPath(error));
            }
        };
        let fd = unsafe { libc::dlopen(copied_path_c.as_ptr(), libc::RTLD_NOW) };
        let linking_error = fd.is_null().then(loader_error);
        // dlopen has established the resident mapping, so the temporary copy is
        // no longer needed even when subsequent validation rejects the plugin.
        let _ = std::fs::remove_file(&copied_path);
        if fd.is_null() {
            return Err(PluginError::Linking(
                linking_error.expect("a null dlopen handle must have an error"),
            ));
        }

        // This is the only dlsym lookup performed for a plugin.
        let descriptor =
            unsafe { libc::dlsym(fd, c"wxr_plugin".as_ptr()) }.cast::<WXRPluginDescriptor>();
        if descriptor.is_null() {
            return Err(PluginError::MissingManifestSymbol);
        }
        let manifest = unsafe { ValidatedManifest::from_descriptor(descriptor) }?;
        Ok(Self {
            manifest,
            source_path: Some(path),
            _fd: fd,
        })
    }

    pub(crate) unsafe fn load_static(
        descriptor: &'static WXRPluginDescriptor,
    ) -> Result<Self, PluginError> {
        let manifest = unsafe { ValidatedManifest::from_descriptor(descriptor) }?;
        Ok(Self {
            manifest,
            source_path: None,
            _fd: ptr::null_mut(),
        })
    }

    pub(crate) fn get_id(&self) -> &str {
        self.manifest.get_id()
    }

    pub(crate) fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    pub(crate) fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.manifest.definition_names()
    }

    pub(crate) fn has_definition(&self, id: &str) -> bool {
        self.manifest.definition_names().any(|name| name == id)
    }

    pub(crate) fn component_definition(&self, id: &str) -> Option<Rc<ComponentDefinition>> {
        self.manifest.component(id)
    }

    pub(crate) fn asset_definition(&self, id: &str) -> Option<Rc<AssetType>> {
        self.manifest.asset(id)
    }

    pub(crate) fn system_definition(&self, id: &str) -> Option<Rc<SystemDefinition>> {
        self.manifest.system(id)
    }

    pub(crate) fn asset_names(&self) -> impl Iterator<Item = &str> {
        self.manifest.asset_names()
    }

    pub(crate) fn is_consistent(&self) -> bool {
        self.manifest.is_consistent()
    }
}

fn loader_error() -> String {
    unsafe {
        let error = libc::dlerror();
        if error.is_null() {
            "dynamic loader returned no error message".to_owned()
        } else {
            CStr::from_ptr(error).to_string_lossy().into_owned()
        }
    }
}
