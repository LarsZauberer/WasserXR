use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::definitions::{
    Definition,
    error::AssetFieldDefinitionError,
    fields::{AssetFieldDefinition, TypeHint},
};

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

#[fixture]
fn field() -> AssetFieldDefinition {
    static NAME: &[u8] = b"material\0";

    AssetFieldDefinition {
        name: NAME.as_ptr().cast(),
        type_hint: TypeHint::Usize as u32,
        getter: Some(getter),
    }
}

#[rstest]
fn validates_field(field: AssetFieldDefinition) {
    assert!(unsafe { field.validate() }.is_ok());
}

#[rstest]
fn rejects_missing_getter(mut field: AssetFieldDefinition) {
    field.getter = None;

    assert_eq!(
        unsafe { field.validate() },
        Err(AssetFieldDefinitionError::GetterIsNull(
            "material".to_owned()
        ))
    );
}

#[rstest]
fn rejects_invalid_type_hint(mut field: AssetFieldDefinition) {
    field.type_hint = u32::MAX;

    assert_eq!(
        unsafe { field.validate() },
        Err(AssetFieldDefinitionError::InvalidTypeHint(u32::MAX))
    );
}
