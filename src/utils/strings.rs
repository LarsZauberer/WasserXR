use std::ffi::{CString, NulError};

/// Converts a Rust string into an owned C-ABI compatible string.
///
/// The returned [`CString`] is NUL-terminated and contains no interior NUL
/// bytes. Use [`CString::as_ptr`] when C only borrows the string for the
/// duration of a call. Use [`CString::into_raw`] only when transferring
/// ownership; the raw pointer must later be released with [`CString::from_raw`]
/// or an equivalent WasserXR free function.
pub fn to_c_abi_string(value: impl AsRef<str>) -> Result<CString, NulError> {
    CString::new(value.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn converts_str_reference() {
        let value = to_c_abi_string("models/cube.glb").unwrap();

        assert_eq!(value.as_bytes_with_nul(), b"models/cube.glb\0");
    }

    #[test]
    fn converts_owned_string() {
        let model = String::from("models/sphere.glb");
        let value = to_c_abi_string(model).unwrap();

        assert_eq!(value.to_str().unwrap(), "models/sphere.glb");
    }

    #[test]
    fn rejects_interior_nul_bytes() {
        assert!(to_c_abi_string("models\0cube.glb").is_err());
    }

    #[test]
    fn raw_pointer_round_trips_when_ownership_is_transferred() {
        let value = to_c_abi_string("models/capsule.glb").unwrap();
        let ptr = value.into_raw();

        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), "models/capsule.glb");
            drop(CString::from_raw(ptr));
        }
    }
}
