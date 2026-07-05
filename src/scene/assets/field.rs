use std::ffi::c_void;

use crate::{error::AssetError, scene::component::FieldType};

/// C ABI function that returns a raw pointer to an asset field.
///
/// # Safety
///
/// The caller must pass a pointer to the asset type the getter was generated
/// for. Returning a pointer to invalid storage makes later field reads
/// undefined behavior.
pub type Getter = unsafe extern "C" fn(*mut c_void) -> *mut c_void;

#[derive(Clone, Copy)]
/// Runtime metadata for one asset field.
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
}

impl Field {
    /// Creates field metadata from a type hint and optional getter.
    pub fn new(type_hint: FieldType, getter: Option<Getter>) -> Self {
        Self { type_hint, getter }
    }

    /// Returns the runtime type hint registered for this field.
    pub fn get_type(&self) -> FieldType {
        self.type_hint
    }

    /// Returns the getter function, or `FieldNoGetter` if the schema has none.
    pub fn get_getter(&self) -> Result<Getter, AssetError> {
        match self.getter {
            Some(getter) => Ok(getter),
            None => Err(AssetError::FieldNoGetter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[test]
    fn field_new() {
        let field = Field::new(FieldType::String, Some(test_getter));

        assert_eq!(field.get_type(), FieldType::String);
    }

    #[test]
    fn field_get_getter_when_present() {
        let field = Field::new(FieldType::Blob, Some(test_getter));

        assert_eq!(
            field.get_getter().unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn field_get_getter_when_missing() {
        let field = Field::new(FieldType::Blob, None);

        assert_eq!(field.get_getter(), Err(AssetError::FieldNoGetter));
    }
}
