use std::ffi::c_void;

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        Definition,
        assets::AssetDefinition,
        error::{AssetDefinitionError, PluginDefinitionError, SystemDefinitionError},
        plugins::PluginDefinition,
        systems::SystemDefinition,
    },
    utils::version::Version,
};

unsafe extern "C" fn destroyer(_: *mut c_void) {}

fn compatible_version() -> Version {
    Version {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
    }
}

#[fixture]
fn plugin() -> PluginDefinition {
    static NAME: &[u8] = b"example\0";

    PluginDefinition {
        name: NAME.as_ptr().cast(),
        engine_version: compatible_version(),
        components: std::ptr::null(),
        component_count: 0,
        assets: std::ptr::null(),
        asset_count: 0,
        systems: std::ptr::null(),
        system_count: 0,
    }
}

#[rstest]
fn validates_plugin(plugin: PluginDefinition) {
    assert!(unsafe { plugin.validate() }.is_ok());
}

#[rstest]
fn rejects_incompatible_engine_version(mut plugin: PluginDefinition) {
    let incompatible_version = if plugin.engine_version.major == 0 {
        Version {
            minor: plugin.engine_version.minor + 1,
            ..plugin.engine_version
        }
    } else {
        Version {
            major: plugin.engine_version.major + 1,
            ..plugin.engine_version
        }
    };
    plugin.engine_version = incompatible_version;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::EngineVersionMismatch {
            name: "example".to_owned(),
            expected: compatible_version(),
            actual: incompatible_version,
        })
    );
}

#[rstest]
fn rejects_missing_components(mut plugin: PluginDefinition) {
    plugin.component_count = 1;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::ComponentsIsNull(
            "example".to_owned()
        ))
    );
}

#[rstest]
fn rejects_missing_assets(mut plugin: PluginDefinition) {
    plugin.asset_count = 1;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::AssetsIsNull("example".to_owned()))
    );
}

#[rstest]
fn rejects_missing_systems(mut plugin: PluginDefinition) {
    plugin.system_count = 1;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::SystemsIsNull("example".to_owned()))
    );
}

#[rstest]
fn rejects_invalid_system(mut plugin: PluginDefinition) {
    let system = SystemDefinition {
        name: std::ptr::null(),
        attacher: None,
        runner: None,
        detacher: None,
        requires: std::ptr::null(),
        requires_count: 0,
        wanted_by: std::ptr::null(),
        wanted_by_count: 0,
        type_id_requests: std::ptr::null(),
        type_id_request_count: 0,
    };
    plugin.systems = &system;
    plugin.system_count = 1;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::SystemInvalid(
            "example".to_owned(),
            SystemDefinitionError::NameIsNull,
        ))
    );
}

#[rstest]
fn rejects_invalid_asset(mut plugin: PluginDefinition) {
    static ASSET_NAME: &[u8] = b"Mesh\0";
    let asset = AssetDefinition {
        name: ASSET_NAME.as_ptr().cast(),
        creator: None,
        destroyer: Some(destroyer),
        fields: std::ptr::null(),
        field_count: 0,
    };
    plugin.assets = &asset;
    plugin.asset_count = 1;

    assert_eq!(
        unsafe { plugin.validate() },
        Err(PluginDefinitionError::AssetInvalid(
            "example".to_owned(),
            AssetDefinitionError::CreatorIsNull("Mesh".to_owned()),
        ))
    );
}
