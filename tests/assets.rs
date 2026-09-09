use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use wasserxr::{
    definitions::{
        assets::AssetDefinition, fields::AssetFieldDefinition, plugins::PluginDefinition,
    },
    errors::{AssetError, SceneError},
    scene::Scene,
    utils::version::Version,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());
static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);
static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
struct TestAsset {
    value: usize,
}

unsafe extern "C" fn create_asset() -> *mut c_void {
    CREATE_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(TestAsset { value: 42 })).cast()
}

unsafe extern "C" fn fail_to_create_asset() -> *mut c_void {
    CREATE_COUNT.fetch_add(1, Ordering::Relaxed);
    null_mut()
}

unsafe extern "C" fn destroy_asset(data: *mut c_void) {
    DESTROY_COUNT.fetch_add(1, Ordering::Relaxed);
    unsafe { drop(Box::from_raw(data.cast::<TestAsset>())) };
}

unsafe extern "C" fn unused_destroyer(_: *mut c_void) {}

unsafe extern "C" fn get_value(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*(data.cast::<TestAsset>())).value)
            .cast_mut()
            .cast()
    }
}

const VALUE_FIELD: AssetFieldDefinition = AssetFieldDefinition {
    name: c"value".as_ptr(),
    getter: Some(get_value),
};

const ASSETS: [AssetDefinition; 2] = [
    AssetDefinition {
        name: c"TestAsset".as_ptr(),
        creator: Some(create_asset),
        destroyer: Some(destroy_asset),
        fields: &VALUE_FIELD,
        field_count: 1,
    },
    AssetDefinition {
        name: c"FailingAsset".as_ptr(),
        creator: Some(fail_to_create_asset),
        destroyer: Some(unused_destroyer),
        fields: std::ptr::null(),
        field_count: 0,
    },
];

const ASSET_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"AssetPlugin".as_ptr(),
    engine_version: Version {
        major: 0,
        minor: 2,
        patch: 0,
    },
    components: std::ptr::null(),
    component_count: 0,
    assets: ASSETS.as_ptr(),
    asset_count: ASSETS.len(),
};

fn scene() -> Scene {
    let scene = Scene::new();
    unsafe { scene.load_static_plugin(ASSET_PLUGIN) }.unwrap();
    scene
}

fn reset_counts() {
    CREATE_COUNT.store(0, Ordering::Relaxed);
    DESTROY_COUNT.store(0, Ordering::Relaxed);
}

#[test]
fn asset_is_created_on_demand_and_cached() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();

    assert!(matches!(
        scene.resolve_asset_id("TestAsset", "first"),
        Err(SceneError::AssetNotFound)
    ));

    let id = scene.get_asset_id("TestAsset", "first").unwrap();
    assert_eq!(scene.resolve_asset_id("TestAsset", "first").unwrap(), id);
    assert_eq!(scene.get_asset_id("TestAsset", "first").unwrap(), id);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn different_data_strings_create_distinct_assets() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();

    let first = scene.get_asset_id("TestAsset", "first").unwrap();
    let second = scene.get_asset_id("TestAsset", "second").unwrap();
    assert_ne!(first, second);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn asset_field_can_be_read() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset = scene.get_asset_id("TestAsset", "field").unwrap();
    let field = scene.resolve_asset_field_id(asset, "value").unwrap();

    let value = scene
        .query_asset_field(asset, field)
        .unwrap()
        .cast::<usize>();
    assert_eq!(unsafe { *value }, 42);
}

#[test]
fn missing_asset_field_is_rejected() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset = scene.get_asset_id("TestAsset", "field").unwrap();

    assert!(matches!(
        scene.resolve_asset_field_id(asset, "missing"),
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    ));
}

#[test]
fn concurrent_lookup_creates_one_cached_asset() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();

    let ids = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| scene.get_asset_id("TestAsset", "shared").unwrap()))
            .collect();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(ids.iter().all(|id| *id == ids[0]));
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn failed_creation_is_not_cached() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();

    for _ in 0..2 {
        assert!(matches!(
            scene.get_asset_id("FailingAsset", "bad"),
            Err(SceneError::AssetError(AssetError::CreationFailure))
        ));
    }

    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
    assert!(matches!(
        scene.resolve_asset_id("FailingAsset", "bad"),
        Err(SceneError::AssetNotFound)
    ));
}

#[test]
fn unknown_asset_type_is_rejected() {
    let scene = scene();

    assert!(matches!(
        scene.get_asset_id("UnknownAsset", "anything"),
        Err(SceneError::AssetNotFound)
    ));
}

#[test]
fn cached_assets_are_destroyed_with_the_scene() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();

    {
        let scene = scene();
        scene.get_asset_id("TestAsset", "first").unwrap();
        scene.get_asset_id("TestAsset", "second").unwrap();
    }

    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 2);
}
