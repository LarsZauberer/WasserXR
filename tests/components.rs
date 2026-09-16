use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::{Mutex, mpsc},
    thread,
    time::Duration,
};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    errors::{ComponentError, EntityError, FieldError, SceneError},
    field::FieldAccess,
    ids::{ComponentID, ComponentTypeID, EntityID, PluginID},
    scene::Scene,
    utils::version::Version,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());
static CREATOR_COUNTER: Mutex<usize> = Mutex::new(0);
static DESTROYER_COUNTER: Mutex<usize> = Mutex::new(0);

unsafe extern "C" fn simple_creator() -> *mut c_void {
    *CREATOR_COUNTER.lock().unwrap() += 1;
    null_mut()
}

unsafe extern "C" fn simple_destroyer(_: *mut c_void) {
    *DESTROYER_COUNTER.lock().unwrap() += 1;
}

unsafe extern "C" fn simple_getter(_: *const c_void) -> *mut c_void {
    null_mut()
}

const COMPATIBLE_ENGINE_VERSION: Version = Version {
    major: 0,
    minor: 2,
    patch: 0,
};

const VALID_COMPONENT_FIELD: ComponentFieldDefinition = ComponentFieldDefinition {
    name: c"MyField".as_ptr(),
    getter: Some(simple_getter),
    mutable: 1,
    serializer: None,
    deserializer: None,
};

const IMMUTABLE_COMPONENT_FIELD: ComponentFieldDefinition = ComponentFieldDefinition {
    name: c"ImmutableField".as_ptr(),
    getter: Some(simple_getter),
    mutable: 0,
    serializer: None,
    deserializer: None,
};

const VALID_COMPONENT_FIELDS: [ComponentFieldDefinition; 2] =
    [VALID_COMPONENT_FIELD, IMMUTABLE_COMPONENT_FIELD];

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
fn empty_scene_cannot_add_component() {
    let scene = Scene::new();

    let entity_id = scene.add_entity();
    let err = add_test_component(&scene, entity_id)
        .expect_err("Added a component to a scene with no plugins");

    assert!(matches!(err, SceneError::NoComponentType));
}

#[rstest]
fn component_query_matches_entities_preserves_order_and_calls_action_once(scene: Scene) {
    use std::cell::Cell;

    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let first_entity = scene.add_entity();
    let second_entity = scene.add_entity();
    let (plugin, component_type, first_component) =
        add_test_component(&scene, first_entity).unwrap();
    add_test_component(&scene, second_entity).unwrap();
    let other_type = scene
        .resolve_component_type_id(plugin, "OtherComponent")
        .unwrap();
    let other_component = scene
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
    let first_mutable_field = scene
        .resolve_field_id(first_entity, first_component, mutable_field)
        .unwrap();
    let first_immutable_field = scene
        .resolve_field_id(first_entity, first_component, immutable_field)
        .unwrap();
    let first_other_field = scene
        .resolve_field_id(first_entity, other_component, other_field)
        .unwrap();
    let requested_fields = if first_mutable_field < first_immutable_field {
        [
            (immutable_field, FieldAccess::Read),
            (mutable_field, FieldAccess::Write),
        ]
    } else {
        [
            (mutable_field, FieldAccess::Write),
            (immutable_field, FieldAccess::Read),
        ]
    };
    let requested_field_ids = if first_mutable_field < first_immutable_field {
        [first_immutable_field, first_mutable_field]
    } else {
        [first_mutable_field, first_immutable_field]
    };
    let read_fields = [(mutable_field, FieldAccess::Read)];
    let duplicate_write_fields = [(mutable_field, FieldAccess::Write)];
    let other_fields = [(other_field, FieldAccess::Read)];
    let all_with_component = [(plugin, component_type, requested_fields.as_slice())];
    let only_first_entity = [
        (plugin, component_type, read_fields.as_slice()),
        (plugin, other_type, other_fields.as_slice()),
        (plugin, component_type, duplicate_write_fields.as_slice()),
    ];
    let calls = Cell::new(0);

    scene
        .query_components(&all_with_component, |results| {
            calls.set(calls.get() + 1);
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].0, first_entity);
            assert_eq!(results[1].0, second_entity);
            assert_eq!(results[0].1[0].0, first_component);
            assert_eq!(
                results[0].1[0]
                    .1
                    .iter()
                    .map(|(field_id, _)| *field_id)
                    .collect::<Vec<_>>(),
                requested_field_ids
            );
        })
        .unwrap();

    assert_eq!(calls.get(), 1);

    scene
        .query_components(&only_first_entity, |results| {
            calls.set(calls.get() + 1);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0, first_entity);
            assert_eq!(results[0].1.len(), 3);
            assert_eq!(results[0].1[0].1[0].0, first_mutable_field);
            assert_eq!(results[0].1[1].1[0].0, first_other_field);
            assert_eq!(results[0].1[2].0, first_component);
            assert_eq!(results[0].1[2].1[0].0, first_mutable_field);
            assert_eq!(results[0].1[0].1[0].1, results[0].1[2].1[0].1);
        })
        .unwrap();

    assert_eq!(calls.get(), 2);

    let immutable_read = [(immutable_field, FieldAccess::Read)];
    let immutable_write = [(immutable_field, FieldAccess::Write)];
    let component = [
        (plugin, component_type, immutable_read.as_slice()),
        (plugin, component_type, immutable_write.as_slice()),
    ];
    assert!(matches!(
        scene.query_components(&component, |_| ()),
        Err(SceneError::EntityError(EntityError::ComponentError(
            ComponentError::FieldError(FieldError::NotMutable)
        )))
    ));
    drop(scene);
}

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
    let read_fields = [(immutable_field, FieldAccess::Read)];
    let write_fields = [(mutable_field, FieldAccess::Write)];
    let read_request = [(plugin, component_type, read_fields.as_slice())];
    let write_request = [(plugin, component_type, write_fields.as_slice())];
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
