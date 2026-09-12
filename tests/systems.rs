use wasserxr::{
    definitions::{
        Definition,
        error::{SystemDefinitionError, TypeIDRequestError},
        systems::SystemDefinition,
        type_id_requests::TypeIDRequests,
    },
    utils::ffi::StringError,
};

fn system() -> SystemDefinition {
    static NAME: &[u8] = b"render\0";

    SystemDefinition {
        name: NAME.as_ptr().cast(),
        requires: std::ptr::null(),
        requires_count: 0,
        wanted_by: std::ptr::null(),
        wanted_by_count: 0,
        type_id_requests: std::ptr::null(),
        type_id_request_count: 0,
    }
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
