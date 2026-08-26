use std::ffi::{CStr, c_char};

#[derive(Debug, PartialEq, Eq)]
pub enum StringError {
    Null,
    NotUtf8,
    Empty,
}

/// Applies `f` to a valid, non-empty UTF-8 string from a C string pointer.
///
/// # Safety
///
/// If `ptr` is not null, it must point to a readable, nul-terminated C string
/// that remains valid for the duration of this call.
///
/// # Usage
///
/// ```
/// let my_string = std::ffi::CString::new("WasserXR").unwrap();
/// let res = unsafe {wasserxr::utils::ffi::validate_string(my_string.as_ptr(), str::to_owned)};
/// assert!(res.is_ok());
/// assert_eq!(res.unwrap(), "WasserXR");
/// ```
pub unsafe fn validate_string<T>(
    ptr: *const c_char,
    f: impl FnOnce(&str) -> T,
) -> Result<T, StringError> {
    if ptr.is_null() {
        return Err(StringError::Null);
    }

    let string = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| StringError::NotUtf8)?;
    if string.is_empty() {
        return Err(StringError::Empty);
    }

    Ok(f(string))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::null(None::<&[u8]>, Err(StringError::Null))]
    #[case::invalid_utf8(Some(&[0xff, 0][..]), Err(StringError::NotUtf8))]
    #[case::empty(Some(&[0][..]), Err(StringError::Empty))]
    fn rejects_invalid_strings(
        #[case] bytes: Option<&[u8]>,
        #[case] expected: Result<(), StringError>,
    ) {
        let ptr = bytes.map_or(std::ptr::null(), |bytes| bytes.as_ptr().cast());
        assert_eq!(unsafe { validate_string(ptr, |_| ()) }, expected);
    }

    #[rstest]
    #[case(b"plugin\0", "plugin")]
    #[case(b"field\0", "field")]
    fn accepts_valid_strings(#[case] bytes: &[u8], #[case] expected: &str) {
        let result = unsafe { validate_string(bytes.as_ptr().cast(), str::to_owned) };
        assert_eq!(result, Ok(expected.to_owned()));
    }
}
