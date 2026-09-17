use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::definitions::{
    Definition,
    error::ComponentFieldDefinitionError,
    fields::{ComponentFieldDefinition, TypeHint},
};

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn serializer(_: *const c_void) {}

unsafe extern "C" fn deserializer(_: *const c_void) {}

#[fixture]
fn field() -> ComponentFieldDefinition {
    static NAME: &[u8] = b"position\0";

    ComponentFieldDefinition {
        name: NAME.as_ptr().cast(),
        type_hint: TypeHint::F32 as u32,
        getter: Some(getter),
        mutable: 0,
        serializer: Some(serializer),
        deserializer: Some(deserializer),
    }
}

#[rstest]
fn validates_field(field: ComponentFieldDefinition) {
    assert!(unsafe { field.validate() }.is_ok());
}

#[rstest]
fn rejects_mutable_field_without_getter(mut field: ComponentFieldDefinition) {
    field.getter = None;
    field.mutable = 1;

    assert_eq!(
        unsafe { field.validate() },
        Err(ComponentFieldDefinitionError::MutableButNoGetter(
            "position".to_owned()
        ))
    );
}

#[rstest]
fn rejects_invalid_type_hint(mut field: ComponentFieldDefinition) {
    field.type_hint = u32::MAX;

    assert_eq!(
        unsafe { field.validate() },
        Err(ComponentFieldDefinitionError::InvalidTypeHint(u32::MAX))
    );
}

#[rstest]
fn field_without_serializer_is_not_serializable(mut field: ComponentFieldDefinition) {
    field.serializer = None;

    assert!(unsafe { field.validate() }.is_ok());
}

#[rstest]
fn field_without_deserializer_is_not_deserializable(mut field: ComponentFieldDefinition) {
    field.deserializer = None;

    assert!(unsafe { field.validate() }.is_ok());
}

#[rstest]
fn rejects_null_name(mut field: ComponentFieldDefinition) {
    field.name = std::ptr::null();

    assert_eq!(
        unsafe { field.validate() },
        Err(ComponentFieldDefinitionError::NameIsNull)
    );
}

#[rstest]
fn rejects_invalid_utf8_name(mut field: ComponentFieldDefinition) {
    let name = [0xff_u8, 0];
    field.name = name.as_ptr().cast();

    assert_eq!(
        unsafe { field.validate() },
        Err(ComponentFieldDefinitionError::NameIsNotUtf8)
    );
}

#[rstest]
fn rejects_empty_name(mut field: ComponentFieldDefinition) {
    let name = [0_u8];
    field.name = name.as_ptr().cast();

    assert_eq!(
        unsafe { field.validate() },
        Err(ComponentFieldDefinitionError::NameIsEmpty)
    );
}
