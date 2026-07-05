use std::ffi::c_char;
use std::ffi::c_void;

use crate::{
    bindings::{WXRSceneError, clear_error, set_error, str_from_ptr},
    scene::{
        assets,
        component::{self, FieldType},
    },
};

/// C ABI function that returns a raw pointer to a component or asset field.
pub type WXRGetter = unsafe extern "C" fn(*mut c_void) -> *mut c_void;

/// C ABI function that serializes one component field into owned bytes.
pub type WXRSerializer = unsafe extern "C" fn(*const c_void) -> component::SerializedBytes;

/// C ABI function that writes serialized bytes back into one component field.
pub type WXRDeserializer = unsafe extern "C" fn(*mut c_void, component::SerializedBytes);

/// Opaque C handle for a component schema.
pub struct WXRComponentSchema {
    _private: [u8; 0],
}

/// Opaque C handle for an asset schema.
pub struct WXRAssetSchema {
    _private: [u8; 0],
}

fn component_schema_mut<'a>(
    schema: *mut WXRComponentSchema,
) -> Result<&'a mut component::Schema, WXRSceneError> {
    if schema.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &mut *(schema as *mut component::Schema) })
}

fn asset_schema_mut<'a>(
    schema: *mut WXRAssetSchema,
) -> Result<&'a mut assets::Schema, WXRSceneError> {
    if schema.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &mut *(schema as *mut assets::Schema) })
}

/// Allocates an empty component schema.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_component_schema_new() -> *mut WXRComponentSchema {
    clear_error();
    Box::into_raw(Box::new(component::Schema::default())).cast()
}

/// Frees a component schema allocated by `wxr_component_schema_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_component_schema_free(schema: *mut WXRComponentSchema) {
    if !schema.is_null() {
        unsafe {
            drop(Box::from_raw(schema as *mut component::Schema));
        }
    }
}

/// Adds a field definition to a component schema.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_component_schema_add_field(
    schema: *mut WXRComponentSchema,
    id: *const c_char,
    type_hint: FieldType,
    getter: WXRGetter,
    mutable: i32,
) -> i32 {
    match (component_schema_mut(schema), unsafe { str_from_ptr(id) }) {
        (Ok(schema), Ok(id)) => {
            schema.add_field(
                id.to_owned(),
                type_hint,
                Some(getter),
                mutable != 0,
                None,
                None,
            );
            clear_error();
            0
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Adds a serializable field definition to a component schema.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_component_schema_add_serialized_field(
    schema: *mut WXRComponentSchema,
    id: *const c_char,
    type_hint: FieldType,
    getter: WXRGetter,
    mutable: i32,
    serializer: WXRSerializer,
    deserializer: WXRDeserializer,
) -> i32 {
    match (component_schema_mut(schema), unsafe { str_from_ptr(id) }) {
        (Ok(schema), Ok(id)) => {
            schema.add_field(
                id.to_owned(),
                type_hint,
                Some(getter),
                mutable != 0,
                Some(serializer),
                Some(deserializer),
            );
            clear_error();
            0
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Allocates an empty asset schema.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_asset_schema_new() -> *mut WXRAssetSchema {
    clear_error();
    Box::into_raw(Box::new(assets::Schema::default())).cast()
}

/// Frees an asset schema allocated by `wxr_asset_schema_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_asset_schema_free(schema: *mut WXRAssetSchema) {
    if !schema.is_null() {
        unsafe {
            drop(Box::from_raw(schema as *mut assets::Schema));
        }
    }
}

/// Adds a field definition to an asset schema.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_asset_schema_add_field(
    schema: *mut WXRAssetSchema,
    id: *const c_char,
    type_hint: FieldType,
    getter: WXRGetter,
) -> i32 {
    match (asset_schema_mut(schema), unsafe { str_from_ptr(id) }) {
        (Ok(schema), Ok(id)) => {
            schema.add_field(id.to_owned(), type_hint, Some(getter));
            clear_error();
            0
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::{CString, c_void},
        ptr,
    };

    unsafe extern "C" fn getter(_data: *mut c_void) -> *mut c_void {
        ptr::null_mut()
    }

    #[test]
    fn component_schema_add_field_accepts_c_values() {
        let schema = wxr_component_schema_new();
        let field = CString::new("value").unwrap();

        assert_eq!(
            unsafe {
                wxr_component_schema_add_field(
                    schema,
                    field.as_ptr(),
                    component::FieldType::I64,
                    getter,
                    1,
                )
            },
            0
        );

        unsafe {
            wxr_component_schema_free(schema);
        }
    }

    #[test]
    fn asset_schema_add_field_accepts_c_values() {
        let schema = wxr_asset_schema_new();
        let field = CString::new("value").unwrap();

        assert_eq!(
            unsafe {
                wxr_asset_schema_add_field(
                    schema,
                    field.as_ptr(),
                    component::FieldType::I64,
                    getter,
                )
            },
            0
        );

        unsafe {
            wxr_asset_schema_free(schema);
        }
    }
}
