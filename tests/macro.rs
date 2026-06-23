use std::{
    ffi::c_void,
    sync::{LazyLock, Mutex},
};

use uuid::Uuid;
use wasserxr::{
    attacher, component, detacher,
    error::{ComponentError, SceneError},
    scene::{
        Scene,
        component::{FieldType, Schema, SerializedBytes},
    },
    system,
};

#[component]
#[derive(Default)]
pub struct MacroComponent {
    #[getter]
    my_int: i32,

    #[mutable]
    default_int: i32,

    #[getter]
    #[mutable]
    #[serializer]
    #[deserializer]
    my_string: String,

    #[mutable]
    default_string: String,

    #[none]
    #[allow(dead_code)]
    hidden: i32,
}

#[component]
#[derive(Default)]
pub struct MacroRoundTripComponent {
    #[getter]
    #[mutable]
    #[serializer]
    #[deserializer]
    health: i32,

    #[getter]
    #[mutable]
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

#[derive(Default, Debug, PartialEq, Eq)]
pub struct MacroOwnedValue {
    value: String,
}

#[component]
#[derive(Default)]
pub struct MacroOwnershipComponent {
    #[mutable]
    value: MacroOwnedValue,
}

#[component(no_schema)]
#[derive(Default)]
pub struct MacroManualSchemaComponent {
    #[none]
    value: i32,
}

#[component(no_schema)]
#[derive(Default)]
pub struct MacroNoSchemaComponent {
    #[allow(dead_code)]
    value: i32,
}

unsafe extern "C" fn macro_manual_schema_value_getter(ptr: *mut c_void) -> *mut c_void {
    unsafe { &mut (*(ptr as *mut MacroManualSchemaComponent)).value as *mut i32 as *mut c_void }
}

#[unsafe(export_name = "wxr_schema_MacroManualSchemaComponent")]
#[allow(non_snake_case)]
pub unsafe extern "C" fn wxr_schema_MacroManualSchemaComponent(schema: *mut Schema) {
    unsafe {
        (*schema).add_field(
            "value".to_owned(),
            FieldType::Long,
            Some(macro_manual_schema_value_getter),
            true,
            None,
            None,
        );
    }
}

#[component]
#[derive(Default)]
pub struct MacroCustomHooksComponent {
    #[getter(macro_custom_value_getter)]
    #[mutable]
    #[serializer(macro_custom_value_serializer)]
    #[deserializer(macro_custom_value_deserializer)]
    value: usize,
}

unsafe extern "C" fn macro_custom_value_getter(ptr: *mut c_void) -> *mut c_void {
    unsafe { &mut (*(ptr as *mut MacroCustomHooksComponent)).value as *mut usize as *mut c_void }
}

unsafe extern "C" fn macro_custom_value_serializer(_ptr: *const c_void) -> SerializedBytes {
    SerializedBytes::from_vec(99usize.to_le_bytes().to_vec())
}

unsafe extern "C" fn macro_custom_value_deserializer(ptr: *mut c_void, data: SerializedBytes) {
    let bytes = unsafe { data.into_vec() };
    if let Ok(bytes) = <[u8; std::mem::size_of::<usize>()]>::try_from(bytes.as_slice()) {
        unsafe {
            (*(ptr as *mut MacroCustomHooksComponent)).value = usize::from_le_bytes(bytes) + 1;
        }
    }
}

#[component]
#[virtual_field(x: f32, getter = macro_position_x, mutable)]
#[virtual_field(y: f32, getter = macro_position_y)]
#[derive(Default)]
pub struct MacroPosition {
    position: [f32; 3],
}

unsafe extern "C" fn macro_position_x(ptr: *mut c_void) -> *mut c_void {
    unsafe { &mut (*(ptr as *mut MacroPosition)).position[0] as *mut f32 as *mut c_void }
}

unsafe extern "C" fn macro_position_y(ptr: *mut c_void) -> *mut c_void {
    unsafe { &mut (*(ptr as *mut MacroPosition)).position[1] as *mut f32 as *mut c_void }
}

static MACRO_SYSTEM_ENTITIES: LazyLock<Mutex<Vec<Vec<Uuid>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static MACRO_EMPTY_SYSTEM_ENTITIES: LazyLock<Mutex<Option<Vec<Vec<Uuid>>>>> =
    LazyLock::new(|| Mutex::new(None));
static MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES: LazyLock<Mutex<Option<Vec<Vec<Uuid>>>>> =
    LazyLock::new(|| Mutex::new(None));
static MACRO_ATTACH_ENTITY: LazyLock<Mutex<Option<Uuid>>> = LazyLock::new(|| Mutex::new(None));
static MACRO_DETACH_ENTITY: LazyLock<Mutex<Option<Uuid>>> = LazyLock::new(|| Mutex::new(None));

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

#[system]
pub fn macro_lifecycle_system(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {}

#[attacher(macro_lifecycle_system)]
pub fn attach_macro_lifecycle_system(scene: &mut Scene) {
    let entity = scene.add_entity();
    scene
        .set_entity_name(entity, "attached".to_owned())
        .unwrap();
    *MACRO_ATTACH_ENTITY.lock().unwrap() = Some(entity);
}

#[detacher(macro_lifecycle_system)]
pub fn detach_macro_lifecycle_system(scene: &mut Scene) {
    let entity = scene.add_entity();
    scene
        .set_entity_name(entity, "detached".to_owned())
        .unwrap();
    *MACRO_DETACH_ENTITY.lock().unwrap() = Some(entity);
}

#[test]
fn component_macro_round_trip_keeps_component_behavior() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroRoundTripComponent".to_owned())
        .unwrap();

    {
        let (health, label) = scene
            .query_mut::<(&mut i32, &mut String)>(
                entity,
                "MacroRoundTripComponent",
                &["health", "label"],
            )
            .unwrap();
        *health = 24;
        *label = "round trip".to_owned();
    }

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    assert!(loaded.has_component(entity, "MacroRoundTripComponent"));
    let (health, label) = loaded
        .query::<(&i32, &String)>(entity, "MacroRoundTripComponent", &["health", "label"])
        .unwrap();
    assert_eq!(*health, 24);
    assert_eq!(label, "");

    let (health,) = loaded
        .query_mut::<(&mut i32,)>(entity, "MacroRoundTripComponent", &["health"])
        .unwrap();
    *health = 30;
    let (health,) = loaded
        .query::<(&i32,)>(entity, "MacroRoundTripComponent", &["health"])
        .unwrap();
    assert_eq!(*health, 30);
}

#[test]
fn component_macro_registers_and_accesses_static_component() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    let (my_int, my_string, default_int) = scene
        .query::<(&i32, &String, &i32)>(
            entity,
            "MacroComponent",
            &["my_int", "my_string", "default_int"],
        )
        .unwrap();
    assert_eq!(*my_int, 0);
    assert_eq!(my_string, "");
    assert_eq!(*default_int, 0);

