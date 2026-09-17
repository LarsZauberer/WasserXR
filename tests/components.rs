use std::{
    ffi::c_void,
    sync::{Arc, Barrier, Mutex, mpsc},
    thread,
    time::Duration,
};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        components::ComponentDefinition,
        fields::{ComponentFieldDefinition, TypeHint},
        plugins::PluginDefinition,
    },
    errors::{ComponentError, EntityError, FieldError, SceneError},
    field::AccessRequest,
    ids::{ComponentID, ComponentTypeID, EntityID, FieldTypeID, PluginID},
    scene::Scene,
    utils::version::Version,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());
static CREATOR_COUNTER: Mutex<usize> = Mutex::new(0);
static DESTROYER_COUNTER: Mutex<usize> = Mutex::new(0);

/// Concrete data makes query ordering and mutation observable.
struct TestComponent {
    mutable: usize,
    immutable: usize,
}

/// Allocates distinct field values for each component created by a test.
unsafe extern "C" fn simple_creator() -> *mut c_void {
    let mut count = CREATOR_COUNTER.lock().unwrap();
    *count += 1;
    Box::into_raw(Box::new(TestComponent {
        mutable: *count,
        immutable: 100 + *count,
    }))
    .cast()
}

/// Releases the component and records its destruction.
unsafe extern "C" fn simple_destroyer(data: *mut c_void) {
    *DESTROYER_COUNTER.lock().unwrap() += 1;
    unsafe { drop(Box::from_raw(data.cast::<TestComponent>())) };
}

/// Exposes the mutable field without creating a Rust reference to plugin data.
unsafe extern "C" fn simple_getter(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*data.cast::<TestComponent>()).mutable)
            .cast_mut()
            .cast()
    }
}

/// Exposes the immutable field for read queries and rejected write queries.
unsafe extern "C" fn immutable_getter(data: *const c_void) -> *mut c_void {
    unsafe {
        (&raw const (*data.cast::<TestComponent>()).immutable)
            .cast_mut()
            .cast()
    }
}

const COMPATIBLE_ENGINE_VERSION: Version = Version {
    major: 0,
    minor: 2,
    patch: 0,
};

const VALID_COMPONENT_FIELD: ComponentFieldDefinition = ComponentFieldDefinition {
    name: c"MyField".as_ptr(),
    type_hint: TypeHint::Usize as u32,
    getter: Some(simple_getter),
    mutable: 1,
    serializer: None,
    deserializer: None,
};

const IMMUTABLE_COMPONENT_FIELD: ComponentFieldDefinition = ComponentFieldDefinition {
    name: c"ImmutableField".as_ptr(),
    type_hint: TypeHint::Usize as u32,
    getter: Some(immutable_getter),
    mutable: 0,
    serializer: None,
    deserializer: None,
};

const VALID_COMPONENT_FIELDS: [ComponentFieldDefinition; 3] = [
    VALID_COMPONENT_FIELD,
    IMMUTABLE_COMPONENT_FIELD,
    ComponentFieldDefinition {
        name: c"HiddenField".as_ptr(),
        type_hint: TypeHint::Usize as u32,
        getter: None,
        mutable: 0,
        serializer: None,
        deserializer: None,
    },
];

const VALID_COMPONENT_WITH_FIELD: ComponentDefinition = ComponentDefinition {
    name: c"MyComponent".as_ptr(),
    creator: Some(simple_creator),
    destroyer: Some(simple_destroyer),
    fields: VALID_COMPONENT_FIELDS.as_ptr(),
    field_count: VALID_COMPONENT_FIELDS.len(),
};

const OTHER_COMPONENT_WITH_FIELD: ComponentDefinition = ComponentDefinition {
    name: c"OtherComponent".as_ptr(),
    creator: Some(simple_creator),
    destroyer: Some(simple_destroyer),
    fields: VALID_COMPONENT_FIELDS.as_ptr(),
    field_count: VALID_COMPONENT_FIELDS.len(),
};

