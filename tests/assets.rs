use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use wasserxr::{
    definitions::{
        assets::AssetDefinition, fields::AssetFieldDefinition, plugins::PluginDefinition,
    },
    errors::{AssetError, SceneError},
    ids::{AssetFieldTypeID, AssetTypeID, PluginID},
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
    systems: std::ptr::null(),
    system_count: 0,
};

fn scene() -> Scene {
    let scene = Scene::new();
    unsafe { scene.load_static_plugin(ASSET_PLUGIN) }.unwrap();
    scene
}

fn asset_type(scene: &Scene, name: &str) -> Result<(PluginID, AssetTypeID), SceneError> {
    let plugin = scene
        .resolve_plugin_id("AssetPlugin")
        .ok_or(SceneError::AssetNotFound)?;
    Ok((plugin, scene.resolve_asset_type_id(plugin, name)?))
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
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();

    assert!(matches!(
        scene.resolve_asset_id(plugin, asset_type, "first"),
        Err(SceneError::AssetNotFound)
    ));

    let id = scene.get_asset_id(plugin, asset_type, "first").unwrap();
    assert_eq!(
        scene.resolve_asset_id(plugin, asset_type, "first").unwrap(),
        id
    );
    assert_eq!(scene.get_asset_id(plugin, asset_type, "first").unwrap(), id);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn different_data_strings_create_distinct_assets() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();

    let first = scene.get_asset_id(plugin, asset_type, "first").unwrap();
    let second = scene.get_asset_id(plugin, asset_type, "second").unwrap();
    assert_ne!(first, second);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}

/// Whole-asset pointers preserve request order and repeat cached identities.
#[test]
fn asset_query_returns_complete_assets_in_request_order() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();
    let field_type = scene
        .resolve_asset_field_type_id(plugin, asset_type, "value")
        .unwrap();
    let first_asset = scene.get_asset_id(plugin, asset_type, "first").unwrap();
    scene
        .resolve_asset_field_id(first_asset, field_type)
        .unwrap();
    let requests = [
        (plugin, asset_type, "second"),
        (plugin, asset_type, "first"),
        (plugin, asset_type, "second"),
    ];

    let mut first_pointer = std::ptr::null();
    scene
        .query_assets(&[(plugin, asset_type, "first")], |pointers| {
            first_pointer = pointers[0];
        })
        .unwrap();
    let mut calls = 0;
    scene
        .query_assets(&requests, |assets| {
            calls += 1;
            assert_eq!(assets.len(), 3);
            assert_eq!(assets[0], assets[2]);
            assert_eq!(assets[1], first_pointer);
            assert_ne!(assets[0], assets[1]);
            for value in assets {
                assert_eq!(unsafe { (*value.cast::<TestAsset>()).value }, 42);
            }
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn missing_asset_field_is_rejected() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();
    let asset = scene.get_asset_id(plugin, asset_type, "field").unwrap();

    assert!(matches!(
        scene.resolve_asset_field_id(asset, AssetFieldTypeID::default()),
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    ));
}

/// Empty queries call once; failures skip the callback and release cache locks.
#[test]
fn asset_query_empty_and_failure_behavior() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, valid) = asset_type(&scene, "TestAsset").unwrap();
    let (_, failing) = asset_type(&scene, "FailingAsset").unwrap();
    let mut calls = 0;
    scene
        .query_assets(&[], |pointers| {
            calls += 1;
            assert!(pointers.is_empty());
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert!(matches!(
        scene.query_assets(&[(plugin, valid, "ok"), (plugin, failing, "bad")], |_| {
            panic!("failed query called callback");
        }),
        Err(SceneError::AssetError(AssetError::CreationFailure))
    ));
    assert!(scene.resolve_asset_id(plugin, valid, "ok").is_ok());
    assert!(matches!(
        scene.query_assets(&[(plugin, AssetTypeID::default(), "missing")], |_| {
            panic!("invalid query called callback");
        }),
        Err(SceneError::AssetNotFound)
    ));
    scene.reset().unwrap();
}

/// Reset cannot destroy data during a callback; a callback panic releases its
/// lock.
#[test]
fn asset_query_keeps_data_alive_until_callback_finishes() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = Arc::new(scene());
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();
    let (locked, locked_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let reader_scene = Arc::clone(&scene);
    let reader = thread::spawn(move || {
        reader_scene
            .query_assets(&[(plugin, asset_type, "first")], |pointers| {
                locked.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                assert_eq!(unsafe { (*pointers[0].cast::<TestAsset>()).value }, 42);
                assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 0);
            })
            .unwrap();
    });
    locked_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let (started, started_rx) = mpsc::channel();
    let (reset, reset_rx) = mpsc::channel();
    let reset_scene = Arc::clone(&scene);
    let resetter = thread::spawn(move || {
        started.send(()).unwrap();
        reset_scene.reset().unwrap();
        reset.send(()).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(reset_rx.recv_timeout(Duration::from_millis(100)).is_err());
    release.send(()).unwrap();
    reset_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    reader.join().unwrap();
    resetter.join().unwrap();
    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 1);

    let panic = std::panic::catch_unwind(|| {
        scene
            .query_assets(&[(plugin, asset_type, "again")], |_| {
                panic!("callback panic")
            })
            .unwrap();
    });
    assert!(panic.is_err());
    scene.reset().unwrap();
    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn concurrent_lookup_creates_one_cached_asset() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();

    let ids = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| scene.get_asset_id(plugin, asset_type, "shared").unwrap()))
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
    let (plugin, asset_type) = asset_type(&scene, "FailingAsset").unwrap();

    for _ in 0..2 {
        assert!(matches!(
            scene.get_asset_id(plugin, asset_type, "bad"),
            Err(SceneError::AssetError(AssetError::CreationFailure))
        ));
    }

    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
    assert!(matches!(
        scene.resolve_asset_id(plugin, asset_type, "bad"),
        Err(SceneError::AssetNotFound)
    ));
}

#[test]
fn unknown_asset_type_is_rejected() {
    let scene = scene();

    assert!(matches!(
        asset_type(&scene, "UnknownAsset"),
        Err(SceneError::AssetNotFound)
    ));
}

#[test]
fn cached_assets_are_destroyed_with_the_scene() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();

    {
        let scene = scene();
        let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();
        scene.get_asset_id(plugin, asset_type, "first").unwrap();
        scene.get_asset_id(plugin, asset_type, "second").unwrap();
    }

    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn reset_destroys_cached_assets() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let (plugin, asset_type) = asset_type(&scene, "TestAsset").unwrap();

    scene.get_asset_id(plugin, asset_type, "first").unwrap();
    scene.reset().unwrap();

    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 1);
    assert!(matches!(
        scene.resolve_asset_id(plugin, asset_type, "first"),
        Err(SceneError::AssetNotFound)
    ));
    scene.get_asset_id(plugin, asset_type, "first").unwrap();
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}
