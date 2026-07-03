#[repr(C)]
/// Owned byte buffer passed over WasserXR's C ABI.
///
/// `SerializedBytes` stores a leaked `Vec<u8>` as raw parts so generated
/// serializers and deserializers can exchange bytes across an extern function
/// boundary.
pub struct SerializedBytes {
    /// Pointer to the first byte.
    pub ptr: *mut u8,
    /// Number of initialized bytes.
    pub len: usize,
    /// Allocation capacity of the original vector.
    pub cap: usize,
}

impl SerializedBytes {
    /// Converts a vector into ABI-safe raw parts.
    ///
    /// The allocation is intentionally kept alive until `into_vec` rebuilds it.
    pub fn from_vec(mut data: Vec<u8>) -> Self {
        let bytes = Self {
            ptr: data.as_mut_ptr(),
            len: data.len(),
            cap: data.capacity(),
        };
        std::mem::forget(data);
        bytes
    }

    #[doc(hidden)]
    pub fn from_serializable<T: serde::Serialize>(value: &T) -> Self {
        Self::from_vec(
            bincode::serde::encode_to_vec(value, bincode::config::standard()).unwrap_or_default(),
        )
    }

    /// Rebuilds the original vector from raw parts.
    ///
    /// # Safety
    ///
    /// The buffer must have been created by `SerializedBytes::from_vec` and
    /// must not have already been consumed. Rebuilding arbitrary raw parts or
    /// consuming the same value twice can corrupt memory.
    pub unsafe fn into_vec(self) -> Vec<u8> {
        if self.ptr.is_null() || self.cap == 0 || self.len > self.cap {
            return Vec::new();
        }

        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) }
    }

    #[doc(hidden)]
    /// Rebuilds and deserializes a value encoded by `from_serializable`.
    ///
    /// # Safety
    ///
    /// This has the same ownership requirements as `into_vec`.
    pub unsafe fn into_deserializable<T: serde::de::DeserializeOwned>(self) -> Option<T> {
        let bytes = unsafe { self.into_vec() };
        let Ok((value, bytes_read)) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
        else {
            return None;
        };

        (bytes_read == bytes.len()).then_some(value)
    }
}
