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
    logging::{LogEntry, LogHandler, LogLevel},
    scene::Scene,
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

#[test]
fn entries_are_c_compatible_and_deep_clone_their_messages() {
    let (sender, receiver): (Sender<(LogLevel, bool, usize, String)>, _) = mpsc::channel();
    let scene = Scene::new();
    unsafe { scene.add_log_handler(handler(inspect_entry, &sender)) };

    scene.info("hello 😀");
    let (level, has_message, length, message) = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("handler did not receive entry");
    assert_eq!(level, LogLevel::Info);
    assert!(has_message);
    assert_eq!(length, "hello 😀".len());
    assert_eq!(message, "hello 😀");

    wait_for_len(&scene, 1);
    let logs = scene.get_logs();
    let entry = logs.get(0).expect("entry missing");
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

    let events: Vec<_> = (0..5)
        .map(|_| {
            receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("handler did not receive entry")
        })
        .collect();
    assert_eq!(
        events,
        vec![
            (LogLevel::Debug, "direct debug".to_owned()),
            (LogLevel::Debug, "debug".to_owned()),
            (LogLevel::Info, "info".to_owned()),
            (LogLevel::Warning, "warning".to_owned()),
            (LogLevel::Error, "error".to_owned()),
        ]
    );
    wait_for_len(&scene, 5);
    let logs = scene.get_logs();
    assert_eq!(
        logs.iter()
            .map(|entry| (entry.level(), entry.getMessage()))
            .collect::<Vec<_>>(),
        events
    );
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

    assert_eq!(
        receiver_a
            .recv_timeout(Duration::from_secs(1))
            .expect("scene A handler did not run"),
        (LogLevel::Info, "a".to_owned())
    );
    assert_eq!(
        receiver_b
            .recv_timeout(Duration::from_secs(1))
            .expect("scene B handler did not run"),
        (LogLevel::Error, "b".to_owned())
    );
    wait_for_len(&scene_a, 1);
    wait_for_len(&scene_b, 1);
    assert_eq!(scene_a.get_logs().get(0).unwrap().getMessage(), "a");
    assert_eq!(scene_b.get_logs().get(0).unwrap().getMessage(), "b");
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

    wait_for_len(&scene, 256);
    assert_eq!(scene.get_logs().len(), 256);
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
    assert_eq!(
        receiver_a
            .recv_timeout(Duration::from_secs(1))
            .expect("first handler did not run"),
        (LogLevel::Warning, "shared".to_owned())
    );
    assert_eq!(
        receiver_b
            .recv_timeout(Duration::from_secs(1))
            .expect("second handler did not run"),
        (LogLevel::Warning, "shared".to_owned())
    );
    wait_for_len(&scene, 1);
    assert!(receiver_a.try_recv().is_err());
    assert!(receiver_b.try_recv().is_err());
}

#[test]
fn a_log_snapshot_synchronizes_ring_inspection() {
    let (sender, receiver) = mpsc::channel::<Event>();
    let scene = Scene::new();
    unsafe { scene.add_log_handler(handler(record_entry, &sender)) };
    scene.info("first");
    wait_for_len(&scene, 1);

    let logs = scene.get_logs();
    scene.info("second");
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("first handler event missing"),
        (LogLevel::Info, "first".to_owned())
    );
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("second handler event missing"),
        (LogLevel::Info, "second".to_owned())
    );
    assert_eq!(logs.len(), 1);
    wait_for_len(&scene, 2);
}

#[test]
fn dropping_a_scene_drains_queued_logging_work() {
    let calls = AtomicUsize::new(0);
    {
        let scene = Scene::new();
        unsafe { scene.add_log_handler(handler(count_entry, &calls)) };
        for _ in 0..32 {
            scene.debug("queued");
        }
    }
    assert_eq!(calls.load(Ordering::SeqCst), 32);
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
    wait_for_len(&scene, 4);
}
