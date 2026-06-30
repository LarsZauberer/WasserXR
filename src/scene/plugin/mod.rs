use std::{
    ffi::{CStr, CString},
    io::ErrorKind,
    os::raw::c_void,
    path::PathBuf,
};

use uuid::Uuid;

use crate::error::PluginError;

pub(crate) struct Plugin {
    path: Option<String>,
    fd_file_path: Option<PathBuf>,
    fd: *mut c_void,
}

impl Plugin {
    pub(crate) fn new(path: String) -> Result<Plugin, PluginError> {
        let fd_file_path = std::env::temp_dir().join(Uuid::now_v7().to_string());
        if let Err(error) = std::fs::copy(&path, &fd_file_path) {
            return match error.kind() {
                ErrorKind::NotFound => Err(PluginError::NotFound),
                _ => Err(PluginError::LinkingError(error.to_string())),
            };
        }

        let path_cstring = Self::create_c_string(fd_file_path.to_string_lossy().into_owned())?;

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
            let _ = std::fs::remove_file(fd_file_path);
            return Err(PluginError::LinkingError(error));
        }

        Ok(Self {
            path: Some(path),
            fd_file_path: Some(fd_file_path),
            fd,
        })
    }

    pub(crate) fn new_static() -> Self {
        Self {
            path: None,
            fd_file_path: None,
            fd: libc::RTLD_DEFAULT,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_test_dynamic(path: String) -> Self {
        Self {
            path: Some(path),
            fd_file_path: None,
            fd: std::ptr::null_mut(),
        }
    }

    pub(crate) fn get_symbol<T>(&self, symbol: &str) -> Result<T, PluginError> {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());

        let Ok(symbol_cstring) = CString::new(symbol.to_owned()) else {
            return Err(PluginError::InvalidSymbol);
        };

        // Safety: Will return either null or will return the function pointer
        let ptr = unsafe { libc::dlsym(self.fd, symbol_cstring.as_ptr()) };
        if ptr.is_null() {
            return Err(PluginError::MissingSymbol(symbol.to_owned()));
        }
        let func: T = unsafe { std::mem::transmute_copy(&ptr) };
        Ok(func)
    }

    pub(crate) fn get_id(&self) -> &str {
        match &self.path {
            Some(path) => path,
            None => "",
        }
    }

    fn create_c_string(data: String) -> Result<CString, PluginError> {
        CString::new(data).map_err(|_| PluginError::InvalidSymbol)
    }

    pub(crate) fn close(&mut self) -> Result<bool, PluginError> {
        if self.fd.is_null() {
            return Ok(true);
        }

        let Some(fd_file_path) = self.fd_file_path.take() else {
            return Ok(true);
        };

        unsafe {
            libc::dlclose(self.fd);
        }
        self.fd = std::ptr::null_mut();

        // Check if it is really unloaded (dlclose doesn't necessarily unload the library)
        let path_cstring = Self::create_c_string(fd_file_path.to_string_lossy().into_owned())?;
        unsafe {
            let still_loaded: *mut c_void =
                libc::dlopen(path_cstring.as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD);

            if !still_loaded.is_null() {
                // Failed to unload
                // Still has to close the new handle
                libc::dlclose(still_loaded);
                let _ = std::fs::remove_file(fd_file_path);
                Ok(false)
            } else {
                let _ = std::fs::remove_file(fd_file_path);
                Ok(true)
            }
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[unsafe(no_mangle)]
    static TEST_DATA: usize = 5;

    fn plugin_library_path() -> String {
        let deps_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let prefix = format!("{}wasserxr_macros-", std::env::consts::DLL_PREFIX);
        let suffix = format!(".{}", std::env::consts::DLL_EXTENSION);

        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&deps_dir)
            .unwrap()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let file_name = path.file_name()?.to_str()?;
                if file_name.starts_with(&prefix) && file_name.ends_with(&suffix) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        candidates.sort();

        candidates
            .pop()
            .unwrap_or_else(|| panic!("missing test plugin library in {}", deps_dir.display()))
            .to_string_lossy()
            .into_owned()
    }

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

        assert!(matches!(result, Err(PluginError::NotFound)));
    }

    #[test]
    fn plugin_new_loads_unique_temp_file() {
        let path = plugin_library_path();

        let first = Plugin::new(path.clone()).unwrap();
        let second = Plugin::new(path.clone()).unwrap();

        assert_eq!(first.get_id(), path);
        assert_ne!(first.fd_file_path, second.fd_file_path);
        assert!(first.fd_file_path.as_ref().unwrap().exists());
        assert!(second.fd_file_path.as_ref().unwrap().exists());
    }

    #[test]
    fn plugin_new_reloads_when_previous_copy_stays_loaded_by_thread() {
        let path = plugin_library_path();
        let mut plugin = Plugin::new(path.clone()).unwrap();
        let first_fd_file_path = plugin.fd_file_path.clone().unwrap();
        let thread_path = first_fd_file_path.clone();
        let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
        let (release_sender, release_receiver) = std::sync::mpsc::channel();

        let thread = std::thread::spawn(move || {
            let path_cstring = CString::new(thread_path.to_string_lossy().into_owned()).unwrap();
            let fd: *mut c_void = unsafe { libc::dlopen(path_cstring.as_ptr(), libc::RTLD_NOW) };
            let loaded = !fd.is_null();
            ready_sender.send(loaded).unwrap();

            if loaded {
                let _ = release_receiver.recv();
                unsafe {
                    libc::dlclose(fd);
                }
            }
        });

        let thread_loaded_copy = ready_receiver.recv().unwrap();
        let close_result = plugin.close();
        let reloaded = Plugin::new(path);
        let reloaded_fd_file_path = reloaded
            .as_ref()
            .unwrap()
            .fd_file_path
            .as_ref()
            .unwrap()
            .clone();
        let _ = release_sender.send(());
        thread.join().unwrap();

        assert!(thread_loaded_copy);
        assert_eq!(close_result, Ok(false));
        assert_ne!(first_fd_file_path, reloaded_fd_file_path);
    }
}
