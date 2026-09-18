use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::definitions::{
    Definition,
    components::ComponentDefinition,
    error::ComponentDefinitionError,
    fields::{ComponentFieldDefinition, TypeHint},
};

unsafe extern "C" fn creator() -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn destroyer(_: *mut c_void) {}

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

#[fixture]
fn component() -> ComponentDefinition {
    static NAME: &[u8] = b"Transform\0";

    ComponentDefinition {
        name: NAME.as_ptr().cast(),
        creator: Some(creator),
        destroyer: Some(destroyer),
        fields: std::ptr::null(),
        field_count: 0,
    }
}

#[rstest]
fn validates_component(component: ComponentDefinition) {
    assert!(unsafe { component.validate() }.is_ok());
}

#[rstest]
fn rejects_null_name(mut component: ComponentDefinition) {
    component.name = std::ptr::null();
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::NameIsNull)
    );
}

#[rstest]
fn rejects_invalid_utf8_name(mut component: ComponentDefinition) {
    let name = [0xff_u8, 0];
    component.name = name.as_ptr().cast();
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::NameIsNotUtf8)
    );
}

#[rstest]
fn rejects_empty_name(mut component: ComponentDefinition) {
    let name = [0_u8];
    component.name = name.as_ptr().cast();
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::NameIsEmpty)
    );
}

#[rstest]
fn rejects_missing_creator(mut component: ComponentDefinition) {
    component.creator = None;
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::CreatorIsNull(
            "Transform".to_owned()
        ))
    );
}

#[rstest]
fn rejects_missing_destroyer(mut component: ComponentDefinition) {
    component.destroyer = None;
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::DestroyerIsNull(
            "Transform".to_owned()
        ))
    );
}

#[rstest]
fn rejects_missing_fields(mut component: ComponentDefinition) {
    component.field_count = 1;
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::FieldsIsNull(
            "Transform".to_owned()
        ))
    );
}

#[rstest]
fn rejects_duplicate_field_names(mut component: ComponentDefinition) {
    let fields = [
        ComponentFieldDefinition {
            name: c"position".as_ptr(),
            type_hint: TypeHint::U32 as u32,
            getter: Some(getter),
            mutable: 0,
            serializer: None,
            deserializer: None,
        },
        ComponentFieldDefinition {
            name: c"position".as_ptr(),
            type_hint: TypeHint::U32 as u32,
            getter: Some(getter),
            mutable: 0,
            serializer: None,
            deserializer: None,
        },
    ];
    component.fields = fields.as_ptr();
    component.field_count = fields.len();

    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::DuplicateFieldName(
            "position".to_owned()
        ))
    );
}
