use std::{
    ffi::{CStr, c_char},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use wasserxr::{
    definitions::{
        components::ComponentDefinition, plugins::PluginDefinition, systems::SystemDefinition,
        type_id_requests::TypeIDRequests,
    },
    errors::{SceneError, SystemError},
    ids::{SystemTypeID, TypeID},
    scene::Scene,
    utils::version::Version,
};

static ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
static DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
static RECEIVED_IDS: Mutex<Vec<TypeID>> = Mutex::new(Vec::new());
static ATTACH_SYSTEM_TYPE_ID: Mutex<Option<SystemTypeID>> = Mutex::new(None);

unsafe extern "C" fn create_component() -> *mut std::ffi::c_void {
    std::ptr::dangling_mut()
}

unsafe extern "C" fn destroy_component(_: *mut std::ffi::c_void) {}

unsafe extern "C" fn attach(_: *const Scene, ids: *const TypeID, count: usize) {
    ATTACH_COUNT.fetch_add(1, Ordering::Relaxed);
    *RECEIVED_IDS.lock().unwrap() = unsafe { std::slice::from_raw_parts(ids, count) }.to_vec();
}

unsafe extern "C" fn attach_and_query_system(scene: *const Scene, _: *const TypeID, _: usize) {
    let system_type_id = ATTACH_SYSTEM_TYPE_ID
        .lock()
        .unwrap()
        .expect("system type ID should be set before attaching");
    assert!(unsafe { &*scene }.get_system_id(system_type_id).is_ok());
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
    methods: std::ptr::null(),
    method_count: 0,
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
    functions: std::ptr::null(),
    function_count: 0,
};

fn system_definition(
    name: &CStr,
    requires: &[*const c_char],
    wanted_by: &[*const c_char],
    type_id_requests: &[TypeIDRequests],
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
        type_id_requests: type_id_requests.as_ptr(),
        type_id_request_count: type_id_requests.len(),
    }
}

fn load_systems(scene: &Scene, systems: &[SystemDefinition]) -> wasserxr::ids::PluginID {
    let plugin = PluginDefinition {
        name: c"ValidationPlugin".as_ptr(),
        engine_version: PLUGIN.engine_version,
        components: &COMPONENT,
        component_count: 1,
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
        scene.get_system_id(system_type),
        Err(SceneError::SystemNotFound)
    ));
    let system = scene.add_system(system_type).unwrap();
    assert_eq!(scene.get_system_id(system_type).unwrap(), system);
    assert!(matches!(
        scene.add_system(system_type),
        Err(SceneError::SystemError(SystemError::AlreadyExists))
    ));
    assert_eq!(
        scene.resolve_requested_type_ids(system_type).unwrap(),
        &[component_type.into()]
    );
    assert_eq!(
        RECEIVED_IDS.lock().unwrap().as_slice(),
        &[component_type.into()]
    );
    assert_eq!(ATTACH_COUNT.load(Ordering::Relaxed), 1);

    scene.remove_system(system).unwrap();
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 1);

    scene.add_system(system_type).unwrap();
    scene.reset().unwrap();
    assert_eq!(ATTACH_COUNT.load(Ordering::Relaxed), 2);
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 2);

    scene.add_system(system_type).unwrap();
    drop(scene);
    assert_eq!(DETACH_COUNT.load(Ordering::Relaxed), 3);
}

#[test]
fn attacher_can_use_scene_system_api() {
    let system = SystemDefinition {
        name: c"querying".as_ptr(),
        attacher: Some(attach_and_query_system),
        runner: Some(run),
        detacher: None,
        requires: std::ptr::null(),
        requires_count: 0,
        wanted_by: std::ptr::null(),
        wanted_by_count: 0,
        type_id_requests: std::ptr::null(),
        type_id_request_count: 0,
    };
    let scene = Scene::new();
    let plugin = load_systems(&scene, &[system]);
    let system_type_id = scene.resolve_system_type_id(plugin, "querying").unwrap();
    *ATTACH_SYSTEM_TYPE_ID.lock().unwrap() = Some(system_type_id);

    scene.add_system(system_type_id).unwrap();

    *ATTACH_SYSTEM_TYPE_ID.lock().unwrap() = None;
}

#[test]
fn required_system_must_already_be_in_the_scene() {
    let requires_base = [c"base".as_ptr()];
    let systems = [
        system_definition(c"base", &[], &[], &[]),
        system_definition(c"dependent", &requires_base, &[], &[]),
    ];
    let scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    let base_type = scene.resolve_system_type_id(plugin, "base").unwrap();
    let dependent_type = scene.resolve_system_type_id(plugin, "dependent").unwrap();

    assert!(matches!(
        scene.add_system(dependent_type),
        Err(SceneError::SystemError(SystemError::DependencyNotFound(name)))
            if name == "base"
    ));
    let base = scene.add_system(base_type).unwrap();
    let dependent = scene.add_system(dependent_type).unwrap();

    assert!(matches!(
        scene.remove_system(base),
        Err(SceneError::SystemError(SystemError::DependencyInUse))
    ));
    assert_eq!(scene.get_system_id(base_type).unwrap(), base);

    scene.remove_system(dependent).unwrap();
    scene.remove_system(base).unwrap();
}

#[test]
fn cyclic_system_dependencies_are_rejected() {
    let wanted_by_b = [c"b".as_ptr()];
    let wanted_by_a = [c"a".as_ptr()];
    let systems = [
        system_definition(c"a", &[], &wanted_by_b, &[]),
        system_definition(c"b", &[], &wanted_by_a, &[]),
    ];
    let scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    let a = scene.resolve_system_type_id(plugin, "a").unwrap();
    let b = scene.resolve_system_type_id(plugin, "b").unwrap();

    scene.add_system(a).unwrap();
    assert!(matches!(
        scene.add_system(b),
        Err(SceneError::SystemError(SystemError::DependencyCycle))
    ));
    assert!(matches!(
        scene.get_system_id(b),
        Err(SceneError::SystemNotFound)
    ));
}

#[test]
fn unresolved_requested_type_ids_reject_the_system() {
    let requests = [TypeIDRequests::ComponentTypeID {
        component: c"Missing".as_ptr(),
    }];
    let systems = [system_definition(c"invalid", &[], &[], &requests)];
    let scene = Scene::new();
    let plugin = load_systems(&scene, &systems);
    let invalid = scene.resolve_system_type_id(plugin, "invalid").unwrap();

    assert!(matches!(
        scene.add_system(invalid),
        Err(SceneError::RequestedTypeIDNotFound)
    ));
    assert!(matches!(
        scene.get_system_id(invalid),
        Err(SceneError::SystemNotFound)
    ));
}
