use std::ffi::c_void;

use crate::{
    error::ComponentError,
    scene::component::{field_type::FieldType, serialized_bytes::SerializedBytes},
};

pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type GetterMut = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);
pub type Mover = unsafe extern "C" fn(*mut c_void, *mut c_void);
pub type Taker = unsafe extern "C" fn(*mut c_void, *mut c_void);
pub type Serializer = unsafe extern "C" fn(*const c_void) -> SerializedBytes;
pub type Deserializer = unsafe extern "C" fn(*mut c_void, SerializedBytes);

#[derive(Clone, Copy)]
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
    getter_mut: Option<GetterMut>,
    setter: Option<Setter>,
    mover: Option<Mover>,
    taker: Option<Taker>,
    serializer: Option<Serializer>,
    deserializer: Option<Deserializer>,
}

impl Field {
    pub fn new(
        type_hint: FieldType,
        getter: Option<Getter>,
        getter_mut: Option<GetterMut>,
        setter: Option<Setter>,
        mover: Option<Mover>,
        taker: Option<Taker>,
        serializer: Option<Serializer>,
        deserializer: Option<Deserializer>,
    ) -> Self {
        Self {
            type_hint,
            getter,
            getter_mut,
            setter,
            mover,
            taker,
            serializer,
            deserializer,
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

    pub fn get_getter_mut(&self) -> Result<GetterMut, ComponentError> {
        match self.getter_mut {
            Some(getter_mut) => Ok(getter_mut),
            None => {
                log::debug!("Schema field has no mutable getter function");
                Err(ComponentError::FieldNoGetterMut)
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

    pub fn get_mover(&self) -> Result<Mover, ComponentError> {
        match self.mover {
            Some(mover) => Ok(mover),
            None => {
                log::debug!("Schema field has no mover function");
                Err(ComponentError::FieldNoMover)
            }
        }
    }

    pub fn get_taker(&self) -> Result<Taker, ComponentError> {
        match self.taker {
            Some(taker) => Ok(taker),
            None => {
                log::debug!("Schema field has no taker function");
                Err(ComponentError::FieldNoTaker)
            }
        }
    }

    pub fn get_serializer(&self) -> Result<Serializer, ComponentError> {
        match self.serializer {
            Some(serializer) => Ok(serializer),
            None => {
                log::debug!("Schema field has no serializer function");
                Err(ComponentError::FieldNoSerializer)
            }
        }
    }

    pub fn get_deserializer(&self) -> Result<Deserializer, ComponentError> {
        match self.deserializer {
            Some(deserializer) => Ok(deserializer),
            None => {
                log::debug!("Schema field has no deserializer function");
                Err(ComponentError::FieldNoDeserializer)
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

    unsafe extern "C" fn test_getter_mut(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn test_setter(_data: *mut c_void, _value: *const c_void) {}

    unsafe extern "C" fn test_mover(_data: *mut c_void, _value: *mut c_void) {}

    unsafe extern "C" fn test_taker(_data: *mut c_void, _out: *mut c_void) {}

    unsafe extern "C" fn test_serializer(_data: *const c_void) -> SerializedBytes {
        SerializedBytes::from_vec(vec![1, 2, 3])
    }

    unsafe extern "C" fn test_deserializer(_data: *mut c_void, _value: SerializedBytes) {}

    #[test]
    fn field_new() {
        let field = Field::new(
            FieldType::Long,
            Some(test_getter),
            Some(test_getter_mut),
            Some(test_setter),
            Some(test_mover),
            Some(test_taker),
            Some(test_serializer),
            Some(test_deserializer),
        );

        assert_eq!(field.get_type(), FieldType::Long);
    }

    #[test]
    fn field_get_getter_when_present() {
        let field = Field::new(
            FieldType::Blob,
            Some(test_getter),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            field.get_getter().unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn field_get_getter_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            Some(test_setter),
            None,
            None,
            None,
            None,
        );

        assert_eq!(field.get_getter(), Err(ComponentError::FieldNoGetter));
    }

    #[test]
    fn field_get_setter_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            Some(test_setter),
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            field.get_setter().unwrap() as usize,
            test_setter as *const () as usize
        );
    }

    #[test]
    fn field_get_setter_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            Some(test_getter),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(field.get_setter(), Err(ComponentError::FieldNoSetter));
    }

    #[test]
    fn field_get_getter_mut_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            Some(test_getter_mut),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            field.get_getter_mut().unwrap() as usize,
            test_getter_mut as *const () as usize
        );
    }

    #[test]
    fn field_get_getter_mut_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            Some(test_getter),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            field.get_getter_mut(),
            Err(ComponentError::FieldNoGetterMut)
        );
    }

    #[test]
    fn field_get_mover_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            Some(test_mover),
            None,
            None,
            None,
        );

        assert_eq!(
            field.get_mover().unwrap() as usize,
            test_mover as *const () as usize
        );
    }

    #[test]
    fn field_get_mover_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            Some(test_taker),
            None,
            None,
        );

        assert_eq!(field.get_mover(), Err(ComponentError::FieldNoMover));
    }

    #[test]
    fn field_get_taker_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            Some(test_taker),
            None,
            None,
        );

        assert_eq!(
            field.get_taker().unwrap() as usize,
            test_taker as *const () as usize
        );
    }

    #[test]
    fn field_get_taker_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            Some(test_mover),
            None,
            None,
            None,
        );

        assert_eq!(field.get_taker(), Err(ComponentError::FieldNoTaker));
    }

    #[test]
    fn field_get_serializer_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            None,
            Some(test_serializer),
            None,
        );

        assert_eq!(
            field.get_serializer().unwrap() as usize,
            test_serializer as *const () as usize
        );
    }

    #[test]
    fn field_get_serializer_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(test_deserializer),
        );

        assert_eq!(
            field.get_serializer(),
            Err(ComponentError::FieldNoSerializer)
        );
    }

    #[test]
    fn field_get_deserializer_when_present() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(test_deserializer),
        );

        assert_eq!(
            field.get_deserializer().unwrap() as usize,
            test_deserializer as *const () as usize
        );
    }

    #[test]
    fn field_get_deserializer_when_missing() {
        let field = Field::new(
            FieldType::Blob,
            None,
            None,
            None,
            None,
            None,
            Some(test_serializer),
            None,
        );

        assert_eq!(
            field.get_deserializer(),
            Err(ComponentError::FieldNoDeserializer)
        );
    }
}
