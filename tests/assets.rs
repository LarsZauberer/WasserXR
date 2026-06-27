use std::{
    fs,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use wasserxr::{
    asset_type, asset_type_creator,
    error::{AssetError, SceneError},
    scene::Scene,
};

static ASSET_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[asset_type]
pub struct MacroFileAsset {
    content: String,
    bytes: usize,

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
fn asset_query_supports_tuple_fields() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = temp_asset_file("tuple", "tuple content");
    let mut scene = Scene::new();

    let (content, bytes) = scene
        .asset_query::<(&String, &usize)>("MacroFileAsset", &path, &["content", "bytes"])
        .unwrap();

    assert_eq!(content, "tuple content");
    assert_eq!(*bytes, "tuple content".len());
}

#[test]
fn asset_query_rejects_missing_file() {
    let _guard = ASSET_TEST_LOCK.lock().unwrap();
    CREATE_COUNT.store(0, Ordering::Relaxed);

    let path = missing_asset_file("invalid");
    let mut scene = Scene::new();

    assert_eq!(
        scene.asset_query::<(&String,)>("MacroFileAsset", &path, &["content"]),
        Err(SceneError::AssetError(AssetError::InvalidAsset))
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
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    );
}
