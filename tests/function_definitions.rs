use std::ptr;

use wasserxr::{
    definitions::{
        Definition,
        error::{FunctionDefinitionError, PluginDefinitionError, TypeIDRequestError},
        functions::FunctionDefinition,
        plugins::PluginDefinition,
        type_id_requests::TypeIDRequests,
    },
    scene::Scene,
    utils::{ffi::StringError, version::Version},
};

/// Supplies a valid callback for definition validation.
unsafe extern "C" fn callback(_: *const Scene, _: *const *mut std::ffi::c_void, _: usize) {}

/// Builds a valid function definition.
fn function() -> FunctionDefinition {
    FunctionDefinition {
        name: c"render".as_ptr(),
        function: Some(callback),
    }
}

/// Builds an empty plugin definition.
fn plugin() -> PluginDefinition {
    PluginDefinition {
        name: c"functions".as_ptr(),
        engine_version: Version {
            major: 0,
            minor: 2,
            patch: 0,
        },
        components: ptr::null(),
        component_count: 0,
        assets: ptr::null(),
        asset_count: 0,
        systems: ptr::null(),
        system_count: 0,
        functions: ptr::null(),
        function_count: 0,
    }
}

#[test]
/// Rejects missing callbacks and invalid names.
fn rejects_null_callback_and_invalid_name() {
    let mut definition = function();
    assert!(unsafe { definition.validate() }.is_ok());
    definition.function = None;
    assert_eq!(
        unsafe { definition.validate() },
        Err(FunctionDefinitionError::FunctionIsNull("render".into()))
    );
    definition.name = ptr::null();
    assert_eq!(
        unsafe { definition.validate() },
        Err(FunctionDefinitionError::NameIsNull)
    );
}

#[test]
/// Rejects malformed function arrays and duplicate names.
fn plugin_rejects_null_array_and_duplicate_function_names() {
    let mut definition = plugin();
    definition.function_count = 1;
    assert_eq!(
        unsafe { definition.validate() },
        Err(PluginDefinitionError::FunctionsIsNull("functions".into()))
    );

    let functions = [function(), function()];
    definition.functions = functions.as_ptr();
    definition.function_count = functions.len();
    assert_eq!(
        unsafe { definition.validate() },
        Err(PluginDefinitionError::DuplicateFunctionName(
            "render".into()
        ))
    );
}

#[test]
/// Preserves callback validation errors at plugin level.
fn plugin_reports_invalid_function_callback() {
    let function = FunctionDefinition {
        function: None,
        ..function()
    };
    let mut definition = plugin();
    definition.functions = &function;
    definition.function_count = 1;
    assert_eq!(
        unsafe { definition.validate() },
        Err(PluginDefinitionError::FunctionInvalid(
            "functions".into(),
            FunctionDefinitionError::FunctionIsNull("render".into())
        ))
    );
}

#[test]
/// Validates requested function names.
fn function_type_id_request_validates_name() {
    let valid = TypeIDRequests::FunctionTypeID {
        function: c"render".as_ptr(),
    };
    assert!(unsafe { valid.validate() }.is_ok());
    let invalid = TypeIDRequests::FunctionTypeID {
        function: ptr::null(),
    };
    assert_eq!(
        unsafe { invalid.validate() },
        Err(TypeIDRequestError::Function(StringError::Null))
    );
}
