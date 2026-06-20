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

    pub unsafe fn into_vec(self) -> Vec<u8> {
        if self.ptr.is_null() || self.cap == 0 || self.len > self.cap {
            return Vec::new();
        }

        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) }
    }
}
