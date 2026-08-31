use std::path::Path;

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::error::{
        ComponentDefinitionError, ComponentFieldDefinitionError, PluginDefinitionError,
    },
    errors::{PluginCompatibilityError, PluginError, SceneError},
    scene::Scene,
};

#[path = "utils/plugin_compile.rs"]
mod plugin_compile;

#[fixture]
fn scene() -> Scene {
    Scene::new()
}

macro_rules! plugin_fixture {
    ($name:ident) => {
        #[fixture]
        fn $name() -> &'static Path {
            plugin_compile::compile_plugin(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("tests/plugins/{}.c", stringify!($name))),
            )
        }
    };
}

plugin_fixture!(valid_empty_plugin);
plugin_fixture!(invalid_field_component_plugin);

////////////////////////////////////////////////////////////////////

#[rstest]
fn add_dynamic_plugin(mut scene: Scene, valid_empty_plugin: &Path) {
    let plugin_id =
        unsafe { scene.load_plugin(valid_empty_plugin) }.expect("Failed to load valid plugin");

    let plugins = scene.get_plugins();
    assert!(
        plugins.contains(&plugin_id),
        "Plugin loaded successful, but didn't make it into the scene plugin storage"
    );

    let plugin_id2 = scene
        .get_plugin("MyPlugin")
        .expect("Failed to find the already added plugin");
    assert_eq!(plugin_id, plugin_id2);
}

#[rstest]
fn no_duplicate_dynamic_plugin_with_same_path(mut scene: Scene, valid_empty_plugin: &Path) {
    let plugin_id =
        unsafe { scene.load_plugin(valid_empty_plugin) }.expect("Failed to load valid plugin");
    let plugin_err = unsafe { scene.load_plugin(valid_empty_plugin) }
        .expect_err("Plugin shouldn't load since it is a duplicate");

    assert!(matches!(
        plugin_err,
        SceneError::PluginCompatibilityError(PluginCompatibilityError::PluginWithSameNameExists)
    ))
}

#[rstest]
fn no_duplicate_dynamic_plugin_with_same_name(mut scene: Scene) {
    let comp1 = valid_empty_plugin();
    let comp2 = valid_empty_plugin();
    assert_ne!(
        comp1, comp2,
        "The fixture didn't produce two different compilations"
    );

    let plugin_id = unsafe { scene.load_plugin(comp1) }.expect("Failed to load valid plugin");
    let plugin_err = unsafe { scene.load_plugin(comp2) }
        .expect_err("Plugin shouldn't load since it is a duplicate");

    assert!(matches!(
        plugin_err,
        SceneError::PluginCompatibilityError(PluginCompatibilityError::PluginWithSameNameExists)
    ))
}

#[rstest]
fn no_invalid_plugin_dynamic_load(mut scene: Scene, invalid_field_component_plugin: &Path) {
    let plugin_err = unsafe { scene.load_plugin(invalid_field_component_plugin) }
        .expect_err("Loaded invalid plugin");
    assert!(matches!(
        plugin_err,
        SceneError::PluginError(PluginError::DefinitionValidationError(
            PluginDefinitionError::ComponentInvalid(
                plugin_name,
                ComponentDefinitionError::FieldInvalid(
                    component_name,
                    ComponentFieldDefinitionError::MutableButNoGetter(field_name)
                )
            )
        )) if plugin_name == "MyPlugin"
            && component_name == "MyComponent"
            && field_name == "MyField"
    ))
}
