use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::definitions::{
    Definition, error::AssetFieldDefinitionError, fields::AssetFieldDefinition,
};

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

#[fixture]
fn field() -> AssetFieldDefinition {
    static NAME: &[u8] = b"material\0";

    AssetFieldDefinition {
        name: NAME.as_ptr().cast(),
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
