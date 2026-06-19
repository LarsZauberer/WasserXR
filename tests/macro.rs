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

    #[getter]
    #[setter]
    my_string: String,

    #[allow(dead_code)]
    hidden: i32,
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
static MACRO_SYSTEM_GROUPS: LazyLock<Mutex<Vec<usize>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[system(entities = [["MacroCounter"], ["MacroMarker"]])]
pub fn macro_group_counter(_scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    *MACRO_SYSTEM_ENTITIES.lock().unwrap() = entities;
    *MACRO_SYSTEM_GROUPS.lock().unwrap() = groups;
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

    let updated = "updated through setter".to_owned();
    scene
        .set(entity, "MacroComponent", "my_string", &updated)
        .unwrap();

    assert_eq!(
        scene
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        "updated through setter"
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
        loaded
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        "serialized through macro"
    );
}

#[test]
fn system_macro_registers_and_runs_static_system() {
    MACRO_SYSTEM_ENTITIES.lock().unwrap().clear();
    MACRO_SYSTEM_GROUPS.lock().unwrap().clear();

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

    assert!(!scene.has_component(entity, "MacroCounter"));

    scene
        .add_component(entity, "MacroCounter".to_owned())
        .unwrap();

    assert!(scene.has_component(entity, "MacroCounter"));
    assert!(!scene.has_component(entity, "MacroMarker"));
}
