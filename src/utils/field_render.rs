use std::{
    ffi::{c_char, c_void},
    str::FromStr,
};

use crate::{definitions::fields::TypeHint, errors::FieldParseError};

/// Renders the field value at `field_data` according to `type_hint`.
///
/// # Safety
///
/// `field_data` must be non-null, aligned, initialized, and point to a value
/// matching `type_hint` for the duration of this call.
pub unsafe fn render_field(field_data: *mut c_void, type_hint: TypeHint) -> String {
    match type_hint {
        TypeHint::I8 => unsafe { (*field_data.cast::<i8>()).to_string() },
        TypeHint::I16 => unsafe { (*field_data.cast::<i16>()).to_string() },
        TypeHint::I32 => unsafe { (*field_data.cast::<i32>()).to_string() },
        TypeHint::I64 => unsafe { (*field_data.cast::<i64>()).to_string() },
        TypeHint::I128 => unsafe { (*field_data.cast::<i128>()).to_string() },
        TypeHint::Isize => unsafe { (*field_data.cast::<isize>()).to_string() },
        TypeHint::U8 => unsafe { (*field_data.cast::<u8>()).to_string() },
        TypeHint::U16 => unsafe { (*field_data.cast::<u16>()).to_string() },
        TypeHint::U32 => unsafe { (*field_data.cast::<u32>()).to_string() },
        TypeHint::U64 => unsafe { (*field_data.cast::<u64>()).to_string() },
        TypeHint::U128 => unsafe { (*field_data.cast::<u128>()).to_string() },
        TypeHint::Usize => unsafe { (*field_data.cast::<usize>()).to_string() },
        TypeHint::F32 => unsafe { (*field_data.cast::<f32>()).to_string() },
        TypeHint::F64 => unsafe { (*field_data.cast::<f64>()).to_string() },
        TypeHint::Char => unsafe { char::from(*field_data.cast::<c_char>() as u8).to_string() },
        TypeHint::Boolean => unsafe { (*field_data.cast::<u8>() != 0).to_string() },
    }
}

/// Parses `input` according to `type_hint` and writes it to `field_data`.
///
/// # Safety
///
/// `field_data` must be non-null, aligned, writable, and point to a value
/// matching `type_hint` for the duration of this call.
pub unsafe fn field_parse(
    field_data: *mut c_void,
    type_hint: TypeHint,
    input: &str,
) -> Result<(), FieldParseError> {
    macro_rules! write_parsed {
        ($type:ty) => {{
            let value = parse(input)?;
            unsafe { field_data.cast::<$type>().write(value) };
        }};
    }

    match type_hint {
        TypeHint::I8 => write_parsed!(i8),
        TypeHint::I16 => write_parsed!(i16),
        TypeHint::I32 => write_parsed!(i32),
        TypeHint::I64 => write_parsed!(i64),
        TypeHint::I128 => write_parsed!(i128),
        TypeHint::Isize => write_parsed!(isize),
        TypeHint::U8 => write_parsed!(u8),
        TypeHint::U16 => write_parsed!(u16),
        TypeHint::U32 => write_parsed!(u32),
        TypeHint::U64 => write_parsed!(u64),
        TypeHint::U128 => write_parsed!(u128),
        TypeHint::Usize => write_parsed!(usize),
        TypeHint::F32 => write_parsed!(f32),
        TypeHint::F64 => write_parsed!(f64),
        TypeHint::Char => unsafe {
            field_data.cast::<c_char>().write(parse_c_char(input)?);
        },
        TypeHint::Boolean => unsafe {
            field_data.cast::<u8>().write(u8::from(parse_bool(input)?));
        },
    }

    Ok(())
}

fn parse<T: FromStr>(input: &str) -> Result<T, FieldParseError> {
    input.parse().map_err(|_| FieldParseError::InvalidInput)
}

fn parse_bool(input: &str) -> Result<bool, FieldParseError> {
    if input.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if input.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(FieldParseError::InvalidInput)
    }
}

fn parse_c_char(input: &str) -> Result<c_char, FieldParseError> {
    let mut chars = input.chars();
    let character = chars.next().ok_or(FieldParseError::InvalidInput)?;
    if chars.next().is_some() {
        return Err(FieldParseError::InvalidInput);
    }

    u8::try_from(u32::from(character))
        .map(|value| value as c_char)
        .map_err(|_| FieldParseError::InvalidInput)
}
