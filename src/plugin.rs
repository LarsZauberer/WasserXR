use std::os::raw::c_void;

use crate::error::WXRError;

pub struct Plugin {
    path: Option<String>,
    fd: *mut c_void,
}

impl Plugin {
    pub fn new(path: String) -> Result<Plugin, WXRError> {
        todo!();
    }

    pub fn new_static() -> Self {
        Self {
            path: None,
            fd: libc::RTLD_DEFAULT,
        }
    }

    pub fn get_symbol<T>(symbol: &str) -> Result<T, WXRError> {
        todo!();
    }

    pub fn destroy(self) {
        todo!();
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
