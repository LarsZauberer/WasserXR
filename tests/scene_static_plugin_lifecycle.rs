use std::{ffi::c_void, ptr::null_mut};

use rstest::rstest;
use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    errors::{PluginCompatibilityError, SceneError},
    scene::Scene,
    utils::version::Version,
};

unsafe extern "C" fn simple_creator() -> *mut c_void {
    null_mut()
}

unsafe extern "C" fn simple_destroyer(_: *mut c_void) {}

unsafe extern "C" fn simple_getter(_: *const c_void) -> *mut c_void {
    null_mut()
}

const COMPATIBLE_ENGINE_VERSION: Version = Version {
    major: 0,
    minor: 2,
    patch: 0,
};

const VALID_EMPTY_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: std::ptr::null(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
};

const VALID_EMPTY_COMPONENT: ComponentDefinition = ComponentDefinition {
    name: c"MyComponent".as_ptr(),
    creator: Some(simple_creator),
    destroyer: Some(simple_destroyer),
    fields: std::ptr::null(),
    field_count: 0,
};

const VALID_EMPTY_COMPONENT_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: &VALID_EMPTY_COMPONENT,
    component_count: 1,
    assets: std::ptr::null(),
    asset_count: 0,
};

const VALID_COMPONENT_FIELD: ComponentFieldDefinition = ComponentFieldDefinition {
    name: c"MyField".as_ptr(),
    getter: Some(simple_getter),
    mutable: 1,
    serializer: None,
    deserializer: None,
};

const VALID_COMPONENT_WITH_FIELD: ComponentDefinition = ComponentDefinition {
    name: c"MyComponent".as_ptr(),
    creator: Some(simple_creator),
    destroyer: Some(simple_destroyer),
    fields: &VALID_COMPONENT_FIELD,
    field_count: 1,
};

const VALID_COMPONENT_FIELD_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: &VALID_COMPONENT_WITH_FIELD,
    component_count: 1,
    assets: std::ptr::null(),
    asset_count: 0,
};

#[rstest]
#[case::empty_plugin(VALID_EMPTY_PLUGIN)]
#[case::empty_component_plugin(VALID_EMPTY_COMPONENT_PLUGIN)]
#[case::component_with_field_plugin(VALID_COMPONENT_FIELD_PLUGIN)]
fn add_simple_plugin(#[case] definition: PluginDefinition) {
    let mut scene = Scene::new();
    let plugin =
        unsafe { scene.load_static_plugin(definition) }.expect("Valid plugins should be loadable");
    let plugins = scene.get_plugins();
    assert!(
        plugins.contains(&plugin),
        "Scene doesn't contain newly added plugin"
    );
}

#[rstest]
#[case::empty_plugin(VALID_EMPTY_PLUGIN)]
#[case::empty_component_plugin(VALID_EMPTY_COMPONENT_PLUGIN)]
#[case::component_with_field_plugin(VALID_COMPONENT_FIELD_PLUGIN)]
fn cannot_add_duplicate_plugin(#[case] definition: PluginDefinition) {
    let mut scene = Scene::new();
    let _ =
        unsafe { scene.load_static_plugin(definition) }.expect("Valid plugins should be loadable");

    let err = unsafe { scene.load_static_plugin(definition) }
        .expect_err("Duplicate plugin shouldn't be addable");
    assert!(matches!(
        err,
        SceneError::PluginCompatibilityError(PluginCompatibilityError::PluginWithSameNameExists)
    ));
}

#[rstest]
#[case::empty_plugin(VALID_EMPTY_PLUGIN)]
#[case::empty_component_plugin(VALID_EMPTY_COMPONENT_PLUGIN)]
#[case::component_with_field_plugin(VALID_COMPONENT_FIELD_PLUGIN)]
fn get_plugin_name(#[case] definition: PluginDefinition) {
    let mut scene = Scene::new();
    let plugin =
        unsafe { scene.load_static_plugin(definition) }.expect("Valid plugins should be loadable");

    let plugin2 = scene
        .get_plugin("MyPlugin")
        .expect("Cannot find the name of the newly added plugin");
    assert_eq!(
        plugin, plugin2,
        "The handles of the same plugin are not identical"
    );
}
