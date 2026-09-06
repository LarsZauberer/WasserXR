use std::{ffi::c_void, ptr::null_mut, sync::Mutex};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    errors::SceneError,
    scene::{EntityID, Scene},
    utils::version::Version,
};

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
};

fn reset_globals() {
    *CREATOR_COUNTER.lock().unwrap() = 0;
    *DESTROYER_COUNTER.lock().unwrap() = 0;
}

#[fixture]
fn scene() -> Scene {
    reset_globals();
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
    let entity_id = scene.add_entity();
    scene
        .add_component(entity_id, "MyComponent")
        .expect("Failed to add the component");
    scene
        .add_component(entity_id, "MyComponent")
        .expect_err("Added duplicate of the same component");
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

    let empty_string_list: [String; 0] = [];
    assert_eq!(
        get_vec_of_component_names(&scene, entity2),
        &empty_string_list
    );

    // Remove components
    scene
        .remove_component(entity1, my_component_id)
        .expect("Failed to remove the component form entity1");
    scene
        .remove_component(entity2, my_component_id)
        .expect_err("Removed a none existent component from entity2");

    // Check double remove
    scene
        .remove_component(entity1, my_component_id)
        .expect_err("Removed the same component twice from entity1");

    // Check component status
    assert_eq!(
        get_vec_of_component_names(&scene, entity1),
        &empty_string_list
    );
    assert_eq!(
        get_vec_of_component_names(&scene, entity2),
        &empty_string_list
    );

    // Check the call count of creator and destroyer
    assert_eq!(*CREATOR_COUNTER.lock().unwrap(), 1);
    assert_eq!(*DESTROYER_COUNTER.lock().unwrap(), 1);
}