const COMPONENTS: [ComponentDefinition; 2] =
    [VALID_COMPONENT_WITH_FIELD, OTHER_COMPONENT_WITH_FIELD];

const VALID_COMPONENT_FIELD_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: COMPONENTS.as_ptr(),
    component_count: COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

fn reset_globals() {
    *CREATOR_COUNTER.lock().unwrap() = 0;
    *DESTROYER_COUNTER.lock().unwrap() = 0;
}

fn add_test_component(
    scene: &Scene,
    entity: EntityID,
) -> Result<(PluginID, ComponentTypeID, ComponentID), SceneError> {
    let plugin = scene
        .resolve_plugin_id("MyPlugin")
        .ok_or(SceneError::NoComponentType)?;
    let component_type = scene.resolve_component_type_id(plugin, "MyComponent")?;
    scene
        .add_component(entity, plugin, component_type)
        .map(|component| (plugin, component_type, component))
}

#[fixture]
fn scene() -> Scene {
    let scene = Scene::new();
    unsafe { scene.load_static_plugin(VALID_COMPONENT_FIELD_PLUGIN) }
        .expect("Failed to load valid plugin");
    scene
}

#[rstest]
fn resolves_component_field_type_hint(scene: Scene) {
    let plugin = scene.resolve_plugin_id("MyPlugin").unwrap();
    let component = scene
        .resolve_component_type_id(plugin, "MyComponent")
        .unwrap();
    let field = scene
        .resolve_field_type_id(plugin, component, "MyField")
        .unwrap();

    assert_eq!(
        scene.get_field_type(plugin, component, field).unwrap(),
        TypeHint::Usize
    );
    assert!(matches!(
        scene.get_field_type(plugin, component, FieldTypeID::default()),
        Err(SceneError::ComponentError(ComponentError::FieldNotFound))
    ));
}

#[rstest]
fn empty_scene_cannot_add_component() {
    let scene = Scene::new();

    let entity_id = scene.add_entity();
    let err = add_test_component(&scene, entity_id)
        .expect_err("Added a component to a scene with no plugins");

    assert!(matches!(err, SceneError::NoComponentType));
}

/// Checks intersection, flat pointer order, duplicate locks, and field
/// mutability.
#[rstest]
fn component_query_matches_entities_preserves_order_and_calls_action_once(scene: Scene) {
    use std::cell::Cell;

    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let first_entity = scene.add_entity();
    let second_entity = scene.add_entity();
    let (plugin, component_type, _) = add_test_component(&scene, first_entity).unwrap();
    add_test_component(&scene, second_entity).unwrap();
    let other_type = scene
        .resolve_component_type_id(plugin, "OtherComponent")
        .unwrap();
    scene
        .add_component(first_entity, plugin, other_type)
        .unwrap();

    let mutable_field = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let immutable_field = scene
        .resolve_field_type_id(plugin, component_type, "ImmutableField")
        .unwrap();
    let other_field = scene
        .resolve_field_type_id(plugin, other_type, "ImmutableField")
        .unwrap();
    let requested_fields = [immutable_field, mutable_field];
    let read_fields = [mutable_field];
    let other_fields = [other_field];
    let all_with_component = [(
        plugin,
        component_type,
        AccessRequest::Read,
        requested_fields.as_slice(),
    )];
    let only_first_entity = [
        (
            plugin,
            component_type,
            AccessRequest::Read,
            read_fields.as_slice(),
        ),
        (
            plugin,
            other_type,
            AccessRequest::Read,
            other_fields.as_slice(),
        ),
        (
            plugin,
            component_type,
            AccessRequest::Write,
            read_fields.as_slice(),
        ),
    ];
    let calls = Cell::new(0);

    scene
        .query_components(&all_with_component, |results| {
            calls.set(calls.get() + 1);
            assert_eq!(
                results
                    .iter()
                    .map(|pointer| unsafe { *pointer.cast::<usize>() })
                    .collect::<Vec<_>>(),
                [101, 1, 102, 2]
            );
        })
        .unwrap();

    assert_eq!(calls.get(), 1);

    scene
        .query_components(&only_first_entity, |results| {
            calls.set(calls.get() + 1);
            assert_eq!(results.len(), 3);
            assert_eq!(results[0], results[2]);
            assert_eq!(unsafe { *results[1].cast::<usize>() }, 103);
            unsafe { *results[2].cast::<usize>() = 7 };
        })
        .unwrap();

    assert_eq!(calls.get(), 2);

    let immutable_fields = [immutable_field];
    let component = [
        (
            plugin,
            component_type,
            AccessRequest::Read,
            immutable_fields.as_slice(),
        ),
        (
            plugin,
            component_type,
            AccessRequest::Write,
            immutable_fields.as_slice(),
        ),
    ];
    assert!(matches!(
        scene.query_components(&component, |_| panic!("invalid query called callback")),
        Err(SceneError::EntityError(EntityError::ComponentError(
            ComponentError::FieldError(FieldError::NotMutable)
        )))
    ));
    // The failed query released its locks, and the previous write is visible.
    scene
        .query_components(&all_with_component, |results| {
            assert_eq!(unsafe { *results[1].cast::<usize>() }, 7);
        })
        .unwrap();
    drop(scene);
}

