use std::{
    ffi::c_void,
    sync::{Arc, Barrier},
    thread,
};

use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    field::AccessRequest,
    ids::{ComponentTypeID, FieldTypeID, PluginID},
    scene::Scene,
    utils::version::Version,
};

struct TestSingleton {
    first: usize,
    second: usize,
}

unsafe extern "C" fn create() -> *mut c_void {
    Box::into_raw(Box::new(TestSingleton {
        first: 1,
        second: 2,
    }))
    .cast()
}

unsafe extern "C" fn destroy(data: *mut c_void) {
    unsafe { drop(Box::from_raw(data.cast::<TestSingleton>())) };
}

unsafe extern "C" fn first(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*data.cast::<TestSingleton>()).first)
            .cast_mut()
            .cast()
    }
}

unsafe extern "C" fn second(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*data.cast::<TestSingleton>()).second)
            .cast_mut()
            .cast()
    }
}

const FIELDS: [ComponentFieldDefinition; 2] = [
    ComponentFieldDefinition {
        name: c"First".as_ptr(),
        getter: Some(first),
        mutable: 1,
        serializer: None,
        deserializer: None,
    },
    ComponentFieldDefinition {
        name: c"Second".as_ptr(),
        getter: Some(second),
        mutable: 1,
        serializer: None,
        deserializer: None,
    },
];

const COMPONENTS: [ComponentDefinition; 1] = [ComponentDefinition {
    name: c"Singleton".as_ptr(),
    creator: Some(create),
    destroyer: Some(destroy),
    fields: FIELDS.as_ptr(),
    field_count: FIELDS.len(),
}];

const PLUGIN: PluginDefinition = PluginDefinition {
    name: c"SingletonPlugin".as_ptr(),
    engine_version: Version {
        major: 0,
        minor: 2,
        patch: 0,
    },
    components: COMPONENTS.as_ptr(),
    component_count: COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

fn scene() -> (Scene, PluginID, ComponentTypeID, [FieldTypeID; 2]) {
    let scene = Scene::new();
    let plugin = unsafe { scene.load_static_plugin(PLUGIN) }.unwrap();
    let component = scene
        .resolve_component_type_id(plugin, "Singleton")
        .unwrap();
    let fields = [
        scene
            .resolve_field_type_id(plugin, component, "First")
            .unwrap(),
        scene
            .resolve_field_type_id(plugin, component, "Second")
            .unwrap(),
    ];
    (scene, plugin, component, fields)
}

#[test]
fn ensure_singleton_creates_and_reuses_one_entity() {
    let (scene, plugin, component, _) = scene();

    let created = scene.ensure_singleton(plugin, component).unwrap();
    let reused = scene.ensure_singleton(plugin, component).unwrap();

    assert_eq!(reused, created);
    assert_eq!(scene.get_entities(), vec![created]);
    assert_eq!(scene.get_components(created).unwrap().len(), 1);
}

#[test]
fn ensure_singleton_leaves_existing_duplicates_untouched() {
    let (scene, plugin, component, fields) = scene();
    let first = scene.add_entity();
    let second = scene.add_entity();
    scene.add_component(first, plugin, component).unwrap();
    scene.add_component(second, plugin, component).unwrap();

    let singleton = scene.ensure_singleton(plugin, component).unwrap();
    scene
        .query_singleton(
            (plugin, component, AccessRequest::Read, &fields[..1]),
            |pointers| assert_eq!(pointers.len(), 1),
        )
        .unwrap();

    assert!([first, second].contains(&singleton));
    assert_eq!(scene.get_entities(), vec![first, second]);
}

#[test]
fn concurrent_ensure_singleton_calls_create_one_entity() {
    let (scene, plugin, component, _) = scene();
    let scene = Arc::new(scene);
    let barrier = Arc::new(Barrier::new(8));
    let threads = (0..8)
        .map(|_| {
            let scene = Arc::clone(&scene);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                scene.ensure_singleton(plugin, component).unwrap()
            })
        })
        .collect::<Vec<_>>();
    let entities = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();

    assert!(entities.iter().all(|entity| *entity == entities[0]));
    assert_eq!(scene.get_entities(), vec![entities[0]]);
}

#[test]
fn query_singleton_ensures_and_queries_one_component() {
    let (scene, plugin, component, fields) = scene();

    scene
        .query_singleton(
            (plugin, component, AccessRequest::Write, &fields),
            |pointers| {
                assert_eq!(unsafe { *pointers[0].cast::<usize>() }, 1);
                assert_eq!(unsafe { *pointers[1].cast::<usize>() }, 2);
                unsafe { *pointers[0].cast::<usize>() = 3 };
            },
        )
        .unwrap();
    scene
        .query_singleton(
            (plugin, component, AccessRequest::Read, &fields[..1]),
            |pointers| assert_eq!(unsafe { *pointers[0].cast::<usize>() }, 3),
        )
        .unwrap();

    assert_eq!(scene.get_entities().len(), 1);
}
