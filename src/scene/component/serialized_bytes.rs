#[repr(C)]
pub struct SerializedBytes {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

impl SerializedBytes {
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

    pub unsafe fn into_vec(self) -> Vec<u8> {
        if self.ptr.is_null() || self.cap == 0 || self.len > self.cap {
            return Vec::new();
        }

        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) }
    }

    #[doc(hidden)]
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
