use std::ffi::c_void;

use crate::{error::AssetError, scene::component::FieldType};

pub type Getter = unsafe extern "C" fn(*mut c_void) -> *mut c_void;

#[derive(Clone, Copy)]
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
}

impl Field {
    pub fn new(type_hint: FieldType, getter: Option<Getter>) -> Self {
        Self { type_hint, getter }
    }

    pub fn get_type(&self) -> FieldType {
        self.type_hint
    }

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
