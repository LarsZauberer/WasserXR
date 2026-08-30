use std::ptr::null;

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::plugins::PluginDefinition,
    errors::{PluginCompatibilityError, SceneError},
    scene::Scene,
    utils::version::Version,
};

#[fixture]
fn simple_valid_plugin() -> PluginDefinition {
    let current_version = Version {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or_default(),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or_default(),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or_default(),
    };

    PluginDefinition {
        name: c"MyPlugin".as_ptr(),
        engine_version: current_version,
        components: null(),
        component_count: 0,
    }
}

#[rstest]
fn add_simple_plugin(simple_valid_plugin: PluginDefinition) {
    let mut scene = Scene::new();
    let plugin = unsafe { scene.load_static_plugin(simple_valid_plugin) }
        .expect("Valid plugins should be loadable");
    let plugins = scene.get_plugins();
    assert!(
        plugins.contains(&plugin),
        "Scene doesn't contain newly added plugin"
    );
}

#[rstest]
fn cannot_add_duplicate_plugin(simple_valid_plugin: PluginDefinition) {
    let mut scene = Scene::new();
    let _ = unsafe { scene.load_static_plugin(simple_valid_plugin) }
        .expect("Valid plugins should be loadable");

    let err = unsafe { scene.load_static_plugin(simple_valid_plugin) }
        .expect_err("Duplicate plugin shouldn't be addable");
    assert!(matches!(
        err,
        SceneError::PluginCompatibilityError(PluginCompatibilityError::PluginWithSameNameExists)
    ));
}

#[rstest]
fn get_plugin_name(simple_valid_plugin: PluginDefinition) {
    let mut scene = Scene::new();
    let plugin = unsafe { scene.load_static_plugin(simple_valid_plugin) }
        .expect("Valid plugins should be loadable");

    let plugin2 = scene
        .get_plugin("MyPlugin")
        .expect("Cannot find the name of the newly added plugin");
    assert_eq!(
        plugin, plugin2,
        "The handles of the same plugin are not identical"
    );
}
