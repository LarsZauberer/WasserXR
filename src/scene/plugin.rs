use core::{ffi::c_void, ptr::null_mut};
use std::ffi::CString;

pub struct Plugin {
    path: String,
    fd: *mut c_void,
}

impl Plugin {
    pub fn new(path: &str) -> Option<Self> {
        // Create the symbol
        let convert_path = CString::new(path.to_owned());
        if convert_path.is_err() {
            return None;
        }
        let convert_path = convert_path.unwrap();

        // Open the library
        let fd: *mut c_void = unsafe { libc::dlopen(convert_path.as_ptr(), libc::RTLD_NOW) };
        if fd.is_null() {
            return None;
        }

        Some(Plugin {
            path: path.to_owned(),
            fd: fd,
        })
    }

    pub fn get_abi_symbol_from_plugin<T>(&self, symbol: &str) -> Option<T> {
        Self::get_abi_symbol_from_fd(self.fd, symbol)
    }

    pub fn get_abi_symbol_from_static<T>(symbol: &str) -> Option<T> {
        Self::get_abi_symbol_from_fd(libc::RTLD_DEFAULT, symbol)
    }

    fn get_abi_symbol_from_fd<T>(handler: *mut c_void, symbol: &str) -> Option<T> {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());
        let symbol_cstring = CString::new(symbol.to_owned());
        if symbol_cstring.is_err() {
            return None;
        }
        let symbol_cstring = symbol_cstring.unwrap();

        // Safety: Will return either null or will return the function pointer
        let ptr = unsafe { libc::dlsym(handler, symbol_cstring.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        let func: T = unsafe { std::mem::transmute_copy(&ptr) };
        Some(func)
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
        self.fd = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn my_func(my_num: usize) -> usize {
        my_num * 5
    }

    type TestFunction = fn(usize) -> usize;

    #[test]
    fn load_static_plugin() {
        let func: Option<TestFunction> = Plugin::get_abi_symbol_from_static("my_func");
        assert!(func.is_some());

        let func: unsafe extern "C" fn(usize) -> usize = unsafe { std::mem::transmute(func) };
        let res = unsafe { func(5) };

        assert_eq!(res, 25);
    }
}
