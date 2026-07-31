use std::ffi::c_void;

use crate::{
    error::ComponentError,
    scene::component::{field_type::FieldType, serialized_bytes::SerializedBytes},
};

/// C ABI function that returns a raw pointer to a component field.
///
/// # Safety
///
/// The caller must pass a pointer to the component type the getter was
/// generated for. Returning a pointer to the wrong type or to invalid storage
/// makes later field reads undefined behavior.
pub type Getter = unsafe extern "C" fn(*mut c_void) -> *mut c_void;

/// C ABI function that serializes one component field into owned bytes.
///
/// # Safety
///
/// The caller must pass a pointer to the component type the serializer was
/// generated for.
pub type Serializer = unsafe extern "C" fn(*const c_void) -> SerializedBytes;

/// C ABI function that writes serialized bytes back into one component field.
///
/// # Safety
///
/// The caller must pass a pointer to the component type the deserializer was
/// generated for, and the bytes must have the format that deserializer expects.
pub type Deserializer = unsafe extern "C" fn(*mut c_void, SerializedBytes);

#[derive(Clone, Copy)]
/// Runtime metadata for one component field.
pub struct Field {
    type_hint: FieldType,
    getter: Option<Getter>,
    mutable: bool,
    serializer: Option<Serializer>,
    deserializer: Option<Deserializer>,
}

impl Field {
    /// Creates field metadata from a type hint and optional field binding functions.
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

    /// Returns the runtime type hint registered for this field.
    pub fn get_type(&self) -> FieldType {
        self.type_hint
    }

    /// Returns the getter function, or `FieldNoGetter` if the schema has none.
    pub fn get_getter(&self) -> Result<Getter, ComponentError> {
        match self.getter {
            Some(getter) => Ok(getter),
            None => Err(ComponentError::FieldNoGetter),
        }
    }

    /// Returns whether this field may be borrowed mutably through `Scene::query_mut`.
    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    /// Returns whether this field can be parsed from text with `parse`.
    pub fn is_string_parsable(&self) -> bool {
        !matches!(self.type_hint, FieldType::Blob)
    }

    /// Returns the serializer function, or `FieldNoSerializer` if the schema has none.
    pub fn get_serializer(&self) -> Result<Serializer, ComponentError> {
        match self.serializer {
            Some(serializer) => Ok(serializer),
            None => Err(ComponentError::FieldNoSerializer),
        }
    }

    /// Returns the deserializer function, or `FieldNoDeserializer` if the schema has none.
    pub fn get_deserializer(&self) -> Result<Deserializer, ComponentError> {
        match self.deserializer {
            Some(deserializer) => Ok(deserializer),
            None => Err(ComponentError::FieldNoDeserializer),
        }
    }

    /// Renders the raw field value as a string according to this field's type hint.
    ///
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
            FieldType::F32Vec2 => unsafe { render_vector(&*(ptr as *const [f32; 2])) },
            FieldType::F32Vec3 => unsafe { render_vector(&*(ptr as *const [f32; 3])) },
            FieldType::F64Vec2 => unsafe { render_vector(&*(ptr as *const [f64; 2])) },
            FieldType::F64Vec3 => unsafe { render_vector(&*(ptr as *const [f64; 3])) },
            FieldType::Char => unsafe { (*(ptr as *const char)).to_string() },
            FieldType::String => unsafe { (*(ptr as *const String)).clone() },
            FieldType::Blob => format!("{ptr:p}"),
            FieldType::Boolean => unsafe { (*(ptr as *const u8) != 0).to_string() },
        })
    }

    /// Parses `input` and writes it into the raw field value.
    ///
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
            FieldType::F32Vec2 => unsafe { *(ptr as *mut [f32; 2]) = parse_vector(input)? },
            FieldType::F32Vec3 => unsafe { *(ptr as *mut [f32; 3]) = parse_vector(input)? },
            FieldType::F64Vec2 => unsafe { *(ptr as *mut [f64; 2]) = parse_vector(input)? },
            FieldType::F64Vec3 => unsafe { *(ptr as *mut [f64; 3]) = parse_vector(input)? },
            FieldType::Char => unsafe { *(ptr as *mut char) = parse_char(input)? },
            FieldType::String => unsafe { *(ptr as *mut String) = input.to_owned() },
            FieldType::Blob => return Err(ComponentError::FieldValueParsing),
            FieldType::Boolean => unsafe { *(ptr as *mut u8) = u8::from(parse_bool(input)?) },
        }

        Ok(())
    }
}

fn parse_value<T: std::str::FromStr>(input: &str) -> Result<T, ComponentError> {
    input.parse().map_err(|_| ComponentError::FieldValueParsing)
}

