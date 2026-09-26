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
        assets::AssetDefinition,
        fields::{AssetFieldDefinition, TypeHint},
        plugins::PluginDefinition,
    },
    errors::{AssetError, SceneError},
    ids::{AssetFieldTypeID, AssetTypeID, TypeID},
    scene::Scene,
    utils::version::Version,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());
static CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);
static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
struct TestAsset {
    value: usize,
    other: usize,
}

unsafe extern "C" fn create_asset() -> *mut c_void {
    CREATE_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(TestAsset {
        value: 42,
        other: 7,
    }))
    .cast()
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

unsafe extern "C" fn get_other(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*(data.cast::<TestAsset>())).other)
            .cast_mut()
            .cast()
    }
}

const FIELDS: [AssetFieldDefinition; 2] = [
    AssetFieldDefinition {
        name: c"value".as_ptr(),
        type_hint: TypeHint::Usize as u32,
        getter: Some(get_value),
    },
    AssetFieldDefinition {
        name: c"other".as_ptr(),
        type_hint: TypeHint::Usize as u32,
        getter: Some(get_other),
    },
];

const ASSETS: [AssetDefinition; 2] = [
    AssetDefinition {
        name: c"TestAsset".as_ptr(),
        creator: Some(create_asset),
        destroyer: Some(destroy_asset),
        fields: FIELDS.as_ptr(),
        field_count: FIELDS.len(),
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
    functions: std::ptr::null(),
    function_count: 0,
};

fn scene() -> Scene {
    let scene = Scene::new();
    unsafe { scene.load_static_plugin(ASSET_PLUGIN) }.unwrap();
    scene
}

fn asset_type(scene: &Scene, name: &str) -> Result<AssetTypeID, SceneError> {
    let plugin = scene
        .resolve_plugin_id("AssetPlugin")
        .ok_or(SceneError::AssetNotFound)?;
    scene.resolve_asset_type_id(plugin, name)
}

fn reset_counts() {
    CREATE_COUNT.store(0, Ordering::Relaxed);
    DESTROY_COUNT.store(0, Ordering::Relaxed);
}

#[test]
fn resolves_asset_field_type_hint() {
    let scene = scene();
    let asset = asset_type(&scene, "TestAsset").unwrap();
    let field = scene.resolve_asset_field_type_id(asset, "value").unwrap();

    assert_eq!(scene.get_asset_field_type(field).unwrap(), TypeHint::Usize);
    let TypeID::AssetTypeID(plugin, asset_slot) = TypeID::from(asset) else {
        unreachable!()
    };
    let missing =
        AssetFieldTypeID::try_from(TypeID::AssetFieldTypeID(plugin, asset_slot, u64::MAX)).unwrap();
    assert!(matches!(
        scene.get_asset_field_type(missing),
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    ));
}

#[test]
fn asset_is_created_on_demand_and_cached() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset_type = asset_type(&scene, "TestAsset").unwrap();

    assert!(matches!(
        scene.resolve_asset_id(asset_type, "first"),
        Err(SceneError::AssetNotFound)
    ));

    let id = scene.get_asset_id(asset_type, "first").unwrap();
    assert_eq!(scene.resolve_asset_id(asset_type, "first").unwrap(), id);
    assert_eq!(scene.get_asset_id(asset_type, "first").unwrap(), id);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn different_data_strings_create_distinct_assets() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset_type = asset_type(&scene, "TestAsset").unwrap();

    let first = scene.get_asset_id(asset_type, "first").unwrap();
    let second = scene.get_asset_id(asset_type, "second").unwrap();
    assert_ne!(first, second);
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}

