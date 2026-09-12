use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use wasserxr::{
    definitions::{
        components::ComponentDefinition, plugins::PluginDefinition, systems::SystemDefinition,
        type_id_requests::TypeIDRequests,
    },
    errors::SceneError,
    ids::TypeID,
    scene::Scene,
    utils::version::Version,
};

static ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
static DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
static RECEIVED_IDS: Mutex<Vec<TypeID>> = Mutex::new(Vec::new());

unsafe extern "C" fn create_component() -> *mut std::ffi::c_void {
    std::ptr::dangling_mut()
}

unsafe extern "C" fn destroy_component(_: *mut std::ffi::c_void) {}

unsafe extern "C" fn attach(_: *const Scene, ids: *const TypeID, count: usize) {
    ATTACH_COUNT.fetch_add(1, Ordering::Relaxed);
    *RECEIVED_IDS.lock().unwrap() = unsafe { std::slice::from_raw_parts(ids, count) }.to_vec();
}

unsafe extern "C" fn run(_: *const Scene, _: *const TypeID, _: usize) {}

unsafe extern "C" fn detach(_: *const Scene, _: *const TypeID, _: usize) {
    DETACH_COUNT.fetch_add(1, Ordering::Relaxed);
}

const COMPONENT: ComponentDefinition = ComponentDefinition {
    name: c"Transform".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
};

const REQUEST: TypeIDRequests = TypeIDRequests::ComponentTypeID {
    component: c"Transform".as_ptr(),
};

const SYSTEM: SystemDefinition = SystemDefinition {
    name: c"render".as_ptr(),
    attacher: Some(attach),
    runner: Some(run),
    detacher: Some(detach),
    requires: std::ptr::null(),
    requires_count: 0,
    wanted_by: std::ptr::null(),
    wanted_by_count: 0,
    type_id_requests: &REQUEST,
    type_id_request_count: 1,
};

const PLUGIN: PluginDefinition = PluginDefinition {
    name: c"SystemPlugin".as_ptr(),
    engine_version: Version {
        major: 0,
        minor: 2,
        patch: 0,
    },
    components: &COMPONENT,
    component_count: 1,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: &SYSTEM,
    system_count: 1,
};

#[test]
fn concrete_system_lifecycle_uses_resolved_type_ids() {
    ATTACH_COUNT.store(0, Ordering::Relaxed);
    DETACH_COUNT.store(0, Ordering::Relaxed);
    RECEIVED_IDS.lock().unwrap().clear();

    let scene = Scene::new();
    let plugin = unsafe { scene.load_static_plugin(PLUGIN) }.unwrap();
    let system_type = scene.resolve_system_type_id(plugin, "render").unwrap();
    let component_type = scene
        .resolve_component_type_id(plugin, "Transform")
        .unwrap();

    assert!(matches!(
        scene.resolve_system_id(plugin, system_type),
        Err(SceneError::SystemNotFound)
    ));
    let system = scene.get_system_id(plugin, system_type).unwrap();
    assert_eq!(scene.get_system_id(plugin, system_type).unwrap(), system);
    assert_eq!(
        scene
            .resolve_requested_type_ids(plugin, system_type)
            .unwrap(),
        &[component_type.into()]
    );
    assert_eq!(
        RECEIVED_IDS.lock().unwrap().as_slice(),
        &[component_type.into()]
    );
    assert_eq!(ATTACH_COUNT.load(Ordering::Relaxed), 1);

    scene.remove_system(system).unwrap();
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 1);

    scene.get_system_id(plugin, system_type).unwrap();
    scene.reset().unwrap();
    assert_eq!(ATTACH_COUNT.load(Ordering::Relaxed), 2);
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 2);

    scene.get_system_id(plugin, system_type).unwrap();
    drop(scene);
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 3);
}
