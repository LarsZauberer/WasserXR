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
    use rstest::rstest;
    use std::ffi::CStr;

    #[rstest]
    #[case("models/cube.glb")]
    #[case("models/capsule.glb")]
    fn converts_and_transfers_ownership_of_rust_string(#[case] model: &str) {
        let model = model.to_owned();
        let ptr = unsafe { wxr_rust_string_to_c_string((&model as *const String).cast()) };

        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), model);
            // Ownership was transferred to the caller, so we must free it here.
            drop(CString::from_raw(ptr));
        }
    }

    #[rstest]
    fn rejects_interior_nul_bytes() {
        let model = String::from("models\0cube.glb");

        assert!(unsafe { wxr_rust_string_to_c_string((&model as *const String).cast()) }.is_null());
    }

    #[rstest]
    fn null_pointer_returns_null() {
        assert!(unsafe { wxr_rust_string_to_c_string(ptr::null()) }.is_null());
    }
}
