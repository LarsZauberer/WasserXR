use std::{ffi::c_void, ptr};

use wasserxr::{
    definitions::{
        functions::FunctionDefinition, plugins::PluginDefinition, systems::SystemDefinition,
        type_id_requests::TypeIDRequests,
    },
    errors::SceneError,
    ids::{FunctionTypeID, TypeID},
    scene::Scene,
    utils::version::Version,
};

/// Adds one opaque integer argument into another and checks scene reentry.
unsafe extern "C" fn add(scene: *const Scene, arguments: *const *mut c_void, count: usize) {
    assert!(!scene.is_null());
    let scene = unsafe { &*scene };
    let plugin = scene.resolve_plugin_id("FunctionPlugin").unwrap();
    assert!(scene.resolve_function_type_id(plugin, "add").is_ok());
    assert_eq!(count, 2);
    let arguments = unsafe { std::slice::from_raw_parts(arguments, count) };
    let left = unsafe { &*(arguments[0] as *const i32) };
    let right = unsafe { &mut *(arguments[1] as *mut i32) };
    *right += left;
}

/// Supplies the system callback required for type ID request testing.
unsafe extern "C" fn run_system(_: *const Scene, _: *const TypeID, _: usize) {}

const FUNCTION: FunctionDefinition = FunctionDefinition {
    name: c"add".as_ptr(),
    function: Some(add),
};
const REQUEST: TypeIDRequests = TypeIDRequests::FunctionTypeID {
    function: c"add".as_ptr(),
};
const SYSTEM: SystemDefinition = SystemDefinition {
    name: c"caller".as_ptr(),
    attacher: None,
    runner: Some(run_system),
    detacher: None,
    requires: ptr::null(),
    requires_count: 0,
    wanted_by: ptr::null(),
    wanted_by_count: 0,
    type_id_requests: &REQUEST,
    type_id_request_count: 1,
};

#[test]
/// Resolves a function directly and through a system, then calls it.
fn runs_function_and_resolves_it_for_system() {
    let scene = Scene::new();
    let definition = PluginDefinition {
        name: c"FunctionPlugin".as_ptr(),
        engine_version: Version {
            major: 0,
            minor: 2,
            patch: 0,
        },
        components: ptr::null(),
        component_count: 0,
        assets: ptr::null(),
        asset_count: 0,
        systems: &SYSTEM,
        system_count: 1,
        functions: &FUNCTION,
        function_count: 1,
    };
    let plugin = unsafe { scene.load_static_plugin(definition) }.unwrap();
    let function = scene.resolve_function_type_id(plugin, "add").unwrap();
    assert_eq!(
        FunctionTypeID::try_from(TypeID::from(function)),
        Ok(function)
    );
    assert_eq!(
        scene
            .resolve_function_type_id(plugin, "missing")
            .unwrap_err()
            .to_string(),
        SceneError::FunctionNotFound.to_string()
    );

    let system = scene.resolve_system_type_id(plugin, "caller").unwrap();
    assert_eq!(
        scene.resolve_requested_type_ids(system).unwrap(),
        vec![TypeID::from(function)]
    );

    let left = 3_i32;
    let mut right = 7_i32;
    let arguments = [
        &left as *const i32 as *mut c_void,
        &mut right as *mut i32 as *mut c_void,
    ];
    unsafe { scene.run_function(function, &arguments) }.unwrap();
    assert_eq!(right, 10);

    let invalid = FunctionTypeID::try_from(TypeID::FunctionTypeID(0, 0)).unwrap();
    assert!(matches!(
        unsafe { scene.run_function(invalid, &[]) },
        Err(SceneError::FunctionNotFound)
    ));
}