    assert_eq!(
        scene.query_mut::<(&mut i32,)>(entity, "MacroComponent", &["my_int"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotMutable
        ))
    );

    let (my_string, default_int) = scene
        .query_mut::<(&mut String, &mut i32)>(
            entity,
            "MacroComponent",
            &["my_string", "default_int"],
        )
        .unwrap();
    *my_string = "updated through query_mut".to_owned();
    *default_int = 42;

    let (my_string, default_int) = scene
        .query::<(&String, &i32)>(entity, "MacroComponent", &["my_string", "default_int"])
        .unwrap();
    assert_eq!(my_string, "updated through query_mut");
    assert_eq!(*default_int, 42);
    assert_eq!(
        scene.query::<(&i32,)>(entity, "MacroComponent", &["hidden"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoGetter
        ))
    );
    assert!(matches!(
        scene.query_mut::<(&mut i32,)>(entity, "MacroComponent", &["hidden"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotMutable
        ))
    ));
}

#[test]
fn component_macro_mutates_non_clone_field() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene
        .add_component(entity, "MacroOwnershipComponent".to_owned())
        .unwrap();

    scene
        .query_mut::<(&mut MacroOwnedValue,)>(entity, "MacroOwnershipComponent", &["value"])
        .unwrap()
        .0
        .value
        .push_str("owned");
    let (value,) = scene
        .query::<(&MacroOwnedValue,)>(entity, "MacroOwnershipComponent", &["value"])
        .unwrap();

    assert_eq!(
        value,
        &MacroOwnedValue {
            value: "owned".to_owned(),
        }
    );
}

