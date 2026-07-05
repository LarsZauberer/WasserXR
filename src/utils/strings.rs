use std::{
    ffi::{CString, c_char, c_void},
    ptr,
};

/// Converts a raw WasserXR string field pointer to an owned C string.
///
/// This is intended for string fields returned by `wxr_query`, where the raw
/// pointer still points at a live Rust String. The returned pointer must be
/// released with `wxr_free_string`.
///
/// # Safety
///
/// If `value` is not null, it must be properly aligned and point to a valid
/// live Rust String for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_rust_string_to_c_string(value: *const c_void) -> *mut c_char {
    if value.is_null() {
        return ptr::null_mut();
    }

    match CString::new(unsafe { &*value.cast::<String>() }.as_str()) {
        Ok(value) => value.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn converts_rust_string_pointer() {
        let model = String::from("models/cube.glb");
        let ptr = unsafe { wxr_rust_string_to_c_string((&model as *const String).cast()) };

        unsafe {
            assert_eq!(
                CStr::from_ptr(ptr).to_bytes_with_nul(),
                b"models/cube.glb\0"
            );
            drop(CString::from_raw(ptr));
        }
    }

    #[test]
    fn rejects_interior_nul_bytes() {
        let model = String::from("models\0cube.glb");

        assert!(unsafe { wxr_rust_string_to_c_string((&model as *const String).cast()) }.is_null());
    }

    #[test]
    fn null_pointer_returns_null() {
        assert!(unsafe { wxr_rust_string_to_c_string(ptr::null()) }.is_null());
    }

    #[test]
    fn raw_pointer_round_trips_when_ownership_is_transferred() {
        let model = String::from("models/capsule.glb");
        let ptr = unsafe { wxr_rust_string_to_c_string((&model as *const String).cast()) };

        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), "models/capsule.glb");
            drop(CString::from_raw(ptr));
        }
    }
}
