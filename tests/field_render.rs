use std::ffi::{c_char, c_void};

use wasserxr::{
    definitions::fields::TypeHint,
    errors::FieldParseError,
    utils::field_render::{field_parse, render_field},
};

fn pointer<T>(value: &mut T) -> *mut c_void {
    std::ptr::from_mut(value).cast()
}

#[test]
fn renders_and_parses_primitive_fields() {
    macro_rules! assert_round_trip {
        ($type:ty, $hint:expr, $input:literal, $expected:expr) => {{
            let mut value: $type = Default::default();
            unsafe { field_parse(pointer(&mut value), $hint, $input) }.unwrap();
            assert_eq!(value, $expected);
            assert_eq!(unsafe { render_field(pointer(&mut value), $hint) }, $input);
        }};
    }

    assert_round_trip!(i8, TypeHint::I8, "-8", -8);
    assert_round_trip!(i16, TypeHint::I16, "-16", -16);
    assert_round_trip!(i32, TypeHint::I32, "-32", -32);
    assert_round_trip!(i64, TypeHint::I64, "-64", -64);
    assert_round_trip!(i128, TypeHint::I128, "-128", -128);
    assert_round_trip!(isize, TypeHint::Isize, "-42", -42);
    assert_round_trip!(u8, TypeHint::U8, "8", 8);
    assert_round_trip!(u16, TypeHint::U16, "16", 16);
    assert_round_trip!(u32, TypeHint::U32, "32", 32);
    assert_round_trip!(u64, TypeHint::U64, "64", 64);
    assert_round_trip!(u128, TypeHint::U128, "128", 128);
    assert_round_trip!(usize, TypeHint::Usize, "42", 42);
    assert_round_trip!(f32, TypeHint::F32, "1.5", 1.5);
    assert_round_trip!(f64, TypeHint::F64, "2.25", 2.25);
    assert_round_trip!(c_char, TypeHint::Char, "z", b'z' as c_char);
    assert_round_trip!(u8, TypeHint::Boolean, "true", 1);
}

#[test]
fn parsing_c_char_does_not_overwrite_adjacent_data() {
    #[repr(C)]
    struct FieldData {
        character: c_char,
        sentinel: [u8; 3],
    }

    let mut data = FieldData {
        character: b'a' as c_char,
        sentinel: [1, 2, 3],
    };

    unsafe { field_parse(pointer(&mut data.character), TypeHint::Char, "z") }.unwrap();

    assert_eq!(data.character, b'z' as c_char);
    assert_eq!(data.sentinel, [1, 2, 3]);
}

#[test]
fn parse_errors_do_not_overwrite_fields() {
    macro_rules! assert_parse_error {
        ($value:expr, $hint:expr, $input:literal) => {{
            let mut value = $value;
            let original = value;
            assert_eq!(
                unsafe { field_parse(pointer(&mut value), $hint, $input) },
                Err(FieldParseError::InvalidInput)
            );
            assert_eq!(value, original);
        }};
    }

    assert_parse_error!(7_u8, TypeHint::U8, "256");
    assert_parse_error!(1_i32, TypeHint::I32, "one");
    assert_parse_error!(1.0_f32, TypeHint::F32, "float");
    assert_parse_error!(b'a' as c_char, TypeHint::Char, "ab");
    assert_parse_error!(b'a' as c_char, TypeHint::Char, "Ā");
    assert_parse_error!(0_u8, TypeHint::Boolean, "yes");
}
