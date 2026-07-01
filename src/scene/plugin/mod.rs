#[cfg(test)]
use std::path::PathBuf;
use std::{
    ffi::{CStr, CString},
    io::ErrorKind,
    os::raw::c_void,
};

use uuid::Uuid;

use crate::error::PluginError;

pub(crate) struct Plugin {
    path: Option<String>,
    #[cfg(test)]
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
            #[cfg(test)]
            fd_file_path: Some(fd_file_path),
            fd,
        })
    }

    pub(crate) fn new_static() -> Self {
        Self {
            path: None,
            #[cfg(test)]
            fd_file_path: None,
            fd: libc::RTLD_DEFAULT,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_test_dynamic(path: String) -> Self {
        Self {
            path: Some(path),
            #[cfg(test)]
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

        assert!(matches!(result, Err(PluginError::NotFound)));
    }
}