fn render_vector<T: ToString, const N: usize>(value: &[T; N]) -> String {
    value
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_vector<T: std::str::FromStr, const N: usize>(
    input: &str,
) -> Result<[T; N], ComponentError> {
    let values = input
        .split(',')
        .map(|part| parse_value(part.trim()))
        .collect::<Result<Vec<_>, _>>()?;

    values
        .try_into()
        .map_err(|_| ComponentError::FieldValueParsing)
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

fn parse_bool(input: &str) -> Result<bool, ComponentError> {
    match input.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ComponentError::FieldValueParsing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_plugin_fixture::{noop_deserializer, null_getter, sample_serializer};
    use rstest::rstest;
    use std::ffi::c_void;

    /// Renders `value`, parses `input` back over the same storage, and asserts
    /// the stored result. `T` must be laid out exactly as the field type expects.
    fn assert_roundtrip<T: PartialEq + std::fmt::Debug>(
        type_hint: FieldType,
        mut value: T,
        rendered: &str,
        input: &str,
        expected: T,
    ) {
        let field = Field::new(type_hint, None, true, None, None);
        let ptr = &mut value as *mut T as *mut c_void;

        assert_eq!(unsafe { field.render(ptr) }.unwrap(), rendered);
        unsafe { field.parse(ptr, input) }.unwrap();
        assert_eq!(value, expected);
    }

    /// Asserts that each `input` is rejected as unparsable and leaves `value`
    /// untouched.
    fn assert_parse_rejected<T: PartialEq + std::fmt::Debug + Clone>(
        type_hint: FieldType,
        mut value: T,
        inputs: &[&str],
    ) {
        let field = Field::new(type_hint, None, true, None, None);
        let ptr = &mut value as *mut T as *mut c_void;
        let original = value.clone();

        for input in inputs {
            assert_eq!(
                unsafe { field.parse(ptr, input) },
                Err(ComponentError::FieldValueParsing)
            );
        }
        assert_eq!(value, original);
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn field_reports_configured_mutability(#[case] mutable: bool) {
        let field = Field::new(FieldType::Blob, Some(null_getter), mutable, None, None);

        assert_eq!(field.is_mutable(), mutable);
    }

    #[rstest]
    fn field_exposes_type_and_each_hook_when_present_or_missing() {
        // A fully-wired field reports its type and returns every hook pointer.
        let full = Field::new(
            FieldType::I64,
            Some(null_getter),
            true,
            Some(sample_serializer),
            Some(noop_deserializer),
        );
        assert_eq!(full.get_type(), FieldType::I64);
        assert_eq!(
            full.get_getter().unwrap() as usize,
            null_getter as *const () as usize
        );
        assert_eq!(
            full.get_serializer().unwrap() as usize,
            sample_serializer as *const () as usize
        );
        assert_eq!(
            full.get_deserializer().unwrap() as usize,
            noop_deserializer as *const () as usize
        );

        // A bare field reports the matching "missing hook" error for each.
        let bare = Field::new(FieldType::Blob, None, false, None, None);
        assert_eq!(bare.get_getter(), Err(ComponentError::FieldNoGetter));
        assert_eq!(
            bare.get_serializer(),
            Err(ComponentError::FieldNoSerializer)
        );
        assert_eq!(
            bare.get_deserializer(),
            Err(ComponentError::FieldNoDeserializer)
        );
    }

    #[rstest]
    fn field_renders_and_parses_scalar_types() {
        assert_roundtrip(FieldType::I32, -4i32, "-4", "42", 42);
        assert_roundtrip(FieldType::F32, 1.5f32, "1.5", "2.25", 2.25);
        assert_roundtrip(FieldType::Char, 'a', "a", "z", 'z');
        assert_roundtrip(
            FieldType::String,
            "old".to_owned(),
            "old",
            "new",
            "new".to_owned(),
        );
        // Boolean: any non-zero byte renders `true`, and parsing is case-insensitive.
        assert_roundtrip(FieldType::Boolean, 2u8, "true", "FALSE", 0u8);
        assert_roundtrip(FieldType::Boolean, 0u8, "false", "tRuE", 1u8);
    }

    #[rstest]
    fn field_renders_and_parses_vectors_tolerating_whitespace() {
        assert_roundtrip(
            FieldType::F32Vec3,
            [1.5f32, 2.5, 3.5],
            "1.5, 2.5, 3.5",
            "4.0, 5.0, 6.0",
            [4.0, 5.0, 6.0],
        );
        // The compact, space-free form parses too.
        assert_roundtrip(
            FieldType::F32Vec3,
            [4.0f32, 5.0, 6.0],
            "4, 5, 6",
            "7.0,8.0,9.0",
            [7.0, 8.0, 9.0],
        );
        assert_roundtrip(
            FieldType::F64Vec2,
            [1.5f64, 2.5],
            "1.5, 2.5",
            "3.0,4.0",
            [3.0, 4.0],
        );
    }

    #[rstest]
    fn field_parse_rejects_invalid_inputs() {
        assert_parse_rejected(FieldType::I32, 7i32, &["not a number"]);
        assert_parse_rejected(FieldType::U8, 1u8, &["-1"]); // unsigned rejects negative
        assert_parse_rejected(FieldType::Char, 'a', &["ab"]); // char rejects multi-char
        assert_parse_rejected(FieldType::Boolean, 1u8, &["yes"]);
        // Vectors reject wrong arity and non-numeric elements.
        assert_parse_rejected(FieldType::F32Vec2, [1.0f32, 2.0], &["3.0", "3.0, invalid"]);
    }

    #[rstest]
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
