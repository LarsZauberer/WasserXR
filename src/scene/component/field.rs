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
            None => Err(ComponentError::FieldNoGetter),
        }
    }

    pub fn get_setter(&self) -> Result<Setter, ComponentError> {
        match self.setter {
            Some(setter) => Ok(setter),
            None => Err(ComponentError::FieldNoSetter),
        }
    }
}
