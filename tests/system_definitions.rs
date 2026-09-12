//! Integration tests for system definitions and their resolved type IDs.

use wasserxr::{
    definitions::{
        Definition,
        error::{SystemDefinitionError, TypeIDRequestError},
        systems::SystemDefinition,
        type_id_requests::TypeIDRequests,
    },
    scene::{AssetFieldTypeID, AssetTypeID, ComponentTypeID, FieldTypeID, Scene, TypeID},
    utils::ffi::StringError,
};

unsafe extern "C" fn callback(_: *const Scene, _: *const TypeID, _: usize) {}

#[test]
fn type_ids_keep_their_requested_type() {
    let component = ComponentTypeID::default();
    let field = FieldTypeID::default();
    let asset = AssetTypeID::default();
    let asset_field = AssetFieldTypeID::default();

    assert_eq!(
        ComponentTypeID::try_from(TypeID::from(component)),
        Ok(component)
    );
    assert_eq!(FieldTypeID::try_from(TypeID::from(field)), Ok(field));
    assert_eq!(AssetTypeID::try_from(TypeID::from(asset)), Ok(asset));
    assert_eq!(
        AssetFieldTypeID::try_from(TypeID::from(asset_field)),
        Ok(asset_field)
    );
    assert!(AssetTypeID::try_from(TypeID::from(component)).is_err());
}

fn system() -> SystemDefinition {
    static NAME: &[u8] = b"render\0";

    SystemDefinition {
        name: NAME.as_ptr().cast(),
        attacher: Some(callback),
        runner: Some(callback),
        detacher: Some(callback),
        requires: std::ptr::null(),
        requires_count: 0,
        wanted_by: std::ptr::null(),
        wanted_by_count: 0,
        type_id_requests: std::ptr::null(),
        type_id_request_count: 0,
    }
}

#[test]
fn requires_only_the_runner() {
    let mut definition = system();
    definition.attacher = None;
    definition.detacher = None;
    assert!(unsafe { definition.validate() }.is_ok());

    definition.runner = None;
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::RunnerIsNull("render".to_owned()))
    );
}

#[test]
fn rejects_invalid_system_name() {
    let mut definition = system();
    definition.name = std::ptr::null();

    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::NameIsNull)
    );
}

#[test]
fn validates_system_and_all_type_id_requests() {
    static PHYSICS: &[u8] = b"physics\0";
    static PRESENT: &[u8] = b"present\0";
    static TRANSFORM: &[u8] = b"Transform\0";
    static POSITION: &[u8] = b"position\0";
    static MESH: &[u8] = b"Mesh\0";
    static VERTICES: &[u8] = b"vertices\0";

    let requires = [PHYSICS.as_ptr().cast()];
    let wanted_by = [PRESENT.as_ptr().cast()];
    let requests = [
        TypeIDRequests::ComponentTypeID {
            component: TRANSFORM.as_ptr().cast(),
        },
        TypeIDRequests::FieldTypeID {
            component: TRANSFORM.as_ptr().cast(),
            field: POSITION.as_ptr().cast(),
        },
        TypeIDRequests::AssetTypeID {
            asset: MESH.as_ptr().cast(),
        },
        TypeIDRequests::AssetFieldTypeID {
            asset: MESH.as_ptr().cast(),
            field: VERTICES.as_ptr().cast(),
        },
    ];
    let definition = SystemDefinition {
        requires: requires.as_ptr(),
        requires_count: requires.len(),
        wanted_by: wanted_by.as_ptr(),
        wanted_by_count: wanted_by.len(),
        type_id_requests: requests.as_ptr(),
        type_id_request_count: requests.len(),
        ..system()
    };

    assert!(unsafe { definition.validate() }.is_ok());
}

#[test]
fn rejects_null_list_pointers_with_nonzero_counts() {
    let mut definition = system();
    definition.requires_count = 1;
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::RequiresIsNull("render".to_owned()))
    );

    definition.requires_count = 0;
    definition.wanted_by_count = 1;
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::WantedByIsNull("render".to_owned()))
    );

    definition.wanted_by_count = 0;
    definition.type_id_request_count = 1;
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::TypeIDRequestsIsNull(
            "render".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_system_names_in_lists() {
    let invalid = [std::ptr::null()];
    let definition = SystemDefinition {
        requires: invalid.as_ptr(),
        requires_count: invalid.len(),
        ..system()
    };
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::RequiredSystemInvalid(
            "render".to_owned(),
            StringError::Null,
        ))
    );

    let definition = SystemDefinition {
        wanted_by: invalid.as_ptr(),
        wanted_by_count: invalid.len(),
        ..system()
    };
    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::WantedBySystemInvalid(
            "render".to_owned(),
            StringError::Null,
        ))
    );
}

#[test]
fn rejects_invalid_type_id_request_names() {
    static TRANSFORM: &[u8] = b"Transform\0";
    let request = TypeIDRequests::FieldTypeID {
        component: TRANSFORM.as_ptr().cast(),
        field: std::ptr::null(),
    };
    let definition = SystemDefinition {
        type_id_requests: &request,
        type_id_request_count: 1,
        ..system()
    };

    assert_eq!(
        unsafe { definition.validate() },
        Err(SystemDefinitionError::TypeIDRequestInvalid(
            "render".to_owned(),
            TypeIDRequestError::Field(StringError::Null),
        ))
    );
}
