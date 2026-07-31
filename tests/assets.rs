//! Integration coverage for the `#[asset_type]` / `#[asset_type_creator]` macros
//! and scene asset queries. The `FileAsset` asset type lives in the fixture.

#[path = "../src/testing_plugin_fixture.rs"]
mod testing_plugin_fixture;

use std::{fs, sync::atomic::Ordering};

use rstest::rstest;
use testing_plugin_fixture::{self as fx, asset_state};
use wasserxr::{
    error::{AssetError, SceneError},
    scene::{Scene, assets::Schema, component::FieldType},
};

/// Writes a temp file and returns its path.
fn temp_asset_file(content: &str) -> String {
    let path = std::env::temp_dir().join(format!("wasserxr-{}.txt", uuid::Uuid::now_v7()));
    fs::write(&path, content).unwrap();
    path.to_str().unwrap().to_owned()
}

fn missing_asset_file() -> String {
    std::env::temp_dir()
        .join(format!("wasserxr-missing-{}.txt", uuid::Uuid::now_v7()))
        .to_str()
        .unwrap()
        .to_owned()
}

#[rstest]
fn asset_query_reads_fields_and_creates_once(#[from(asset_state)] _state: fx::AssetState) {
    let path = temp_asset_file("asset content");
    let mut scene = Scene::new();

    // A tuple query reads several field types at once.
    let (content, bytes, available) = scene
        .asset_query::<(&String, &usize, &bool)>(
            "FileAsset",
            &path,
            &["content", "bytes", "available"],
        )
        .unwrap();
    assert_eq!(content, "asset content");
    assert_eq!(*bytes, "asset content".len());
    assert!(*available);

    // A second query of the same asset reuses the cached instance.
    let (bytes,) = scene
        .asset_query::<(&usize,)>("FileAsset", &path, &["bytes"])
        .unwrap();
    assert_eq!(*bytes, "asset content".len());
    assert_eq!(fx::ASSET_CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[rstest]
fn ensure_asset_loaded_enables_read_only_queries_across_assets(
    #[from(asset_state)] _state: fx::AssetState,
) {
    let first = temp_asset_file("first content");
    let second = temp_asset_file("second content");
    let mut scene = Scene::new();

    scene.ensure_asset_loaded("FileAsset", &first).unwrap();
    scene.ensure_asset_loaded("FileAsset", &second).unwrap();

    let (first_content,) = scene
        .asset_query_loaded::<(&String,)>("FileAsset", &first, &["content"])
        .unwrap();
    let (second_bytes,) = scene
        .asset_query_loaded::<(&usize,)>("FileAsset", &second, &["bytes"])
        .unwrap();
    assert_eq!(first_content, "first content");
    assert_eq!(*second_bytes, "second content".len());
    assert_eq!(fx::ASSET_CREATE_COUNT.load(Ordering::Relaxed), 2);

    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(scene.get_loaded_asset_data_strings("FileAsset"), expected);
    assert_eq!(
        scene.get_loaded_asset_data_strings("MissingAsset"),
        Vec::<String>::new()
    );
}

#[rstest]
fn asset_macro_registers_vector_and_boolean_field_types() {
    let mut schema = Schema::default();
    unsafe { fx::wxr_asset_schema_FileAsset(&mut schema) };

    assert_eq!(schema.get_field_type("position"), Ok(FieldType::F64Vec2));
    assert_eq!(schema.get_field_type("available"), Ok(FieldType::Boolean));
}

#[rstest]
#[case(None, SceneError::AssetError(AssetError::InvalidAsset))]
#[case(Some("hidden"), SceneError::AssetError(AssetError::FieldNotFound))]
fn asset_query_rejects_bad_requests(
    #[from(asset_state)] _state: fx::AssetState,
    #[case] none_field: Option<&str>,
    #[case] expected: SceneError,
) {
    let mut scene = Scene::new();
    let (path, field) = match none_field {
        // Missing file: the asset creator fails.
        None => (missing_asset_file(), "content"),
        // Present file, but the requested field is `#[none]`.
        Some(field) => (temp_asset_file("hidden content"), field),
    };

    assert_eq!(
        scene.asset_query::<(&String,)>("FileAsset", &path, &[field]),
        Err(expected)
    );
}
