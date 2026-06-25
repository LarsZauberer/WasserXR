use std::ffi::c_void;

use crate::{
    error::ComponentError,
    scene::{
        component::{field_type::FieldType, serialized_bytes::SerializedBytes},
        logging::LogManager,
    },
};

pub type Getter = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
pub type Serializer = unsafe extern "C" fn(*const c_void) -> SerializedBytes;
pub type Deserializer = unsafe extern "C" fn(*mut c_void, SerializedBytes);

#[derive(Clone, Copy)]
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
    mutable: bool,
    serializer: Option<Serializer>,
    deserializer: Option<Deserializer>,
}

impl Field {
    pub fn new(
        type_hint: FieldType,
        getter: Option<Getter>,
        mutable: bool,
        serializer: Option<Serializer>,
        deserializer: Option<Deserializer>,
    ) -> Self {
        Self {
            type_hint,
            getter,
            mutable,
            serializer,
            deserializer,
        }
    }

    pub fn get_type(&self) -> FieldType {
        self.type_hint
    }

    pub fn get_getter(&self, log_manager: &LogManager) -> Result<Getter, ComponentError> {
        match self.getter {
            Some(getter) => Ok(getter),
            None => {
                crate::debug!(log_manager, "Schema field has no getter function");
                Err(ComponentError::FieldNoGetter)
            }
        }
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    pub fn get_serializer(&self, log_manager: &LogManager) -> Result<Serializer, ComponentError> {
        match self.serializer {
            Some(serializer) => Ok(serializer),
            None => {
                crate::debug!(log_manager, "Schema field has no serializer function");
                Err(ComponentError::FieldNoSerializer)
            }
        }
    }

    pub fn get_deserializer(
        &self,
        log_manager: &LogManager,
    ) -> Result<Deserializer, ComponentError> {
        match self.deserializer {
            Some(deserializer) => Ok(deserializer),
            None => {
                crate::debug!(log_manager, "Schema field has no deserializer function");
                Err(ComponentError::FieldNoDeserializer)
            }
        }
    }

    /// # Safety
    ///
    /// `ptr` must point to a value matching this field's type hint.
    pub unsafe fn render(&self, ptr: *mut c_void) -> Result<String, ComponentError> {
        if ptr.is_null() {
            return Err(ComponentError::FieldValueParsing);
        }

        Ok(match self.type_hint {
            FieldType::I8 => unsafe { (*(ptr as *const i8)).to_string() },
            FieldType::I16 => unsafe { (*(ptr as *const i16)).to_string() },
            FieldType::I32 => unsafe { (*(ptr as *const i32)).to_string() },
            FieldType::I64 => unsafe { (*(ptr as *const i64)).to_string() },
            FieldType::I128 => unsafe { (*(ptr as *const i128)).to_string() },
            FieldType::Isize => unsafe { (*(ptr as *const isize)).to_string() },
            FieldType::U8 => unsafe { (*(ptr as *const u8)).to_string() },
            FieldType::U16 => unsafe { (*(ptr as *const u16)).to_string() },
            FieldType::U32 => unsafe { (*(ptr as *const u32)).to_string() },
            FieldType::U64 => unsafe { (*(ptr as *const u64)).to_string() },
            FieldType::U128 => unsafe { (*(ptr as *const u128)).to_string() },
            FieldType::Usize => unsafe { (*(ptr as *const usize)).to_string() },
            FieldType::F32 => unsafe { (*(ptr as *const f32)).to_string() },
            FieldType::F64 => unsafe { (*(ptr as *const f64)).to_string() },
            FieldType::Char => unsafe { (*(ptr as *const char)).to_string() },
            FieldType::String => unsafe { (*(ptr as *const String)).clone() },
            FieldType::Blob => format!("{ptr:p}"),
        })
    }

    /// # Safety
    ///
    /// `ptr` must point to a value matching this field's type hint.
    pub unsafe fn parse(&self, ptr: *mut c_void, input: &str) -> Result<(), ComponentError> {
        if ptr.is_null() {
            return Err(ComponentError::FieldValueParsing);
        }

        match self.type_hint {
            FieldType::I8 => unsafe { *(ptr as *mut i8) = parse_value(input)? },
            FieldType::I16 => unsafe { *(ptr as *mut i16) = parse_value(input)? },
            FieldType::I32 => unsafe { *(ptr as *mut i32) = parse_value(input)? },
            FieldType::I64 => unsafe { *(ptr as *mut i64) = parse_value(input)? },
            FieldType::I128 => unsafe { *(ptr as *mut i128) = parse_value(input)? },
            FieldType::Isize => unsafe { *(ptr as *mut isize) = parse_value(input)? },
            FieldType::U8 => unsafe { *(ptr as *mut u8) = parse_value(input)? },
            FieldType::U16 => unsafe { *(ptr as *mut u16) = parse_value(input)? },
            FieldType::U32 => unsafe { *(ptr as *mut u32) = parse_value(input)? },
            FieldType::U64 => unsafe { *(ptr as *mut u64) = parse_value(input)? },
            FieldType::U128 => unsafe { *(ptr as *mut u128) = parse_value(input)? },
            FieldType::Usize => unsafe { *(ptr as *mut usize) = parse_value(input)? },
            FieldType::F32 => unsafe { *(ptr as *mut f32) = parse_value(input)? },
            FieldType::F64 => unsafe { *(ptr as *mut f64) = parse_value(input)? },
            FieldType::Char => unsafe { *(ptr as *mut char) = parse_char(input)? },
            FieldType::String => unsafe { *(ptr as *mut String) = input.to_owned() },
            FieldType::Blob => return Err(ComponentError::FieldValueParsing),
        }

        Ok(())
    }
}

fn parse_value<T: std::str::FromStr>(input: &str) -> Result<T, ComponentError> {
    input.parse().map_err(|_| ComponentError::FieldValueParsing)
}

fn parse_char(input: &str) -> Result<char, ComponentError> {
    let mut chars = input.chars();
    let Some(value) = chars.next() else {
        return Err(ComponentError::FieldValueParsing);
    };

    if chars.next().is_some() {
        return Err(ComponentError::FieldValueParsing);
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::logging::LogManager;
    use std::ffi::c_void;

    unsafe extern "C" fn test_getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn test_serializer(_data: *const c_void) -> SerializedBytes {
        SerializedBytes::from_vec(vec![1, 2, 3])
    }

    unsafe extern "C" fn test_deserializer(_data: *mut c_void, _value: SerializedBytes) {}

    fn test_log_manager() -> LogManager {
        LogManager::new("WasserXR".to_owned())
    }

    #[test]
    fn field_new() {
        let field = Field::new(
            FieldType::I64,
            Some(test_getter),
            true,
            Some(test_serializer),
            Some(test_deserializer),
        );

        assert_eq!(field.get_type(), FieldType::I64);
        assert!(field.is_mutable());
    }

    #[test]
    fn field_get_getter_when_present() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, Some(test_getter), false, None, None);

        assert_eq!(
            field.get_getter(&log_manager).unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn field_get_getter_when_missing() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, None, false, None, None);

        assert_eq!(
            field.get_getter(&log_manager),
            Err(ComponentError::FieldNoGetter)
        );
    }

