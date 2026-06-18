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
        // Create the symbol
        let Ok(convert_path) = CString::new(path.to_owned()) else {
            return Err(PluginError::InvalidSymbol);
        };

        // Open the library
        let fd: *mut c_void = unsafe { libc::dlopen(convert_path.as_ptr(), libc::RTLD_NOW) };
        if fd.is_null() {
            let error = unsafe {
                let error = libc::dlerror();
                if error.is_null() {
                    "Dynamic loader returned no error message".to_owned()
                } else {
                    CStr::from_ptr(error).to_string_lossy().into_owned()
                }
            };
            return Err(PluginError::LinkingError(error));
        }

        Ok(Self {
            path: Some(path),
            fd,
        })
    }

    pub(crate) fn new_static() -> Self {
        Self {
            path: None,
            fd: libc::RTLD_DEFAULT,
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
}

impl Drop for Plugin {
    fn drop(&mut self) {
        if self.fd.is_null() {
            return;
        }

        unsafe {
            libc::dlclose(self.fd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[unsafe(no_mangle)]
    static TEST_DATA: usize = 5;

    #[test]
    fn test_static_plugin() {
        let plugin = Plugin::new_static();

        assert_eq!(plugin.get_id(), "");

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

        match plugin.get_symbol::<*const usize>("TEST_DATA") {
            Ok(value) => {
                assert!(!value.is_null());
                unsafe {
                    assert_eq!(value.read(), 5);
                }
            }
            Err(_) => {
                panic!("TEST_DATA symbol had an error");
            }
        }
    }
}