/// Field pointers preserve request and field order, including duplicates.
#[test]
fn asset_query_returns_fields_in_request_order() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset_type = asset_type(&scene, "TestAsset").unwrap();
    let field_type = scene
        .resolve_asset_field_type_id(asset_type, "value")
        .unwrap();
    let other_type = scene
        .resolve_asset_field_type_id(asset_type, "other")
        .unwrap();
    let first_asset = scene.get_asset_id(asset_type, "first").unwrap();
    scene
        .resolve_asset_field_id(first_asset, field_type)
        .unwrap();
    let requests = [
        (asset_type, "second", &[other_type, field_type][..]),
        (asset_type, "first", &[field_type][..]),
        (asset_type, "second", &[field_type, field_type][..]),
    ];

    let mut first_pointer = std::ptr::null();
    scene
        .with_asset_fields(&[(asset_type, "first", &[field_type])], |pointers| {
            first_pointer = pointers[0];
        })
        .unwrap();
    let mut calls = 0;
    scene
        .with_asset_fields(&requests, |fields| {
            calls += 1;
            assert_eq!(fields.len(), 5);
            assert_eq!(unsafe { *fields[0].cast::<usize>() }, 7);
            assert_eq!(unsafe { *fields[1].cast::<usize>() }, 42);
            assert_eq!(fields[1], fields[3]);
            assert_eq!(fields[3], fields[4]);
            assert_eq!(fields[2], first_pointer);
            assert_ne!(fields[1], fields[2]);
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
    let asset_type = asset_type(&scene, "TestAsset").unwrap();
    let asset = scene.get_asset_id(asset_type, "field").unwrap();
    let TypeID::AssetTypeID(plugin, asset_slot) = TypeID::from(asset_type) else {
        unreachable!()
    };
    let missing =
        AssetFieldTypeID::try_from(TypeID::AssetFieldTypeID(plugin, asset_slot, u64::MAX)).unwrap();

    assert!(matches!(
        scene.resolve_asset_field_id(asset, missing),
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    ));
    assert!(matches!(
        scene.with_asset_fields(&[(asset_type, "field", &[missing])], |_| {
            panic!("missing field called callback")
        }),
        Err(SceneError::AssetError(AssetError::FieldNotFound))
    ));
}

/// Empty queries call once; failures skip the callback and release cache locks.
#[test]
fn asset_query_empty_and_failure_behavior() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let valid = asset_type(&scene, "TestAsset").unwrap();
    let failing = asset_type(&scene, "FailingAsset").unwrap();
    let mut calls = 0;
    scene
        .with_asset_fields(&[], |pointers| {
            calls += 1;
            assert!(pointers.is_empty());
        })
        .unwrap();
    assert_eq!(calls, 1);
    scene
        .with_asset_fields(&[(valid, "fieldless", &[])], |pointers| {
            calls += 1;
            assert!(pointers.is_empty());
        })
        .unwrap();
    assert_eq!(calls, 2);
    assert!(scene.resolve_asset_id(valid, "fieldless").is_ok());
    assert!(matches!(
        scene.with_asset_fields(&[(valid, "ok", &[]), (failing, "bad", &[])], |_| {
            panic!("failed query called callback");
        }),
        Err(SceneError::AssetError(AssetError::CreationFailure))
    ));
    assert!(scene.resolve_asset_id(valid, "ok").is_ok());
    let TypeID::AssetTypeID(plugin, _) = TypeID::from(valid) else {
        unreachable!()
    };
    let missing = AssetTypeID::try_from(TypeID::AssetTypeID(plugin, u64::MAX)).unwrap();
    assert!(matches!(
        scene.with_asset_fields(&[(missing, "missing", &[])], |_| {
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
    let asset_type = asset_type(&scene, "TestAsset").unwrap();
    let field_type = scene
        .resolve_asset_field_type_id(asset_type, "value")
        .unwrap();
    let (locked, locked_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let reader_scene = Arc::clone(&scene);
    let reader = thread::spawn(move || {
        reader_scene
            .with_asset_fields(&[(asset_type, "first", &[field_type])], |pointers| {
                locked.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                assert_eq!(unsafe { *pointers[0].cast::<usize>() }, 42);
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
            .with_asset_fields(&[(asset_type, "again", &[field_type])], |_| {
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
    let asset_type = asset_type(&scene, "TestAsset").unwrap();

    let ids = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| scene.get_asset_id(asset_type, "shared").unwrap()))
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
    let asset_type = asset_type(&scene, "FailingAsset").unwrap();

    for _ in 0..2 {
        assert!(matches!(
            scene.get_asset_id(asset_type, "bad"),
            Err(SceneError::AssetError(AssetError::CreationFailure))
        ));
    }

    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
    assert!(matches!(
        scene.resolve_asset_id(asset_type, "bad"),
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
        let asset_type = asset_type(&scene, "TestAsset").unwrap();
        scene.get_asset_id(asset_type, "first").unwrap();
        scene.get_asset_id(asset_type, "second").unwrap();
    }

    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 2);
}

#[test]
fn reset_destroys_cached_assets() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_counts();
    let scene = scene();
    let asset_type = asset_type(&scene, "TestAsset").unwrap();

    scene.get_asset_id(asset_type, "first").unwrap();
    scene.reset().unwrap();

    assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 1);
    assert!(matches!(
        scene.resolve_asset_id(asset_type, "first"),
        Err(SceneError::AssetNotFound)
    ));
    scene.get_asset_id(asset_type, "first").unwrap();
    assert_eq!(CREATE_COUNT.load(Ordering::Relaxed), 2);
}