#[test]
fn component_macro_serializes_static_component_fields() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    {
        let (my_string, default_int, default_string) = scene
            .query_mut::<(&mut String, &mut i32, &mut String)>(
                entity,
                "MacroComponent",
                &["my_string", "default_int", "default_string"],
            )
            .unwrap();
        *my_string = "serialized through macro".to_owned();
        *default_int = 12;
        *default_string = "default serialization".to_owned();
    }

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    let (my_int, default_int, my_string, default_string) = loaded
        .query::<(&i32, &i32, &String, &String)>(
            entity,
            "MacroComponent",
            &["my_int", "default_int", "my_string", "default_string"],
        )
        .unwrap();
    assert_eq!(*my_int, 0);
    assert_eq!(*default_int, 12);
    assert_eq!(my_string, "serialized through macro");
    assert_eq!(default_string, "default serialization");
}

#[test]
fn component_macro_allows_custom_schema_function() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroManualSchemaComponent".to_owned())
        .unwrap();

    let (value,) = scene
        .query_mut::<(&mut i32,)>(entity, "MacroManualSchemaComponent", &["value"])
        .unwrap();
    *value = 64;

    let (value,) = scene
        .query::<(&i32,)>(entity, "MacroManualSchemaComponent", &["value"])
        .unwrap();
    assert_eq!(*value, 64);
}

#[test]
fn component_macro_allows_missing_schema_function() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroNoSchemaComponent".to_owned())
        .unwrap();

    assert_eq!(
        scene.query::<(&i32,)>(entity, "MacroNoSchemaComponent", &["value"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotFound
        ))
    );
}

#[test]
fn component_macro_registers_custom_field_functions() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroCustomHooksComponent".to_owned())
        .unwrap();

    let (value,) = scene
        .query_mut::<(&mut usize,)>(entity, "MacroCustomHooksComponent", &["value"])
        .unwrap();
    *value = 12;

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    let (value,) = loaded
        .query::<(&usize,)>(entity, "MacroCustomHooksComponent", &["value"])
        .unwrap();
    assert_eq!(*value, 100);
}

#[test]
fn component_macro_registers_virtual_fields() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroPosition".to_owned())
        .unwrap();

    let (x,) = scene
        .query_mut::<(&mut f32,)>(entity, "MacroPosition", &["x"])
        .unwrap();
    *x = 7.5;

    let (x, y, position) = scene
        .query::<(&f32, &f32, &[f32; 3])>(entity, "MacroPosition", &["x", "y", "position"])
        .unwrap();
    assert_eq!(*x, 7.5);
    assert_eq!(*y, 0.0);
    assert_eq!(position[0], 7.5);

    assert_eq!(
        scene.query_mut::<(&mut f32,)>(entity, "MacroPosition", &["y"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotMutable
        ))
    );

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();
    let (x,) = loaded
        .query::<(&f32,)>(entity, "MacroPosition", &["x"])
        .unwrap();
    assert_eq!(*x, 0.0);
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
fn attacher_and_detacher_macros_run_system_lifecycle_hooks() {
    *MACRO_ATTACH_ENTITY.lock().unwrap() = None;
    *MACRO_DETACH_ENTITY.lock().unwrap() = None;

    let mut scene = Scene::new();

    scene
        .add_system("macro_lifecycle_system".to_owned(), 1)
        .unwrap();

    let attach_entity = MACRO_ATTACH_ENTITY.lock().unwrap().unwrap();
    assert_eq!(scene.get_entity_name(attach_entity), Ok("attached"));
    assert_eq!(*MACRO_DETACH_ENTITY.lock().unwrap(), None);

    scene.remove_system("macro_lifecycle_system").unwrap();

    let detach_entity = MACRO_DETACH_ENTITY.lock().unwrap().unwrap();
    assert_eq!(scene.get_entity_name(detach_entity), Ok("detached"));
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