/// A write to one field excludes reads of other fields in the same component.
#[rstest]
fn component_write_query_conflicts_with_reads_of_other_fields(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let (plugin, component_type, _) = add_test_component(&scene, entity).unwrap();
    let mutable_field = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let immutable_field = scene
        .resolve_field_type_id(plugin, component_type, "ImmutableField")
        .unwrap();
    let read_fields = [immutable_field];
    let write_fields = [mutable_field];
    let read_request = [(
        plugin,
        component_type,
        AccessRequest::Read,
        read_fields.as_slice(),
    )];
    let write_request = [(
        plugin,
        component_type,
        AccessRequest::Write,
        write_fields.as_slice(),
    )];
    let (read_locked, read_locked_rx) = mpsc::channel();
    let (release_read, release_read_rx) = mpsc::channel();
    let (write_started, write_started_rx) = mpsc::channel();
    let (write_locked, write_locked_rx) = mpsc::channel();

    thread::scope(|scope| {
        let scene = &scene;
        scope.spawn(move || {
            scene
                .query_components(&read_request, |_| {
                    read_locked.send(()).unwrap();
                    release_read_rx.recv().unwrap();
                })
                .unwrap();
        });
        read_locked_rx.recv().unwrap();

        scope.spawn(move || {
            write_started.send(()).unwrap();
            scene
                .query_components(&write_request, |_| write_locked.send(()).unwrap())
                .unwrap();
        });
        write_started_rx.recv().unwrap();
        assert!(
            write_locked_rx
                .recv_timeout(Duration::from_millis(200))
                .is_err()
        );

        release_read.send(()).unwrap();
        write_locked_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
    });

    drop(scene);
}

