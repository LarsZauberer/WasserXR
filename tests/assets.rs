use std::{
    fs,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use wasserxr::{
    asset_type, asset_type_creator,
    scene::{
        Scene, SceneError,
        assets::{AssetError, Schema},
        component::FieldType,
    },
};

static ASSET_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[asset_type]
pub struct MacroFileAsset {
    content: String,
    bytes: usize,
    position: [f64; 2],
    available: bool,

    #[none]
    #[allow(dead_code)]
    hidden: String,
}

#[asset_type_creator(MacroFileAsset)]
fn create_macro_file_asset(_scene: &mut Scene, path: &str) -> Option<MacroFileAsset> {
    CREATE_COUNT.fetch_add(1, Ordering::Relaxed);
    let content = fs::read_to_string(path).ok()?;
    let bytes = content.len();

    Some(MacroFileAsset {
        content,
        bytes,
        position: [1.0, 2.0],
        available: true,
        hidden: "hidden".to_owned(),
    })
}

fn temp_asset_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(format!("wasserxr-{name}-{}.txt", uuid::Uuid::now_v7()));
    fs::write(&path, content).unwrap();
    path.to_str().unwrap().to_owned()
}

fn missing_asset_file(name: &str) -> String {
    std::env::temp_dir()
        .join(format!(
            "wasserxr-missing-{name}-{}.txt",
            uuid::Uuid::now_v7()
        ))
        .to_str()
        .unwrap()
        .to_owned()
}

#[test]
fn asset_query_reads_file_content() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("content", "asset content");
    let mut scene = Scene::new();

    let (content,) = scene
        .asset_query::<(&String,)>("MacroFileAsset", &path, &["content"])
        .unwrap();

    assert_eq!(content, "asset content");
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn asset_query_reuses_cached_asset() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("cache", "cached content");
    let mut scene = Scene::new();

    {
        let (content,) = scene
            .asset_query::<(&String,)>("MacroFileAsset", &path, &["content"])
            .unwrap();
        assert_eq!(content, "cached content");
    }

    {
        let (bytes,) = scene
            .asset_query::<(&usize,)>("MacroFileAsset", &path, &["bytes"])
            .unwrap();
        assert_eq!(*bytes, "cached content".len());
    }

    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn ensure_asset_loaded_allows_read_only_asset_query() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("loaded", "loaded content");
    let mut scene = Scene::new();

    scene.ensure_asset_loaded("MacroFileAsset", &path).unwrap();

    let (content,) = scene
        .asset_query_loaded::<(&String,)>("MacroFileAsset", &path, &["content"])
        .unwrap();

    assert_eq!(content, "loaded content");
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn asset_query_loaded_allows_multiple_shared_asset_borrows() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let first_path = temp_asset_file("first-loaded", "first content");
    let second_path = temp_asset_file("second-loaded", "second content");
    let mut scene = Scene::new();

    scene
        .ensure_asset_loaded("MacroFileAsset", &first_path)
        .unwrap();
    scene
        .ensure_asset_loaded("MacroFileAsset", &second_path)
        .unwrap();

    let (first_content,) = scene
        .asset_query_loaded::<(&String,)>("MacroFileAsset", &first_path, &["content"])
        .unwrap();
    let (second_bytes,) = scene
        .asset_query_loaded::<(&usize,)>("MacroFileAsset", &second_path, &["bytes"])
        .unwrap();

    assert_eq!(first_content, "first content");
    assert_eq!(*second_bytes, "second content".len());
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn get_loaded_asset_data_strings_lists_loaded_assets() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let first_path = temp_asset_file("list-first", "first content");
    let second_path = temp_asset_file("list-second", "second content");
    let mut expected = vec![first_path.clone(), second_path.clone()];
    expected.sort();
    let mut scene = Scene::new();

    scene
        .ensure_asset_loaded("MacroFileAsset", &second_path)
        .unwrap();
    scene
        .ensure_asset_loaded("MacroFileAsset", &first_path)
        .unwrap();

    assert_eq!(
        scene.get_loaded_asset_data_strings("MacroFileAsset"),
        expected
    );
    assert_eq!(
        scene.get_loaded_asset_data_strings("MissingAsset"),
        Vec::<String>::new()
    );
}

#[test]
fn asset_query_supports_tuple_fields() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("tuple", "tuple content");
    let mut scene = Scene::new();

    let (content, bytes, available) = scene
        .asset_query::<(&String, &usize, &bool)>(
            "MacroFileAsset",
            &path,
            &["content", "bytes", "available"],
        )
        .unwrap();

    assert_eq!(content, "tuple content");
    assert_eq!(*bytes, "tuple content".len());
    assert!(*available);
}

#[test]
fn asset_macro_registers_vector_and_boolean_field_types() {
    let mut schema = Schema::default();
    unsafe { wxr_asset_schema_MacroFileAsset(&mut schema) };

    assert_eq!(schema.get_field_type("position"), Ok(FieldType::F64Vec2));
    assert_eq!(schema.get_field_type("available"), Ok(FieldType::Boolean));
}

#[test]
fn asset_query_rejects_missing_file() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = missing_asset_file("invalid");
    let mut scene = Scene::new();

    assert_eq!(
        scene.asset_query::<(&String,)>("MacroFileAsset", &path, &["content"]),
        Err(SceneError::Asset(AssetError::InvalidAsset))
    );
}

#[test]
fn asset_query_rejects_none_field() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("none", "hidden content");
    let mut scene = Scene::new();

    assert_eq!(
        scene.asset_query::<(&String,)>("MacroFileAsset", &path, &["hidden"]),
        Err(SceneError::Asset(AssetError::FieldNotFound))
    );
}
