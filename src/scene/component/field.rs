use std::ffi::c_void;

use crate::{error::ComponentError, scene::component::field_type::FieldType};

pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);

#[derive(Clone, Copy)]
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl Field {
    pub fn new(type_hint: FieldType, getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self {
            type_hint,
            getter,
            setter,
        }
    }

    pub fn get_type(&self) -> FieldType {
        self.type_hint
    }

    pub fn get_getter(&self) -> Result<Getter, ComponentError> {
        match self.getter {
            Some(getter) => Ok(getter),
            None => {
                log::debug!("Schema field has no getter function");
                Err(ComponentError::FieldNoGetter)
            }
        }
    }

    pub fn get_setter(&self) -> Result<Setter, ComponentError> {
        match self.setter {
            Some(setter) => Ok(setter),
            None => {
                log::debug!("Schema field has no setter function");
                Err(ComponentError::FieldNoSetter)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    unsafe extern "C" fn test_getter(_data: *const c_void) -> *const c_void {
        std::ptr::null()
    }

    unsafe extern "C" fn test_setter(_data: *mut c_void, _value: *const c_void) {}

    #[test]
    fn field_new() {
        let field = Field::new(FieldType::Long, Some(test_getter), Some(test_setter));

        assert_eq!(field.get_type(), FieldType::Long);
    }

    #[test]
    fn field_get_getter_when_present() {
        let field = Field::new(FieldType::Blob, Some(test_getter), None);

        assert_eq!(
            field.get_getter().unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn field_get_getter_when_missing() {
        let field = Field::new(FieldType::Blob, None, Some(test_setter));

        assert_eq!(field.get_getter(), Err(ComponentError::FieldNoGetter));
    }

    #[test]
    fn field_get_setter_when_present() {
        let field = Field::new(FieldType::Blob, None, Some(test_setter));

        assert_eq!(
            field.get_setter().unwrap() as usize,
            test_setter as *const () as usize
        );
    }

    #[test]
    fn field_get_setter_when_missing() {
        let field = Field::new(FieldType::Blob, Some(test_getter), None);

        assert_eq!(field.get_setter(), Err(ComponentError::FieldNoSetter));
    }
}
