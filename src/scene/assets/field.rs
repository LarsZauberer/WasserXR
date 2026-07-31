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
    use crate::testing_plugin_fixture::null_getter;
    use rstest::rstest;

    #[rstest]
    fn field_exposes_type(#[values(FieldType::String, FieldType::Blob)] type_hint: FieldType) {
        let field = Field::new(type_hint, Some(null_getter));

        assert_eq!(field.get_type(), type_hint);
    }

    #[rstest]
    fn field_getter_is_returned_when_present_and_errors_when_missing() {
        let present = Field::new(FieldType::Blob, Some(null_getter));
        assert_eq!(
            present.get_getter().unwrap() as usize,
            null_getter as *const () as usize
        );

        let missing = Field::new(FieldType::Blob, None);
        assert_eq!(missing.get_getter(), Err(AssetError::FieldNoGetter));
    }
}
