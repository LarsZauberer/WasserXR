use std::{
    ffi::c_void,
    sync::{
        Arc, Barrier, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use wasserxr::{
    definitions::{
        assets::AssetDefinition, components::ComponentDefinition, plugins::PluginDefinition,
        systems::SystemDefinition,
    },
    field::AccessRequest,
    ids::{AssetTypeID, ComponentTypeID, FieldTypeID, PluginID, SystemTypeID, TypeID},
    logging::{LogEntry, LogHandler, LogLevel},
    scene::Scene,
    utils::version::Version,
};

type Event = (LogLevel, String);

unsafe extern "C" fn record_entry(data: *mut c_void, entry: *const LogEntry) {
    let sender = unsafe { &*(data as *const Sender<Event>) };
    let entry = unsafe { &*entry };
    let _ = sender.send((entry.level(), entry.getMessage()));
}

unsafe extern "C" fn inspect_entry(data: *mut c_void, entry: *const LogEntry) {
    let sender = unsafe { &*(data as *const Sender<(LogLevel, bool, usize, String)>) };
    let entry = unsafe { &*entry };
    let message =
        String::from_utf8(entry.message_bytes().to_vec()).expect("entry message is UTF-8");
    let _ = sender.send((
        entry.level(),
        !entry.message_bytes().is_empty(),
        entry.message_bytes().len(),
        message,
    ));
}

fn handler<T>(callback: wasserxr::logging::LogCallback, state: &T) -> LogHandler {
    LogHandler {
        callback,
        data: state as *const T as *mut c_void,
    }
}

fn wait_for_len(scene: &Scene, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if scene.get_logs().len() >= expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "logger did not process {expected} entries"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn wait_for_latest(scene: &Scene, message: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if scene
            .get_logs()
            .iter()
            .last()
            .is_some_and(|entry| entry.getMessage() == message)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "logger did not process entry {message}"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn recv_until(receiver: &mpsc::Receiver<Event>, expected: Event) -> Event {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let event = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("handler did not receive expected entry");
        if event == expected {
            return event;
        }
    }
}

fn events_after(scene: &Scene, index: usize) -> Vec<Event> {
    scene
        .get_logs()
        .iter()
        .skip(index)
        .map(|entry| (entry.level(), entry.getMessage()))
        .collect()
}

unsafe extern "C" fn create_component() -> *mut c_void {
    std::ptr::dangling_mut()
}

unsafe extern "C" fn destroy_component(_: *mut c_void) {}

unsafe extern "C" fn create_asset() -> *mut c_void {
    std::ptr::dangling_mut()
}

unsafe extern "C" fn fail_to_create_asset() -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn destroy_asset(_: *mut c_void) {}

unsafe extern "C" fn run_system(_: *const Scene, _: *const TypeID, _: usize) {}

const COMPONENT: ComponentDefinition = ComponentDefinition {
    name: c"TestComponent".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
    methods: std::ptr::null(),
    method_count: 0,
};

const ASSETS: [AssetDefinition; 2] = [
    AssetDefinition {
        name: c"TestAsset".as_ptr(),
        creator: Some(create_asset),
        destroyer: Some(destroy_asset),
        fields: std::ptr::null(),
        field_count: 0,
    },
    AssetDefinition {
        name: c"FailingAsset".as_ptr(),
        creator: Some(fail_to_create_asset),
        destroyer: Some(destroy_asset),
        fields: std::ptr::null(),
        field_count: 0,
    },
];

const SYSTEM: SystemDefinition = SystemDefinition {
    name: c"TestSystem".as_ptr(),
    attacher: None,
    runner: Some(run_system),
    detacher: None,
    requires: std::ptr::null(),
    requires_count: 0,
    wanted_by: std::ptr::null(),
    wanted_by_count: 0,
    type_id_requests: std::ptr::null(),
    type_id_request_count: 0,
};

const PLUGIN: PluginDefinition = PluginDefinition {
    name: c"LoggingPlugin".as_ptr(),
    engine_version: Version {
        major: 0,
        minor: 2,
        patch: 0,
    },
    components: &COMPONENT,
    component_count: 1,
    assets: ASSETS.as_ptr(),
    asset_count: ASSETS.len(),
    systems: &SYSTEM,
    system_count: 1,
    functions: std::ptr::null(),
    function_count: 0,
};

fn load_test_plugin(
    scene: &Scene,
) -> (
    PluginID,
    ComponentTypeID,
    AssetTypeID,
    AssetTypeID,
    SystemTypeID,
) {
    let plugin = unsafe { scene.load_static_plugin(PLUGIN) }.unwrap();
    (
        plugin,
        scene
            .resolve_component_type_id(plugin, "TestComponent")
            .unwrap(),
        scene.resolve_asset_type_id(plugin, "TestAsset").unwrap(),
        scene.resolve_asset_type_id(plugin, "FailingAsset").unwrap(),
        scene.resolve_system_type_id(plugin, "TestSystem").unwrap(),
    )
}

#[test]
fn entries_are_c_compatible_and_deep_clone_their_messages() {
    let (sender, receiver): (Sender<(LogLevel, bool, usize, String)>, _) = mpsc::channel();
    let scene = Scene::new();
    unsafe { scene.add_log_handler(handler(inspect_entry, &sender)) };

    scene.info("hello 😀");
    let (level, has_message, length, message) = loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("handler did not receive entry");
        if event.3 == "hello 😀" {
            break event;
        }
    };
    assert_eq!(level, LogLevel::Info);
    assert!(has_message);
    assert_eq!(length, "hello 😀".len());
    assert_eq!(message, "hello 😀");

    wait_for_latest(&scene, "hello 😀");
    let logs = scene.get_logs();
    let entry = logs
        .iter()
        .find(|entry| entry.getMessage() == "hello 😀")
        .expect("entry missing");
    let clone = entry.clone();
    assert_eq!(clone.level(), entry.level());
    assert_eq!(clone.getMessage(), entry.getMessage());
    assert_ne!(
        clone.message_bytes().as_ptr(),
        entry.message_bytes().as_ptr()
    );
}

#[test]
fn scene_methods_submit_each_log_level_in_order() {
    let (sender, receiver) = mpsc::channel::<Event>();
    let scene = Scene::new();
    unsafe { scene.add_log_handler(handler(record_entry, &sender)) };

    scene.log(LogLevel::Debug, "direct debug");
    scene.debug("debug");
    scene.info("info");
    scene.warning("warning");
    scene.error("error");

    let expected = vec![
        (LogLevel::Debug, "direct debug".to_owned()),
        (LogLevel::Debug, "debug".to_owned()),
        (LogLevel::Info, "info".to_owned()),
        (LogLevel::Warning, "warning".to_owned()),
        (LogLevel::Error, "error".to_owned()),
    ];
    let events: Vec<_> = expected
        .iter()
        .cloned()
        .map(|event| recv_until(&receiver, event))
        .collect();
    wait_for_latest(&scene, "error");
    let logs: Vec<_> = scene
        .get_logs()
        .iter()
        .map(|entry| (entry.level(), entry.getMessage()))
        .filter(|event| expected.contains(event))
        .collect();
    assert_eq!(events, expected);
    assert_eq!(logs, expected);
}

#[test]
fn scenes_have_isolated_logs_and_handlers() {
    let (sender_a, receiver_a) = mpsc::channel::<Event>();
    let (sender_b, receiver_b) = mpsc::channel::<Event>();
    let scene_a = Scene::new();
    let scene_b = Scene::new();
    unsafe {
        scene_a.add_log_handler(handler(record_entry, &sender_a));
        scene_b.add_log_handler(handler(record_entry, &sender_b));
    }

    scene_a.info("a");
    scene_b.error("b");

    recv_until(&receiver_a, (LogLevel::Info, "a".to_owned()));
    recv_until(&receiver_b, (LogLevel::Error, "b".to_owned()));
    wait_for_latest(&scene_a, "a");
    wait_for_latest(&scene_b, "b");
    assert!(
        scene_a
            .get_logs()
            .iter()
            .any(|entry| entry.getMessage() == "a")
    );
    assert!(
        scene_b
            .get_logs()
            .iter()
            .any(|entry| entry.getMessage() == "b")
    );
}

#[test]
fn concurrent_submission_is_processed_without_serializing_callers() {
    let scene = Arc::new(Scene::new());
    let threads: Vec<_> = (0..8)
        .map(|thread_id| {
            let scene = Arc::clone(&scene);
            thread::spawn(move || {
                for entry_id in 0..32 {
                    scene.info(format!("{thread_id}:{entry_id}"));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().expect("log producer panicked");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let count = scene
            .get_logs()
            .iter()
            .filter(|entry| entry.getMessage().contains(':'))
            .count();
        if count == 256 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "logger did not process all concurrent entries"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

struct SlowState {
    calls: AtomicUsize,
    entered: Barrier,
    release: Barrier,
}

unsafe extern "C" fn block_first(data: *mut c_void, _: *const LogEntry) {
    let state = unsafe { &*(data as *const SlowState) };
    if state.calls.fetch_add(1, Ordering::SeqCst) == 0 {
        state.entered.wait();
        state.release.wait();
    }
}

#[test]
fn submission_does_not_wait_for_a_slow_handler() {
    let state = SlowState {
        calls: AtomicUsize::new(0),
        entered: Barrier::new(2),
        release: Barrier::new(2),
    };
    let scene = Arc::new(Scene::new());
    unsafe { scene.add_log_handler(handler(block_first, &state)) };
    scene.info("first");
    state.entered.wait();

    let (sender, receiver) = mpsc::channel::<()>();
    let start = Instant::now();
    let producer = thread::spawn({
        let scene = Arc::clone(&scene);
        move || {
            scene.info("second");
            sender.send(()).expect("submission observer dropped");
        }
    });
    receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("submission waited for the slow callback");
    assert!(start.elapsed() < Duration::from_millis(500));

    state.release.wait();
    producer.join().expect("producer panicked");
    wait_for_len(&scene, 2);
}

#[test]
fn retention_is_ordered_and_evicts_only_the_oldest_entry() {
    let scene = Scene::new();
    for index in 0..1025 {
        scene.info(index.to_string());
    }
    wait_for_latest(&scene, "1024");

    let logs = scene.get_logs();
    assert_eq!(logs.len(), 1024);
    assert_eq!(logs.get(0).unwrap().getMessage(), "1");
    assert_eq!(logs.get(1023).unwrap().getMessage(), "1024");
    assert!(logs.iter().zip(logs.iter().skip(1)).all(|(old, new)| {
        old.getMessage().parse::<usize>().unwrap() + 1 == new.getMessage().parse().unwrap()
    }));
}

#[test]
fn multiple_stateful_handlers_receive_the_same_entry_once() {
    let (sender_a, receiver_a) = mpsc::channel::<Event>();
    let (sender_b, receiver_b) = mpsc::channel::<Event>();
    let scene = Scene::new();
    unsafe {
        scene.add_log_handler(handler(record_entry, &sender_a));
        scene.add_log_handler(handler(record_entry, &sender_b));
    }

    scene.warning("shared");
    recv_until(&receiver_a, (LogLevel::Warning, "shared".to_owned()));
    recv_until(&receiver_b, (LogLevel::Warning, "shared".to_owned()));
    wait_for_latest(&scene, "shared");
    assert!(!receiver_a.try_iter().any(|event| event.1 == "shared"));
    assert!(!receiver_b.try_iter().any(|event| event.1 == "shared"));
    assert_eq!(
        scene
            .get_logs()
            .iter()
            .filter(|entry| entry.getMessage() == "shared")
            .count(),
        1
    );
}

#[test]
fn a_log_snapshot_synchronizes_ring_inspection() {
    let (sender, receiver) = mpsc::channel::<Event>();
    let scene = Scene::new();
    unsafe { scene.add_log_handler(handler(record_entry, &sender)) };
    scene.info("first");
    wait_for_latest(&scene, "first");

    let logs = scene.get_logs();
    scene.info("second");
    recv_until(&receiver, (LogLevel::Info, "first".to_owned()));
    recv_until(&receiver, (LogLevel::Info, "second".to_owned()));
    assert!(logs.iter().any(|entry| entry.getMessage() == "first"));
    assert!(!logs.iter().any(|entry| entry.getMessage() == "second"));
    wait_for_latest(&scene, "second");
}

#[test]
fn dropping_a_scene_drains_queued_logging_work() {
    let calls = AtomicUsize::new(0);
    {
        let scene = Scene::new();
        wait_for_latest(&scene, "Created a new scene");
        unsafe { scene.add_log_handler(handler(count_entry, &calls)) };
        for _ in 0..32 {
            scene.debug("queued");
        }
    }
    assert_eq!(calls.load(Ordering::SeqCst), 33);
}

unsafe extern "C" fn count_entry(data: *mut c_void, _: *const LogEntry) {
    let calls = unsafe { &*(data as *const AtomicUsize) };
    calls.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn scene_remains_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Scene>();

    let state = Arc::new(Mutex::new(0));
    let scene = Arc::new(Scene::new());
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let scene = Arc::clone(&scene);
            let state = Arc::clone(&state);
            thread::spawn(move || {
                scene.debug("shared");
                *state.lock().unwrap() += 1;
            })
        })
        .collect();
    for thread in threads {
        thread.join().expect("shared scene thread panicked");
    }
    assert_eq!(*state.lock().unwrap(), 4);
    wait_for_latest(&scene, "shared");
    assert_eq!(
        scene
            .get_logs()
            .iter()
            .filter(|entry| entry.getMessage() == "shared")
            .count(),
        4
    );
}

#[test]
fn instrumented_scene_operations_report_success() {
    let mut scene = Scene::new();
    let (plugin, component_type, asset_type, _, system_type) = load_test_plugin(&scene);
    let entity = scene.add_entity();
    let component = scene.add_component(entity, component_type).unwrap();
    scene
        .with_component_fields(&[(component_type, AccessRequest::Read, &[])], |_| {})
        .unwrap();
    scene
        .with_singleton_fields((component_type, AccessRequest::Read, &[]), |_| {})
        .unwrap();
    let asset = scene.get_asset_id(asset_type, "bounded-data").unwrap();
    scene
        .with_asset_fields(&[(asset_type, "bounded-data")], |_| {})
        .unwrap();
    let system = scene.add_system(system_type).unwrap();
    scene.tick();
    scene.remove_system(system).unwrap();
    scene.remove_component(component).unwrap();
    scene.remove_entity(entity).unwrap();
    scene.reset().unwrap();
    wait_for_latest(&scene, "Reset the scene");

    let messages: Vec<_> = scene.get_logs().iter().map(LogEntry::getMessage).collect();
    for expected in [
        "Created a new scene".to_owned(),
        format!("Loaded statically linked plugin {plugin:?}"),
        format!("Added entity {entity:?}"),
        format!("Added component {component:?} to entity {entity:?}"),
        "Ran component query (requirements: 1)".to_owned(),
        format!("Ensured entity {entity:?} has singleton component type {component_type:?}"),
        format!("Queried singleton component type {component_type:?} (fields: 0)"),
        format!("Loaded or reused asset {asset:?}"),
        "Queried assets: 1".to_owned(),
        format!("Added system {system:?}"),
        "Ran systems: 1".to_owned(),
        format!("Removed system {system:?}"),
        format!("Removed component {component:?}"),
        format!("Removed entity {entity:?}"),
        "Removed systems: 0".to_owned(),
        "Removed entities: 0".to_owned(),
        "Removed cached assets: 1".to_owned(),
        "Reset the scene".to_owned(),
    ] {
        assert!(
            messages.contains(&expected),
            "missing log entry: {expected}"
        );
    }
    assert!(
        !messages
            .iter()
            .any(|message| message.contains("bounded-data"))
    );
}

#[test]
fn instrumented_failures_report_warnings_without_success_entries() {
    let scene = Scene::new();
    let (_, component_type, _, failing_asset_type, system_type) = load_test_plugin(&scene);
    let entity = scene.add_entity();
    let component = scene.add_component(entity, component_type).unwrap();
    scene.remove_entity(entity).unwrap();
    let query_entity = scene.add_entity();
    scene.add_component(query_entity, component_type).unwrap();
    let system = scene.add_system(system_type).unwrap();
    wait_for_latest(&scene, &format!("Added system {system:?}"));
    let baseline = scene.get_logs().len();

    let plugin_error = unsafe { scene.load_static_plugin(PLUGIN) }.unwrap_err();
    let entity_error = scene.remove_entity(entity).unwrap_err();
    let component_error = scene.remove_component(component).unwrap_err();
    let duplicate_system_error = scene.add_system(system_type).unwrap_err();
    let asset_error = scene
        .get_asset_id(failing_asset_type, "secret-input")
        .unwrap_err();
    let query_error = scene
        .with_asset_fields(&[(failing_asset_type, "secret-input")], |_| {})
        .unwrap_err();

    let TypeID::ComponentTypeID(plugin, component_slot) = TypeID::from(component_type) else {
        unreachable!()
    };
    let missing_field =
        FieldTypeID::try_from(TypeID::FieldTypeID(plugin, component_slot, u64::MAX)).unwrap();
    let query = [(component_type, AccessRequest::Read, &[missing_field][..])];
    let component_query_error = scene.with_component_fields(&query, |_| {}).unwrap_err();
    let final_message =
        format!("Could not run component query (requirements: 1): {component_query_error}");
    wait_for_latest(&scene, &final_message);

    let events = events_after(&scene, baseline);
    assert!(events.iter().all(|(level, _)| *level == LogLevel::Warning));
    for expected in [
        format!("Could not load statically linked plugin: {plugin_error}"),
        format!("Could not remove entity {entity:?}: {entity_error}"),
        format!("Could not remove component {component:?}: {component_error}"),
        format!("Could not add system type {system_type:?}: {duplicate_system_error}"),
        format!("Could not load asset type {failing_asset_type:?}: {asset_error}"),
        format!("Could not query assets: {query_error}"),
        final_message,
    ] {
        assert!(events.iter().any(|(_, message)| message == &expected));
    }
    assert!(
        !events
            .iter()
            .any(|(_, message)| message.contains("secret-input"))
    );
}

#[test]
fn getters_resolvers_and_logging_apis_add_no_automatic_entries() {
    let scene = Scene::new();
    let (_, component_type, asset_type, _, system_type) = load_test_plugin(&scene);
    let plugin = scene.resolve_plugin_id("LoggingPlugin").unwrap();
    let entity = scene.add_entity();
    let component = scene.add_component(entity, component_type).unwrap();
    wait_for_latest(
        &scene,
        &format!("Added component {component:?} to entity {entity:?}"),
    );
    let baseline = scene.get_logs().len();

    assert_eq!(scene.get_plugin_name(plugin), "LoggingPlugin");
    assert_eq!(scene.resolve_plugin_id("LoggingPlugin"), Some(plugin));
    assert_eq!(
        scene
            .resolve_component_type_id(plugin, "TestComponent")
            .unwrap(),
        component_type
    );
    assert_eq!(
        scene.resolve_asset_type_id(plugin, "TestAsset").unwrap(),
        asset_type
    );
    assert_eq!(
        scene.resolve_system_type_id(plugin, "TestSystem").unwrap(),
        system_type
    );
    assert_eq!(scene.get_entities(), vec![entity]);
    assert_eq!(scene.get_components(entity).unwrap().len(), 1);
    assert!(scene.get_system_id(system_type).is_err());
    scene.debug("caller debug");
    scene.info("caller info");
    scene.warning("caller warning");
    scene.error("caller error");
    wait_for_latest(&scene, "caller error");

    assert_eq!(
        events_after(&scene, baseline),
        vec![
            (LogLevel::Debug, "caller debug".to_owned()),
            (LogLevel::Info, "caller info".to_owned()),
            (LogLevel::Warning, "caller warning".to_owned()),
            (LogLevel::Error, "caller error".to_owned()),
        ]
    );
}

#[test]
fn reset_preserves_each_instrumented_boundary() {
    let scene = Scene::new();
    let entity = scene.add_entity();
    wait_for_latest(&scene, &format!("Added entity {entity:?}"));
    let baseline = scene.get_logs().len();

    scene.reset().unwrap();
    wait_for_latest(&scene, "Reset the scene");

    assert_eq!(
        events_after(&scene, baseline),
        vec![
            (LogLevel::Debug, "Removed systems: 0".to_owned()),
            (LogLevel::Debug, "Removed entities: 1".to_owned()),
            (LogLevel::Debug, "Removed cached assets: 0".to_owned()),
            (LogLevel::Debug, "Reset the scene".to_owned()),
        ]
    );
}

#[test]
fn newly_registered_handler_can_observe_an_older_queued_entry() {
    let slow_state = SlowState {
        calls: AtomicUsize::new(0),
        entered: Barrier::new(2),
        release: Barrier::new(2),
    };
    let (sender, receiver) = mpsc::channel::<Event>();
    let scene = Scene::new();
    wait_for_latest(&scene, "Created a new scene");
    unsafe { scene.add_log_handler(handler(block_first, &slow_state)) };
    slow_state.entered.wait();

    scene.info("queued before registration");
    unsafe { scene.add_log_handler(handler(record_entry, &sender)) };
    slow_state.release.wait();

    assert_eq!(
        recv_until(
            &receiver,
            (LogLevel::Info, "queued before registration".to_owned())
        ),
        (LogLevel::Info, "queued before registration".to_owned())
    );
    assert_eq!(
        recv_until(
            &receiver,
            (LogLevel::Debug, "Registered a log handler".to_owned())
        ),
        (LogLevel::Debug, "Registered a log handler".to_owned())
    );
}
