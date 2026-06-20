use std::sync::{LazyLock, Mutex};

use uuid::Uuid;
use wasserxr::{
    component,
    error::{ComponentError, SceneError},
    scene::Scene,
    system,
};

#[component]
#[derive(Default)]
pub struct MacroComponent {
    #[getter]
    my_int: i32,

    default_int: i32,

    #[getter]
    #[setter]
    #[serializer]
    #[deserializer]
    my_string: String,

    default_string: String,

    #[none]
    #[allow(dead_code)]
    hidden: i32,
}

#[component]
#[derive(Default)]
pub struct MacroRoundTripComponent {
    #[getter]
    #[setter]
    #[serializer]
    #[deserializer]
    health: i32,

    #[getter]
    #[setter]
    label: String,
}

#[component]
#[derive(Default)]
pub struct MacroCounter {
    #[allow(dead_code)]
    value: i64,
}

#[component]
#[derive(Default)]
pub struct MacroMarker {
    #[allow(dead_code)]
    value: i64,
}

static MACRO_SYSTEM_ENTITIES: LazyLock<Mutex<Vec<Vec<Uuid>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static MACRO_EMPTY_SYSTEM_ENTITIES: LazyLock<Mutex<Option<Vec<Vec<Uuid>>>>> =
    LazyLock::new(|| Mutex::new(None));
static MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES: LazyLock<Mutex<Option<Vec<Vec<Uuid>>>>> =
    LazyLock::new(|| Mutex::new(None));

#[system(entities = [["MacroCounter"], ["MacroMarker"]])]
pub fn macro_group_counter(_scene: &mut Scene, entities: Vec<Vec<Uuid>>) {
    *MACRO_SYSTEM_ENTITIES.lock().unwrap() = entities;
}

#[system]
pub fn macro_empty_system(_scene: &mut Scene, entities: Vec<Vec<Uuid>>) {
    *MACRO_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = Some(entities);
}

#[system(entities = [])]
pub fn macro_explicit_empty_system(_scene: &mut Scene, entities: Vec<Vec<Uuid>>) {
    *MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = Some(entities);
}

#[test]
fn component_macro_round_trip_keeps_component_behavior() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroRoundTripComponent".to_owned())
        .unwrap();

    scene
        .set(entity, "MacroRoundTripComponent", "health", &24_i32)
        .unwrap();

    let label = "round trip".to_owned();
    scene
        .set(entity, "MacroRoundTripComponent", "label", &label)
        .unwrap();

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    assert!(loaded.has_component(entity, "MacroRoundTripComponent"));
    assert_eq!(
        *loaded
            .get::<i32>(entity, "MacroRoundTripComponent", "health")
            .unwrap(),
        24
    );
    assert_eq!(
        loaded
            .get::<String>(entity, "MacroRoundTripComponent", "label")
            .unwrap(),
        ""
    );

    loaded
        .set(entity, "MacroRoundTripComponent", "health", &30_i32)
        .unwrap();
    assert_eq!(
        *loaded
            .get::<i32>(entity, "MacroRoundTripComponent", "health")
            .unwrap(),
        30
    );
}

#[test]
fn component_macro_registers_and_accesses_static_component() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    assert_eq!(
        *scene
            .get::<i32>(entity, "MacroComponent", "my_int")
            .unwrap(),
        0
    );
    assert_eq!(
        scene
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        ""
    );
    assert_eq!(
        *scene
            .get::<i32>(entity, "MacroComponent", "default_int")
            .unwrap(),
        0
    );

    assert_eq!(
        scene.set(entity, "MacroComponent", "my_int", &7_i32),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoSetter
        ))
    );

    let updated = "updated through setter".to_owned();
    scene
        .set(entity, "MacroComponent", "my_string", &updated)
        .unwrap();
    scene
        .set(entity, "MacroComponent", "default_int", &41_i32)
        .unwrap();

    assert_eq!(
        scene
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        "updated through setter"
    );
    assert_eq!(
        *scene
            .get::<i32>(entity, "MacroComponent", "default_int")
            .unwrap(),
        41
    );
    assert_eq!(
        scene.get::<i32>(entity, "MacroComponent", "hidden"),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoGetter
        ))
    );
    assert_eq!(
        scene.set(entity, "MacroComponent", "hidden", &7_i32),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoSetter
        ))
    );
}

#[test]
fn component_macro_serializes_static_component_fields() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    let updated = "serialized through macro".to_owned();
    scene
        .set(entity, "MacroComponent", "my_string", &updated)
        .unwrap();
    scene
        .set(entity, "MacroComponent", "default_int", &12_i32)
        .unwrap();
    let default_string = "default serialization".to_owned();
    scene
        .set(entity, "MacroComponent", "default_string", &default_string)
        .unwrap();

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    assert_eq!(
        *loaded
            .get::<i32>(entity, "MacroComponent", "my_int")
            .unwrap(),
        0
    );
    assert_eq!(
        *loaded
            .get::<i32>(entity, "MacroComponent", "default_int")
            .unwrap(),
        12
    );
    assert_eq!(
        loaded
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        "serialized through macro"
    );
    assert_eq!(
        loaded
            .get::<String>(entity, "MacroComponent", "default_string")
            .unwrap(),
        "default serialization"
    );
}

#[test]
fn system_macro_registers_and_runs_static_system() {
    MACRO_SYSTEM_ENTITIES.lock().unwrap().clear();

    let mut scene = Scene::new();
    let counter = scene.add_entity();
    scene
        .add_component(counter, "MacroCounter".to_owned())
        .unwrap();

    let marker = scene.add_entity();
    scene
        .add_component(marker, "MacroMarker".to_owned())
        .unwrap();

    let both = scene.add_entity();
    scene
        .add_component(both, "MacroCounter".to_owned())
        .unwrap();
    scene.add_component(both, "MacroMarker".to_owned()).unwrap();

    scene
        .add_system("macro_group_counter".to_owned(), 1)
        .unwrap();

    scene.tick();

    let entities = MACRO_SYSTEM_ENTITIES.lock().unwrap();
    assert_eq!(entities.len(), 2);
    assert_eq!(
        entities.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(entities[0].contains(&counter));
    assert!(entities[0].contains(&both));
    assert_eq!(entities[1], vec![marker]);
}

#[test]
fn system_macro_allows_empty_entities() {
    *MACRO_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = None;
    *MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = None;

    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroCounter".to_owned())
        .unwrap();

    scene
        .add_system("macro_empty_system".to_owned(), 1)
        .unwrap();
    scene
        .add_system("macro_explicit_empty_system".to_owned(), 1)
        .unwrap();

    scene.tick();

    let entities = MACRO_EMPTY_SYSTEM_ENTITIES.lock().unwrap();
    assert_eq!(entities.as_ref(), Some(&Vec::new()));
    let entities = MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES.lock().unwrap();
    assert_eq!(entities.as_ref(), Some(&Vec::new()));
}

#[test]
fn scene_has_component_reports_component_presence() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert!(!scene.has_component(entity, "MacroCounter"));

    scene
        .add_component(entity, "MacroCounter".to_owned())
        .unwrap();

    assert!(scene.has_component(entity, "MacroCounter"));
    assert!(!scene.has_component(entity, "MacroMarker"));
}
