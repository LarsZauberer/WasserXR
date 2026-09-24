use std::{
    ffi::{CStr, c_char},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use wasserxr::{
    definitions::{plugins::PluginDefinition, systems::SystemDefinition},
    ids::{PluginID, SystemID, SystemTypeID, TypeID},
    scene::Scene,
    utils::version::Version,
};

static TICK_ORDER: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static ADD_ON_TICK: Mutex<Option<SystemTypeID>> = Mutex::new(None);
static REMOVE_ON_TICK: Mutex<Option<SystemID>> = Mutex::new(None);
static ADDED_RUNS: AtomicUsize = AtomicUsize::new(0);
static REMOVED_RUNS: AtomicUsize = AtomicUsize::new(0);
static SLOW_FINISHED: AtomicBool = AtomicBool::new(false);
static DEPENDENT_RAN_BEFORE_SLOW_FINISHED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn run(_: *const Scene, _: *const TypeID, _: usize) {}

unsafe extern "C" fn run_a(_: *const Scene, _: *const TypeID, _: usize) {
    TICK_ORDER.lock().unwrap().push("a");
}

unsafe extern "C" fn run_b(_: *const Scene, _: *const TypeID, _: usize) {
    TICK_ORDER.lock().unwrap().push("b");
}

unsafe extern "C" fn run_c(_: *const Scene, _: *const TypeID, _: usize) {
    TICK_ORDER.lock().unwrap().push("c");
}

unsafe extern "C" fn run_d(_: *const Scene, _: *const TypeID, _: usize) {
    TICK_ORDER.lock().unwrap().push("d");
}

unsafe extern "C" fn run_e(_: *const Scene, _: *const TypeID, _: usize) {
    TICK_ORDER.lock().unwrap().push("e");
}

unsafe extern "C" fn add_on_tick(scene: *const Scene, _: *const TypeID, _: usize) {
    if let Some(system_type) = ADD_ON_TICK.lock().unwrap().take() {
        unsafe { &*scene }.add_system(system_type).unwrap();
    }
}

unsafe extern "C" fn run_added(_: *const Scene, _: *const TypeID, _: usize) {
    ADDED_RUNS.fetch_add(1, Ordering::Relaxed);
}

unsafe extern "C" fn remove_on_tick(scene: *const Scene, _: *const TypeID, _: usize) {
    if let Some(system) = REMOVE_ON_TICK.lock().unwrap().take() {
        unsafe { &*scene }.remove_system(system).unwrap();
    }
}

unsafe extern "C" fn run_removed(_: *const Scene, _: *const TypeID, _: usize) {
    REMOVED_RUNS.fetch_add(1, Ordering::Relaxed);
}

unsafe extern "C" fn run_slow(_: *const Scene, _: *const TypeID, _: usize) {
    thread::sleep(Duration::from_millis(100));
    SLOW_FINISHED.store(true, Ordering::Release);
}

unsafe extern "C" fn run_after_fast(_: *const Scene, _: *const TypeID, _: usize) {
    DEPENDENT_RAN_BEFORE_SLOW_FINISHED
        .store(!SLOW_FINISHED.load(Ordering::Acquire), Ordering::Relaxed);
}

fn system_definition(
    name: &CStr,
    requires: &[*const c_char],
    wanted_by: &[*const c_char],
) -> SystemDefinition {
    SystemDefinition {
        name: name.as_ptr(),
        attacher: None,
        runner: Some(run),
        detacher: None,
        requires: requires.as_ptr(),
        requires_count: requires.len(),
        wanted_by: wanted_by.as_ptr(),
        wanted_by_count: wanted_by.len(),
        type_id_requests: std::ptr::null(),
        type_id_request_count: 0,
    }
}

fn load_systems(scene: &Scene, systems: &[SystemDefinition]) -> PluginID {
    let plugin = PluginDefinition {
        name: c"TickPlugin".as_ptr(),
        engine_version: Version {
            major: 0,
            minor: 2,
            patch: 0,
        },
        components: std::ptr::null(),
        component_count: 0,
        assets: std::ptr::null(),
        asset_count: 0,
        systems: systems.as_ptr(),
        system_count: systems.len(),
        functions: std::ptr::null(),
        function_count: 0,
    };
    unsafe { scene.load_static_plugin(plugin) }.unwrap()
}

#[test]
fn tick_honors_requires_and_wanted_by() {
    TICK_ORDER.lock().unwrap().clear();
    let requires_a = [c"a".as_ptr()];
    let requires_a_and_c = [c"a".as_ptr(), c"c".as_ptr()];
    let wanted_by_d = [c"d".as_ptr()];
    let mut systems = [
        system_definition(c"a", &[], &[]),
        system_definition(c"b", &requires_a, &[]),
        system_definition(c"c", &[], &wanted_by_d),
        system_definition(c"d", &[], &[]),
        system_definition(c"e", &requires_a_and_c, &[]),
    ];
    systems[0].runner = Some(run_a);
    systems[1].runner = Some(run_b);
    systems[2].runner = Some(run_c);
    systems[3].runner = Some(run_d);
    systems[4].runner = Some(run_e);

    let mut scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    for name in ["d", "c", "a", "b", "e"] {
        let system_type = scene.resolve_system_type_id(plugin, name).unwrap();
        scene.add_system(system_type).unwrap();
    }

    scene.tick();

    let order = TICK_ORDER.lock().unwrap();
    let position = |name| order.iter().position(|entry| *entry == name).unwrap();
    assert!(position("a") < position("b"));
    assert!(position("c") < position("d"));
    assert!(position("a") < position("e"));
    assert!(position("c") < position("e"));
}

#[test]
fn systems_added_during_tick_run_on_the_next_tick() {
    ADDED_RUNS.store(0, Ordering::Relaxed);
    let mut systems = [
        system_definition(c"adder", &[], &[]),
        system_definition(c"added", &[], &[]),
    ];
    systems[0].runner = Some(add_on_tick);
    systems[1].runner = Some(run_added);

    let mut scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    let adder = scene.resolve_system_type_id(plugin, "adder").unwrap();
    let added = scene.resolve_system_type_id(plugin, "added").unwrap();
    scene.add_system(adder).unwrap();
    *ADD_ON_TICK.lock().unwrap() = Some(added);

    scene.tick();
    assert_eq!(ADDED_RUNS.load(Ordering::Relaxed), 0);

    scene.tick();
    assert_eq!(ADDED_RUNS.load(Ordering::Relaxed), 1);
}

#[test]
fn systems_removed_during_tick_finish_the_current_tick() {
    REMOVED_RUNS.store(0, Ordering::Relaxed);
    let wanted_by_removed = [c"removed".as_ptr()];
    let mut systems = [
        system_definition(c"remover", &[], &wanted_by_removed),
        system_definition(c"removed", &[], &[]),
    ];
    systems[0].runner = Some(remove_on_tick);
    systems[1].runner = Some(run_removed);

    let mut scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    let remover = scene.resolve_system_type_id(plugin, "remover").unwrap();
    let removed = scene.resolve_system_type_id(plugin, "removed").unwrap();
    scene.add_system(remover).unwrap();
    let removed = scene.add_system(removed).unwrap();
    *REMOVE_ON_TICK.lock().unwrap() = Some(removed);

    scene.tick();
    assert_eq!(REMOVED_RUNS.load(Ordering::Relaxed), 1);

    scene.tick();
    assert_eq!(REMOVED_RUNS.load(Ordering::Relaxed), 1);
}

#[test]
fn tick_runs_ready_systems_without_waiting_for_unrelated_systems() {
    if thread::available_parallelism().map_or(1, usize::from) < 2 {
        return;
    }

    SLOW_FINISHED.store(false, Ordering::Relaxed);
    DEPENDENT_RAN_BEFORE_SLOW_FINISHED.store(false, Ordering::Relaxed);
    let requires_fast = [c"fast".as_ptr()];
    let mut systems = [
        system_definition(c"slow", &[], &[]),
        system_definition(c"fast", &[], &[]),
        system_definition(c"after_fast", &requires_fast, &[]),
    ];
    systems[0].runner = Some(run_slow);
    systems[2].runner = Some(run_after_fast);

    let mut scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    for name in ["slow", "fast", "after_fast"] {
        let system_type = scene.resolve_system_type_id(plugin, name).unwrap();
        scene.add_system(system_type).unwrap();
    }

    scene.tick();

    assert!(DEPENDENT_RAN_BEFORE_SLOW_FINISHED.load(Ordering::Relaxed));
    assert!(SLOW_FINISHED.load(Ordering::Acquire));
}
