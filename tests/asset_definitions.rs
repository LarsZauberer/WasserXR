use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::definitions::{
    Definition,
    assets::AssetDefinition,
    error::{AssetDefinitionError, AssetFieldDefinitionError},
    fields::{AssetFieldDefinition, TypeHint},
};

unsafe extern "C" fn creator() -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn destroyer(_: *mut c_void) {}

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

#[fixture]
fn asset() -> AssetDefinition {
    static NAME: &[u8] = b"Mesh\0";

    AssetDefinition {
        name: NAME.as_ptr().cast(),
        creator: Some(creator),
        destroyer: Some(destroyer),
        fields: std::ptr::null(),
        field_count: 0,
    }
}

#[rstest]
fn validates_asset(asset: AssetDefinition) {
    assert!(unsafe { asset.validate() }.is_ok());
}

#[rstest]
fn rejects_missing_creator(mut asset: AssetDefinition) {
    asset.creator = None;

    assert_eq!(
        unsafe { asset.validate() },
        Err(AssetDefinitionError::CreatorIsNull("Mesh".to_owned()))
    );
}

#[rstest]
fn rejects_missing_destroyer(mut asset: AssetDefinition) {
    asset.destroyer = None;

    assert_eq!(
        unsafe { asset.validate() },
        Err(AssetDefinitionError::DestroyerIsNull("Mesh".to_owned()))
    );
}

#[rstest]
fn rejects_missing_fields(mut asset: AssetDefinition) {
    asset.field_count = 1;

    assert_eq!(
        unsafe { asset.validate() },
        Err(AssetDefinitionError::FieldsIsNull("Mesh".to_owned()))
    );
}

#[rstest]
fn rejects_invalid_field(mut asset: AssetDefinition) {
    static FIELD_NAME: &[u8] = b"vertices\0";
    let field = AssetFieldDefinition {
        name: FIELD_NAME.as_ptr().cast(),
        type_hint: TypeHint::Usize as u32,
        getter: None,
    };
    asset.fields = &field;
    asset.field_count = 1;

    assert_eq!(
        unsafe { asset.validate() },
        Err(AssetDefinitionError::FieldInvalid(
            "Mesh".to_owned(),
            AssetFieldDefinitionError::GetterIsNull("vertices".to_owned()),
        ))
    );
}

#[rstest]
fn rejects_duplicate_field_names(mut asset: AssetDefinition) {
    let fields = [
        AssetFieldDefinition {
            name: c"vertices".as_ptr(),
            type_hint: TypeHint::Usize as u32,
            getter: Some(getter),
        },
        AssetFieldDefinition {
            name: c"vertices".as_ptr(),
            type_hint: TypeHint::Usize as u32,
            getter: Some(getter),
        },
    ];
    asset.fields = fields.as_ptr();
    asset.field_count = fields.len();

    assert_eq!(
        unsafe { asset.validate() },
        Err(AssetDefinitionError::DuplicateFieldName(
            "vertices".to_owned()
        ))
    );
}