/// Covers empty groups, missing components/fields, and mixed access to one
/// component.
#[rstest]
fn component_query_edge_cases(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let (plugin, component_type, _) = add_test_component(&scene, entity).unwrap();
    let mutable = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let immutable = scene
        .resolve_field_type_id(plugin, component_type, "ImmutableField")
        .unwrap();
    let hidden = scene
        .resolve_field_type_id(plugin, component_type, "HiddenField")
        .unwrap();
    let other = scene
        .resolve_component_type_id(plugin, "OtherComponent")
        .unwrap();
    let mut calls = 0;
    for query in [
        vec![],
        vec![(plugin, other, AccessRequest::Read, &[][..])],
        vec![(plugin, component_type, AccessRequest::Write, &[][..])],
    ] {
        scene
            .query_components(&query, |pointers| {
                calls += 1;
                assert!(pointers.is_empty());
            })
            .unwrap();
    }
    assert_eq!(calls, 3);
    let error = scene
        .query_components(
            &[(
                plugin,
                component_type,
                AccessRequest::Read,
                &[FieldTypeID::default()],
            )],
            |_| panic!("missing field called callback"),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SceneError::EntityError(EntityError::ComponentError(ComponentError::FieldNotFound))
    ));
    let error = scene
        .query_components(
            &[(plugin, component_type, AccessRequest::Read, &[hidden])],
            |_| panic!("hidden field called callback"),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SceneError::EntityError(EntityError::ComponentError(ComponentError::FieldError(
            FieldError::NoGetter
        )))
    ));
    scene
        .query_components(
            &[
                (plugin, component_type, AccessRequest::Read, &[immutable]),
                (
                    plugin,
                    component_type,
                    AccessRequest::Write,
                    &[mutable, mutable],
                ),
            ],
            |pointers| {
                assert_eq!(unsafe { *pointers[0].cast::<usize>() }, 101);
                assert_eq!(pointers[1], pointers[2]);
                unsafe { *pointers[1].cast::<usize>() = 9 };
            },
        )
        .unwrap();
    drop(scene);
}

/// Opposite request orders must finish and serialize writes across all
/// entities.
#[rstest]
fn reversed_component_queries_do_not_deadlock(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let first = scene.add_entity();
    let (plugin, component_type, _) = add_test_component(&scene, first).unwrap();
    let other = scene
        .resolve_component_type_id(plugin, "OtherComponent")
        .unwrap();
    scene.add_component(first, plugin, other).unwrap();
    let second = scene.add_entity();
    // Reverse concrete component IDs on the second entity as well.
    scene.add_component(second, plugin, other).unwrap();
    add_test_component(&scene, second).unwrap();
    let field = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let other_field = scene
        .resolve_field_type_id(plugin, other, "MyField")
        .unwrap();
    let scene = Arc::new(scene);
    let start = Arc::new(Barrier::new(2));
    let (done, finished) = mpsc::channel();
    let handles: Vec<_> = [false, true]
        .into_iter()
        .map(|reverse| {
            let scene = Arc::clone(&scene);
            let start = Arc::clone(&start);
            let done = done.clone();
            thread::spawn(move || {
                let mut query = [
                    (plugin, component_type, AccessRequest::Write, &[field][..]),
                    (plugin, other, AccessRequest::Write, &[other_field][..]),
                ];
                if reverse {
                    query.reverse();
                }
                for _ in 0..100 {
                    start.wait();
                    scene
                        .query_components(&query, |pointers| {
                            for &pointer in pointers {
                                unsafe { *pointer.cast::<usize>() += 1 };
                            }
                        })
                        .unwrap();
                }
                done.send(()).unwrap();
            })
        })
        .collect();
    for _ in 0..2 {
        finished
            .recv_timeout(Duration::from_secs(5))
            .expect("queries deadlocked");
    }
    for handle in handles {
        handle.join().unwrap();
    }
    scene
        .query_components(
            &[
                (plugin, component_type, AccessRequest::Read, &[field]),
                (plugin, other, AccessRequest::Read, &[other_field]),
            ],
            |pointers| {
                let values: Vec<_> = pointers
                    .iter()
                    .map(|pointer| unsafe { *pointer.cast::<usize>() })
                    .collect();
                assert_eq!(values, [201, 202, 204, 203]);
            },
        )
        .unwrap();
    drop(scene);
}

