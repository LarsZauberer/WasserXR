use std::{ffi::c_void, ptr::null_mut, sync::Mutex};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    errors::{EntityError, SceneError},
    scene::{EntityID, Scene},
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

const VALID_COMPONENT_WITH_FIELD: ComponentDefinition = ComponentDefinition {
    name: c"MyComponent".as_ptr(),
    creator: Some(simple_creator),
    destroyer: Some(simple_destroyer),
    fields: &VALID_COMPONENT_FIELD,
    field_count: 1,
};

const VALID_COMPONENT_FIELD_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: &VALID_COMPONENT_WITH_FIELD,
    component_count: 1,
    assets: std::ptr::null(),
    asset_count: 0,
};

fn reset_globals() {
    *CREATOR_COUNTER.lock().unwrap() = 0;
    *DESTROYER_COUNTER.lock().unwrap() = 0;
}

#[fixture]
fn scene() -> Scene {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(VALID_COMPONENT_FIELD_PLUGIN) }
        .expect("Failed to load valid plugin");
    scene
}

#[rstest]
fn empty_scene_cannot_add_component() {
    let mut scene = Scene::new();

    let entity_id = scene.add_entity();
    let err = scene
        .add_component(entity_id, "MyComponent")
        .expect_err("Added a component to a scene with no plugins");

    assert!(matches!(err, SceneError::NoComponentType));
}

#[rstest]
fn entity_cannot_have_duplicate_component(mut scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity_id = scene.add_entity();
    scene
        .add_component(entity_id, "MyComponent")
        .expect("Failed to add the component");
    let err = scene
        .add_component(entity_id, "MyComponent")
        .expect_err("Added duplicate of the same component");
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
fn component_lifecycle(mut scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    // Add entities
    let entity1 = scene.add_entity();
    let entity2 = scene.add_entity();

    // Add component
    let my_component_id = scene
        .add_component(entity1, "MyComponent")
        .expect("Failed to add component to entity1");

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
fn component_is_scoped_to_entity(mut scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    // Add entities
    let entity1 = scene.add_entity();
    let entity2 = scene.add_entity();

    // Add component
    let my_component_id = scene
        .add_component(entity1, "MyComponent")
        .expect("Failed to add component to entity1");

    let err = scene
        .resolve_component_id(entity2, "MyComponent")
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
fn component_cannot_be_removed_twice(mut scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let component = scene
        .add_component(entity, "MyComponent")
        .expect("Failed to add component");
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