    #[test]
    fn field_is_mutable_when_enabled() {
        let field = Field::new(FieldType::Blob, Some(test_getter), true, None, None);

        assert!(field.is_mutable());
    }

    #[test]
    fn field_is_not_mutable_by_default() {
        let field = Field::new(FieldType::Blob, Some(test_getter), false, None, None);

        assert!(!field.is_mutable());
    }

    #[test]
    fn field_get_serializer_when_present() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, None, false, Some(test_serializer), None);

        assert_eq!(
            field.get_serializer(&log_manager).unwrap() as usize,
            test_serializer as *const () as usize
        );
    }

    #[test]
    fn field_get_serializer_when_missing() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, None, false, None, Some(test_deserializer));

        assert_eq!(
            field.get_serializer(&log_manager),
            Err(ComponentError::FieldNoSerializer)
        );
    }

    #[test]
    fn field_get_deserializer_when_present() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, None, false, None, Some(test_deserializer));

        assert_eq!(
            field.get_deserializer(&log_manager).unwrap() as usize,
            test_deserializer as *const () as usize
        );
    }

    #[test]
    fn field_get_deserializer_when_missing() {
        let log_manager = test_log_manager();
        let field = Field::new(FieldType::Blob, None, false, Some(test_serializer), None);

        assert_eq!(
            field.get_deserializer(&log_manager),
            Err(ComponentError::FieldNoDeserializer)
        );
    }

    #[test]
    fn field_render_and_parse_signed_integer() {
        let field = Field::new(FieldType::I32, None, true, None, None);
        let mut value = -4i32;
        let ptr = &mut value as *mut i32 as *mut c_void;

        assert_eq!(unsafe { field.render(ptr) }.unwrap(), "-4");
        unsafe { field.parse(ptr, "42") }.unwrap();

        assert_eq!(value, 42);
        assert_eq!(
            unsafe { field.parse(ptr, "not a number") },
            Err(ComponentError::FieldValueParsing)
        );
    }

    #[test]
    fn field_parse_unsigned_integer_rejects_negative_input() {
        let field = Field::new(FieldType::U8, None, true, None, None);
        let mut value = 1u8;
        let ptr = &mut value as *mut u8 as *mut c_void;

        assert_eq!(
            unsafe { field.parse(ptr, "-1") },
            Err(ComponentError::FieldValueParsing)
        );
        assert_eq!(value, 1);
    }

    #[test]
    fn field_render_and_parse_float() {
        let field = Field::new(FieldType::F32, None, true, None, None);
        let mut value = 1.5f32;
        let ptr = &mut value as *mut f32 as *mut c_void;

        assert_eq!(unsafe { field.render(ptr) }.unwrap(), "1.5");
        unsafe { field.parse(ptr, "2.25") }.unwrap();

        assert_eq!(value, 2.25);
    }

    #[test]
    fn field_render_and_parse_char() {
        let field = Field::new(FieldType::Char, None, true, None, None);
        let mut value = 'a';
        let ptr = &mut value as *mut char as *mut c_void;

        assert_eq!(unsafe { field.render(ptr) }.unwrap(), "a");
        unsafe { field.parse(ptr, "z") }.unwrap();

        assert_eq!(value, 'z');
        assert_eq!(
            unsafe { field.parse(ptr, "ab") },
            Err(ComponentError::FieldValueParsing)
        );
    }

    #[test]
    fn field_render_and_parse_string() {
        let field = Field::new(FieldType::String, None, true, None, None);
        let mut value = "old".to_owned();
        let ptr = &mut value as *mut String as *mut c_void;

        assert_eq!(unsafe { field.render(ptr) }.unwrap(), "old");
        unsafe { field.parse(ptr, "new") }.unwrap();

        assert_eq!(value, "new");
    }

    #[test]
    fn field_renders_blob_pointer_and_rejects_parse() {
        let field = Field::new(FieldType::Blob, None, true, None, None);
        let mut value = 1u8;
        let ptr = &mut value as *mut u8 as *mut c_void;

        assert!(unsafe { field.render(ptr) }.unwrap().starts_with("0x"));
        assert_eq!(
            unsafe { field.parse(ptr, "0x0") },
            Err(ComponentError::FieldValueParsing)
        );
    }
}