/// Shared readers overlap, while component/entity removal waits for the
/// callback.
#[rstest]
fn component_queries_keep_owners_alive(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let scene = Arc::new(scene);
    for remove_entity in [false, true] {
        let entity = scene.add_entity();
        let (plugin, component_type, component) = add_test_component(&scene, entity).unwrap();
        let field = scene
            .resolve_field_type_id(plugin, component_type, "MyField")
            .unwrap();
        let (locked, locked_rx) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let reader_scene = Arc::clone(&scene);
        let reader = thread::spawn(move || {
            reader_scene
                .query_components(
                    &[(plugin, component_type, AccessRequest::Read, &[field])],
                    |pointers| {
                        locked.send(()).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                        assert!(unsafe { *pointers[0].cast::<usize>() } > 0);
                    },
                )
                .unwrap();
        });
        locked_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        scene
            .query_components(
                &[(plugin, component_type, AccessRequest::Read, &[field])],
                |_| (),
            )
            .unwrap();
        let (started, started_rx) = mpsc::channel();
        let (removed, removed_rx) = mpsc::channel();
        let remover_scene = Arc::clone(&scene);
        let remover = thread::spawn(move || {
            started.send(()).unwrap();
            if remove_entity {
                remover_scene.remove_entity(entity).unwrap();
            } else {
                remover_scene.remove_component(entity, component).unwrap();
            }
            removed.send(()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(removed_rx.recv_timeout(Duration::from_millis(100)).is_err());
        release.send(()).unwrap();
        removed_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        reader.join().unwrap();
        remover.join().unwrap();
    }
    drop(scene);
}

#[rstest]
fn entity_cannot_have_duplicate_component(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity_id = scene.add_entity();
    add_test_component(&scene, entity_id).expect("Failed to add the component");
    let err =
        add_test_component(&scene, entity_id).expect_err("Added duplicate of the same component");
    assert!(matches!(
        err,
        SceneError::EntityError(EntityError::ComponentAlreadyExists)
    ));
    // Drop the component before releasing TEST_LOCK so its destroyer cannot race
    // another test's counters.
    drop(scene);
}

fn get_vec_of_component_names(scene: &Scene, entity_id: EntityID) -> Vec<String> {
    let components = scene
        .get_components(entity_id)
        .expect("Entity has to exist");
    components
        .iter()
        .map(|c| {
            scene
                .get_component_name(entity_id, *c)
                .expect("Component exists")
                .to_owned()
        })
        .collect()
}

#[rstest]
fn component_lifecycle(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    // Add entities
    let entity1 = scene.add_entity();
    let entity2 = scene.add_entity();

    // Add component
    let (_, _, my_component_id) =
        add_test_component(&scene, entity1).expect("Failed to add component to entity1");

    // Check component add status
    assert_eq!(
        get_vec_of_component_names(&scene, entity1),
        &["MyComponent"]
    );
    assert!(get_vec_of_component_names(&scene, entity2).is_empty());

    // Remove components
    scene
        .remove_component(entity1, my_component_id)
        .expect("Failed to remove the component from entity1");

    // Check component status
    assert!(get_vec_of_component_names(&scene, entity1).is_empty());
    assert!(get_vec_of_component_names(&scene, entity2).is_empty());

    // Check the call count of creator and destroyer
    assert_eq!(*CREATOR_COUNTER.lock().unwrap(), 1);
    assert_eq!(*DESTROYER_COUNTER.lock().unwrap(), 1);
}

#[rstest]
fn component_is_scoped_to_entity(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    // Add entities
    let entity1 = scene.add_entity();
    let entity2 = scene.add_entity();

    // Add component
    let (plugin, component_type, my_component_id) =
        add_test_component(&scene, entity1).expect("Failed to add component to entity1");

    let err = scene
        .resolve_component_id(entity2, plugin, component_type)
        .expect_err("Got a component that shouldn't exist");
    assert!(matches!(
        err,
        SceneError::EntityError(EntityError::ComponentNotFound)
    ));

    scene
        .remove_component(entity1, my_component_id)
        .expect("Failed to remove the component from entity1");
}

#[rstest]
fn component_cannot_be_removed_twice(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let (_, _, component) = add_test_component(&scene, entity).expect("Failed to add component");
    scene
        .remove_component(entity, component)
        .expect("Failed to remove component");

    // Check double remove
    let err = scene
        .remove_component(entity, component)
        .expect_err("Removed the same component twice");
    assert!(matches!(
        err,
        SceneError::EntityError(EntityError::ComponentNotFound)
    ));
}
