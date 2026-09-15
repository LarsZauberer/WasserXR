use std::{ffi::c_void, ptr::null_mut, sync::Mutex};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        components::ComponentDefinition, fields::ComponentFieldDefinition,
        plugins::PluginDefinition,
    },
    errors::{ComponentError, EntityError, FieldError, SceneError},
    field::{Field, FieldAccess},
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

const VALID_COMPONENT_FIELD_PLUGIN: PluginDefinition = PluginDefinition {
    name: c"MyPlugin".as_ptr(),
    engine_version: COMPATIBLE_ENGINE_VERSION,
    components: &VALID_COMPONENT_WITH_FIELD,
    component_count: 1,
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
fn component_fields_enforce_mutability(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let (plugin, component_type, added_component) = add_test_component(&scene, entity).unwrap();
    let component = scene
        .resolve_component_id(entity, plugin, component_type)
        .unwrap();
    assert_eq!(component, added_component);
    let mutable_field_type = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let mutable_field = scene
        .resolve_field_id(entity, component, mutable_field_type)
        .unwrap();

    let immutable_field_type = scene
        .resolve_field_type_id(plugin, component_type, "ImmutableField")
        .unwrap();
    let immutable_field = scene
        .resolve_field_id(entity, component, immutable_field_type)
        .unwrap();

    let requests = [
        (mutable_field, FieldAccess::Write),
        (immutable_field, FieldAccess::Read),
    ];
    scene
        .query_component_fields(entity, component, &requests, |fields| {
            for ((id, access), field) in requests.into_iter().zip(fields) {
                assert!(matches!(
                    (access, field),
                    (FieldAccess::Read, Field::Read(field_id, _))
                        | (FieldAccess::Write, Field::Write(field_id, _))
                        if id == *field_id
                ));
            }
            assert!(
                fields
                    .iter()
                    .any(|field| matches!(field, Field::Write(_, pointer) if pointer.is_null()))
            );
        })
        .unwrap();

    assert!(matches!(
        scene.query_component_fields(
            entity,
            component,
            &[(immutable_field, FieldAccess::Write)],
            |_| ()
        ),
        Err(SceneError::EntityError(EntityError::ComponentError(
            ComponentError::FieldError(FieldError::NotMutable)
        )))
    ));
    drop(scene);
}

#[rstest]
fn component_fields_keep_requested_order(scene: Scene) {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_globals();
    let entity = scene.add_entity();
    let (plugin, component_type, component) = add_test_component(&scene, entity).unwrap();
    let first_type = scene
        .resolve_field_type_id(plugin, component_type, "MyField")
        .unwrap();
    let first = scene
        .resolve_field_id(entity, component, first_type)
        .unwrap();
    let second_type = scene
        .resolve_field_type_id(plugin, component_type, "ImmutableField")
        .unwrap();
    let second = scene
        .resolve_field_id(entity, component, second_type)
        .unwrap();
    let requests = if first < second {
        [(second, FieldAccess::Read), (first, FieldAccess::Read)]
    } else {
        [(first, FieldAccess::Read), (second, FieldAccess::Read)]
    };

    scene
        .query_component_fields(entity, component, &requests, |fields| {
            let returned = fields
                .iter()
                .map(|field| match field {
                    Field::Read(id, _) | Field::Write(id, _) => *id,
                })
                .collect::<Vec<_>>();
            assert_eq!(returned, requests.map(|(id, _)| id));
        })
        .unwrap();
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
