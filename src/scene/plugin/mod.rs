use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
};

use crate::error::PluginError;

pub(crate) struct Plugin {
    path: Option<String>,
    fd: *mut c_void,
}

impl Plugin {
    pub(crate) fn new(path: String) -> Result<Plugin, PluginError> {
        let path_cstring = Self::create_c_string(path.clone())?;

        // Open the library
        let fd: *mut c_void = unsafe { libc::dlopen(path_cstring.as_ptr(), libc::RTLD_NOW) };
        if fd.is_null() {
            let error = unsafe {
                let error = libc::dlerror();
                if error.is_null() {
                    "Dynamic loader returned no error message".to_owned()
                } else {
                    CStr::from_ptr(error).to_string_lossy().into_owned()
                }
            };
            log::error!("Plugin `{}` could not be loaded: {}", path, error);
            return Err(PluginError::LinkingError(error));
        }

        log::info!("Plugin `{}` opened", path);
        Ok(Self {
            path: Some(path),
            fd,
        })
    }

    pub(crate) fn new_static() -> Self {
        log::info!("Static plugin initialized");
        Self {
            path: None,
            fd: libc::RTLD_DEFAULT,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_test_dynamic(path: String) -> Self {
        Self {
            path: Some(path),
            fd: std::ptr::null_mut(),
        }
    }

    pub(crate) fn get_symbol<T>(&self, symbol: &str) -> Result<T, PluginError> {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());

        let Ok(symbol_cstring) = CString::new(symbol.to_owned()) else {
            log::error!("Symbol name contains a null byte");
            return Err(PluginError::InvalidSymbol);
        };

        // Safety: Will return either null or will return the function pointer
        let ptr = unsafe { libc::dlsym(self.fd, symbol_cstring.as_ptr()) };
        if ptr.is_null() {
            log::debug!("Symbol `{}` was not found", symbol);
            return Err(PluginError::MissingSymbol(symbol.to_owned()));
        }
        let func: T = unsafe { std::mem::transmute_copy(&ptr) };
        log::debug!("Symbol `{}` resolved", symbol);
        Ok(func)
    }

    pub(crate) fn get_id(&self) -> &str {
        match &self.path {
            Some(path) => path,
            None => "",
        }
    }

    fn create_c_string(data: String) -> Result<CString, PluginError> {
        CString::new(data).map_err(|_| {
            log::error!("Plugin path contains a null byte");
            PluginError::InvalidSymbol
        })
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        if self.fd.is_null() {
            return;
        }

        let Some(path) = &self.path else {
            return;
        };

        unsafe {
            libc::dlclose(self.fd);
        }

        // Check if it is really unloaded (dlclose doesn't necessarily unload the library)
        let Ok(path_cstring) = Self::create_c_string(path.clone()) else {
            log::warn!(
                "Failed to check if the plugin `{}` is truely unloaded",
                path
            );
            return;
        };
        unsafe {
            let still_loaded: *mut c_void =
                libc::dlopen(path_cstring.as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD);

            if !still_loaded.is_null() {
                // Failed to unload
                // Still has to close the new handle
                libc::dlclose(still_loaded);
                log::warn!(
                    "Plugin `{}` failed to be unloaded. It is still in kernel memory. You cannot load old systems, components should have been removed and new systems and components will not be loaded from this plugin. Still threads spawned by the plugin could still be running. Furthermore, when you load the plugin again, it will not load a new version. Make sure that at the end of a plugin lifetime no threads are running anymore.",
                    path
                );
            } else {
                log::info!("Plugin `{}` closed", path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[unsafe(no_mangle)]
    static TEST_DATA: usize = 5;

    #[test]
    fn plugin_new_static() {
        let plugin = Plugin::new_static();

        assert_eq!(plugin.get_id(), "");
    }

    #[test]
    fn plugin_get_symbol_for_existing_symbol() {
        let plugin = Plugin::new_static();

        let value = plugin.get_symbol::<*const usize>("TEST_DATA").unwrap();

        assert!(!value.is_null());
        unsafe {
            assert_eq!(value.read(), 5);
        }
    }

    #[test]
    fn plugin_get_symbol_for_missing_symbol() {
        let plugin = Plugin::new_static();

        match plugin.get_symbol::<*const usize>("nonexistent") {
            Ok(_) => {
                panic!("Nonexistent symbol should have not been able to be found");
            }
            Err(PluginError::MissingSymbol(symbol)) => {
                assert_eq!(symbol, "nonexistent");
            }
            Err(_) => {
                panic!("Nonexistent symbol had an error that was not a MissingSymbol error");
            }
        }
    }

    #[test]
    fn plugin_get_symbol_for_invalid_symbol() {
        let plugin = Plugin::new_static();

        assert_eq!(
            plugin.get_symbol::<*const usize>("invalid\0symbol"),
            Err(PluginError::InvalidSymbol)
        );
    }

    #[test]
    fn plugin_new_for_missing_path() {
        let result = Plugin::new("/definitely/missing/wasserxr/test/plugin.so".to_owned());

        assert!(matches!(result, Err(PluginError::LinkingError(_))));
    }
}
