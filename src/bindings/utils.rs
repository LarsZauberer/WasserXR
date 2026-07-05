use std::{ffi::c_char, ptr};

use crate::{
    bindings::{WXRSceneError, clear_error, set_error, str_from_ptr, string_to_ptr},
    utils::paths::get_asset_path,
};

/// Resolves an asset path and returns the absolute path as an owned C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_asset_path(path: *const c_char) -> *mut c_char {
    match unsafe { str_from_ptr(path) } {
        Ok(path) => match get_asset_path(path) {
            Some(path) => {
                clear_error();
                string_to_ptr(path.to_string_lossy().into_owned())
            }
            None => {
                set_error(WXRSceneError::FileIo);
                ptr::null_mut()
            }
        },
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn get_asset_path_returns_string() {
        let path = CString::new("Cargo.toml").unwrap();
        let resolved = unsafe { wxr_get_asset_path(path.as_ptr()) };

        assert!(!resolved.is_null());
        assert!(
            unsafe { CStr::from_ptr(resolved) }
                .to_str()
                .unwrap()
                .ends_with("Cargo.toml")
        );

        unsafe {
            crate::bindings::wxr_free_string(resolved);
        }
    }
}
