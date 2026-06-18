use crate::{Scene, System, Uuid};
use std::{
    ffi::c_void,
    sync::{LazyLock, Mutex},
};

#[repr(C)]
struct MacroCounter {
    value: i64,
}

static MACRO_SYSTEM_ENTITIES: LazyLock<Mutex<Vec<Vec<Uuid>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static MACRO_SYSTEM_GROUPS: LazyLock<Mutex<Vec<usize>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[unsafe(no_mangle)]
unsafe extern "C" fn wxr_create_macro_counter() -> *mut c_void {
    Box::into_raw(Box::new(MacroCounter { value: 0 })) as *mut c_void
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wxr_destroy_macro_counter(data: *mut c_void) {
    unsafe {
        drop(Box::from_raw(data as *mut MacroCounter));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wxr_create_macro_marker() -> *mut c_void {
    Box::into_raw(Box::new(MacroCounter { value: 0 })) as *mut c_void
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wxr_destroy_macro_marker(data: *mut c_void) {
    unsafe {
        drop(Box::from_raw(data as *mut MacroCounter));
    }
}

#[System(entities = [["macro_counter"], ["macro_marker"]])]
pub fn macro_group_counter(_scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    *MACRO_SYSTEM_ENTITIES.lock().unwrap() = entities;
    *MACRO_SYSTEM_GROUPS.lock().unwrap() = groups;
}

#[test]
fn system_macro_registers_and_runs_static_system() {
    MACRO_SYSTEM_ENTITIES.lock().unwrap().clear();
    MACRO_SYSTEM_GROUPS.lock().unwrap().clear();

    let mut scene = Scene::new();
    let counter = scene.add_entity();
    scene
        .add_component(counter, "macro_counter".to_owned())
        .unwrap();

    let marker = scene.add_entity();
    scene
        .add_component(marker, "macro_marker".to_owned())
        .unwrap();

    let both = scene.add_entity();
    scene
        .add_component(both, "macro_counter".to_owned())
        .unwrap();
    scene
        .add_component(both, "macro_marker".to_owned())
        .unwrap();

    scene
        .add_system("macro_group_counter".to_owned(), 1)
        .unwrap();

    scene.tick();

    let groups = MACRO_SYSTEM_GROUPS.lock().unwrap();
    assert_eq!(*groups, vec![2, 1]);

    let entities = MACRO_SYSTEM_ENTITIES.lock().unwrap();
    assert_eq!(entities.len(), 2);
    assert!(entities[0].contains(&counter));
    assert!(entities[0].contains(&both));
    assert_eq!(entities[1], vec![marker]);
}

#[test]
fn scene_has_component_reports_component_presence() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert!(!scene.has_component(entity, "macro_counter"));

    scene
        .add_component(entity, "macro_counter".to_owned())
        .unwrap();

    assert!(scene.has_component(entity, "macro_counter"));
    assert!(!scene.has_component(entity, "macro_marker"));
}
