use std::ffi::c_void;

use wasserxr::{
    definitions::{
        Definition,
        components::ComponentDefinition,
        error::{ComponentDefinitionError, MethodDefinitionError},
        methods::MethodDefinition,
        plugins::PluginDefinition,
    },
    scene::Scene,
    utils::version::Version,
};

/// Provides a component constructor for validation fixtures.
unsafe extern "C" fn creator() -> *mut c_void {
    std::ptr::null_mut()
}

/// Provides a component destructor for validation fixtures.
unsafe extern "C" fn destroyer(_: *mut c_void) {}

/// Provides a callback with the public method ABI.
unsafe extern "C" fn method(_: *const Scene, _: *mut c_void, _: *const *mut c_void, _: usize) {}

/// Builds a component without fields or methods.
fn component() -> ComponentDefinition {
    ComponentDefinition {
        name: c"Transform".as_ptr(),
        creator: Some(creator),
        destroyer: Some(destroyer),
        fields: std::ptr::null(),
        field_count: 0,
        methods: std::ptr::null(),
        method_count: 0,
    }
}

/// Accepts distinct method names on one component.
#[test]
fn accepts_multiple_methods() {
    let methods = [
        MethodDefinition {
            name: c"move".as_ptr(),
            method: Some(method),
        },
        MethodDefinition {
            name: c"rotate".as_ptr(),
            method: Some(method),
        },
    ];
    let mut component = component();
    component.methods = methods.as_ptr();
    component.method_count = methods.len();

    assert_eq!(unsafe { component.validate() }, Ok(()));
}

/// Converts component methods through the public plugin loading path.
#[test]
fn loads_plugin_with_methods() {
    let methods = [MethodDefinition {
        name: c"move".as_ptr(),
        method: Some(method),
    }];
    let mut component = component();
    component.methods = methods.as_ptr();
    component.method_count = methods.len();
    let plugin = PluginDefinition {
        name: c"methods".as_ptr(),
        engine_version: Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        },
        components: &component,
        component_count: 1,
        assets: std::ptr::null(),
        asset_count: 0,
        systems: std::ptr::null(),
        system_count: 0,
        functions: std::ptr::null(),
        function_count: 0,
    };
    let scene = Scene::new();
    let id = unsafe { scene.load_static_plugin(plugin) }.unwrap();
    assert!(scene.get_plugins().contains(&id));
}

/// Rejects null, empty, and non-UTF-8 method names.
#[test]
fn rejects_invalid_method_names() {
    let invalid_utf8 = [0xff_u8, 0];
    let names = [
        (std::ptr::null(), MethodDefinitionError::NameIsNull),
        (c"".as_ptr(), MethodDefinitionError::NameIsEmpty),
        (
            invalid_utf8.as_ptr().cast(),
            MethodDefinitionError::NameIsNotUtf8,
        ),
    ];
    for (name, expected) in names {
        let definition = MethodDefinition {
            name,
            method: Some(method),
        };
        assert_eq!(unsafe { definition.validate() }, Err(expected));
    }
}

/// Rejects a null callback directly and when nested in a component.
#[test]
fn rejects_null_method_callback() {
    let definition = MethodDefinition {
        name: c"move".as_ptr(),
        method: None,
    };
    assert_eq!(
        unsafe { definition.validate() },
        Err(MethodDefinitionError::MethodIsNull("move".to_owned()))
    );

    let mut component = component();
    component.methods = &definition;
    component.method_count = 1;
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::MethodInvalid(
            "Transform".to_owned(),
            MethodDefinitionError::MethodIsNull("move".to_owned()),
        ))
    );
}

/// Rejects a null method array when its count is positive.
#[test]
fn rejects_null_method_array() {
    let mut component = component();
    component.method_count = 1;
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::MethodsIsNull(
            "Transform".to_owned()
        ))
    );
}

/// Rejects duplicate names before they can enter a manifest.
#[test]
fn rejects_duplicate_method_names() {
    let methods = [
        MethodDefinition {
            name: c"move".as_ptr(),
            method: Some(method),
        },
        MethodDefinition {
            name: c"move".as_ptr(),
            method: Some(method),
        },
    ];
    let mut component = component();
    component.methods = methods.as_ptr();
    component.method_count = methods.len();
    assert_eq!(
        unsafe { component.validate() },
        Err(ComponentDefinitionError::DuplicateMethodName(
            "move".to_owned()
        ))
    );
}
