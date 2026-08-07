use std::{
    ffi::c_void,
    sync::{LazyLock, Mutex},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasserxr::{
    attacher, component, component_creator, detacher,
    scene::{
        Scene, SceneError,
        component::{
            ComponentError, FieldType, SerializedBytes, WXRComponentDescriptor,
            WXRComponentFieldDescriptor,
        },
        plugin::{Version, WXRPluginDescriptor},
        system::{WXRSystemDescriptor, WXRSystemEntityGroupDescriptor},
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

    #[getter]
    #[mutable]
    enabled: bool,

    #[none]
    #[allow(dead_code)]
    hidden: i32,
}

#[component_creator(MacroComponent)]
pub fn create_macro_component(scene: &mut Scene) -> Option<MacroComponent> {
    let _ = scene.get_plugins();
    Some(MacroComponent::default())
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

#[component_creator(MacroRoundTripComponent)]
pub fn create_macro_round_trip_component(_scene: &mut Scene) -> Option<MacroRoundTripComponent> {
    Some(MacroRoundTripComponent::default())
}

#[component]
#[derive(Default)]
pub struct MacroCounter {
    #[allow(dead_code)]
    value: i64,
}

#[component_creator(MacroCounter)]
pub fn create_macro_counter(_scene: &mut Scene) -> Option<MacroCounter> {
    Some(MacroCounter::default())
}

#[component]
#[derive(Default)]
pub struct MacroMarker {
    #[allow(dead_code)]
    value: i64,
}

#[component_creator(MacroMarker)]
pub fn create_macro_marker(_scene: &mut Scene) -> Option<MacroMarker> {
    Some(MacroMarker::default())
}

#[derive(Default, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MacroOwnedValue {
    value: String,
}

#[component]
#[derive(Default)]
pub struct MacroOwnershipComponent {
    #[mutable]
    value: MacroOwnedValue,
}

#[component_creator(MacroOwnershipComponent)]
pub fn create_macro_ownership_component(_scene: &mut Scene) -> Option<MacroOwnershipComponent> {
    Some(MacroOwnershipComponent::default())
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

#[component_creator(MacroCustomHooksComponent)]
pub fn create_macro_custom_hooks_component(
    _scene: &mut Scene,
) -> Option<MacroCustomHooksComponent> {
    Some(MacroCustomHooksComponent::default())
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
#[derive(Default)]
pub struct MacroPosition {
    #[mutable]
    position: [f32; 3],
}

#[component_creator(MacroPosition)]
pub fn create_macro_position(_scene: &mut Scene) -> Option<MacroPosition> {
    Some(MacroPosition::default())
}

#[component]
#[derive(Default)]
pub struct MacroFailingComponent {
    #[allow(dead_code)]
    value: i32,
}

#[component_creator(MacroFailingComponent)]
pub fn create_macro_failing_component(_scene: &mut Scene) -> Option<MacroFailingComponent> {
    None
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

#[system]
pub fn macro_group_counter(_scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    *MACRO_SYSTEM_ENTITIES.lock().unwrap() = entities;
}

#[system]
pub fn macro_empty_system(_scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    *MACRO_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = Some(entities);
}

#[system]
pub fn macro_explicit_empty_system(_scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    *MACRO_EXPLICIT_EMPTY_SYSTEM_ENTITIES.lock().unwrap() = Some(entities);
}

#[system]
pub fn macro_lifecycle_system(_scene: &mut Scene, _delta: f32, _entities: Vec<Vec<Uuid>>) {}

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

macro_rules! component_field {
    ($name:literal, $type:ident, $getter:expr, $mutable:expr, $serializer:expr, $deserializer:expr) => {
        WXRComponentFieldDescriptor {
            name: concat!($name, "\0").as_ptr().cast(),
            field_type: FieldType::$type as u32,
            getter: $getter,
            mutable: $mutable,
            serializer: $serializer,
            deserializer: $deserializer,
        }
    };
}

static MACRO_COMPONENT_FIELDS: [WXRComponentFieldDescriptor; 5] = [
    component_field!(
        "my_int",
        I32,
        Some(wxr_get_MacroComponent_my_int),
        0,
        None,
        None
    ),
    component_field!(
        "default_int",
        I32,
        Some(wxr_get_MacroComponent_default_int),
        1,
        Some(wxr_serialize_MacroComponent_default_int),
        Some(wxr_deserialize_MacroComponent_default_int)
    ),
    component_field!(
        "my_string",
        String,
        Some(wxr_get_MacroComponent_my_string),
        1,
        Some(wxr_serialize_MacroComponent_my_string),
        Some(wxr_deserialize_MacroComponent_my_string)
    ),
    component_field!(
        "default_string",
        String,
        Some(wxr_get_MacroComponent_default_string),
        1,
        Some(wxr_serialize_MacroComponent_default_string),
        Some(wxr_deserialize_MacroComponent_default_string)
    ),
    component_field!(
        "enabled",
        Boolean,
        Some(wxr_get_MacroComponent_enabled),
        1,
        None,
        None
    ),
];
static ROUND_TRIP_FIELDS: [WXRComponentFieldDescriptor; 2] = [
    component_field!(
        "health",
        I32,
        Some(wxr_get_MacroRoundTripComponent_health),
        1,
        Some(wxr_serialize_MacroRoundTripComponent_health),
        Some(wxr_deserialize_MacroRoundTripComponent_health)
    ),
    component_field!(
        "label",
        String,
        Some(wxr_get_MacroRoundTripComponent_label),
        1,
        None,
        None
    ),
];
static COUNTER_FIELDS: [WXRComponentFieldDescriptor; 1] = [component_field!(
    "value",
    I64,
    Some(wxr_get_MacroCounter_value),
    0,
    Some(wxr_serialize_MacroCounter_value),
    Some(wxr_deserialize_MacroCounter_value)
)];
static MARKER_FIELDS: [WXRComponentFieldDescriptor; 1] = [component_field!(
    "value",
    I64,
    Some(wxr_get_MacroMarker_value),
    0,
    Some(wxr_serialize_MacroMarker_value),
    Some(wxr_deserialize_MacroMarker_value)
)];
static OWNERSHIP_FIELDS: [WXRComponentFieldDescriptor; 1] = [component_field!(
    "value",
    Blob,
    Some(wxr_get_MacroOwnershipComponent_value),
    1,
    Some(wxr_serialize_MacroOwnershipComponent_value),
    Some(wxr_deserialize_MacroOwnershipComponent_value)
)];
static CUSTOM_FIELDS: [WXRComponentFieldDescriptor; 1] = [component_field!(
    "value",
    Usize,
    Some(macro_custom_value_getter),
    1,
    Some(macro_custom_value_serializer),
    Some(macro_custom_value_deserializer)
)];
static POSITION_FIELDS: [WXRComponentFieldDescriptor; 3] = [
    component_field!("x", F32, Some(macro_position_x), 1, None, None),
    component_field!("y", F32, Some(macro_position_y), 0, None, None),
    component_field!(
        "position",
        F32Vec3,
        Some(wxr_get_MacroPosition_position),
        1,
        Some(wxr_serialize_MacroPosition_position),
        Some(wxr_deserialize_MacroPosition_position)
    ),
];
static FAILING_FIELDS: [WXRComponentFieldDescriptor; 1] = [component_field!(
    "value",
    I32,
    Some(wxr_get_MacroFailingComponent_value),
    0,
    Some(wxr_serialize_MacroFailingComponent_value),
    Some(wxr_deserialize_MacroFailingComponent_value)
)];

macro_rules! component_descriptor {
    ($name:literal, $creator:ident, $destroyer:ident, $fields:ident) => {
        WXRComponentDescriptor {
            name: concat!($name, "\0").as_ptr().cast(),
            creator: Some($creator),
            destroyer: Some($destroyer),
            fields: $fields.as_ptr(),
            field_count: $fields.len(),
            methods: std::ptr::null(),
            method_count: 0,
        }
    };
}

static COMPONENTS: [WXRComponentDescriptor; 8] = [
    component_descriptor!(
        "MacroComponent",
        wxr_create_MacroComponent,
        wxr_destroy_MacroComponent,
        MACRO_COMPONENT_FIELDS
    ),
    component_descriptor!(
        "MacroRoundTripComponent",
        wxr_create_MacroRoundTripComponent,
        wxr_destroy_MacroRoundTripComponent,
        ROUND_TRIP_FIELDS
    ),
    component_descriptor!(
        "MacroCounter",
        wxr_create_MacroCounter,
        wxr_destroy_MacroCounter,
        COUNTER_FIELDS
    ),
    component_descriptor!(
        "MacroMarker",
        wxr_create_MacroMarker,
        wxr_destroy_MacroMarker,
        MARKER_FIELDS
    ),
    component_descriptor!(
        "MacroOwnershipComponent",
        wxr_create_MacroOwnershipComponent,
        wxr_destroy_MacroOwnershipComponent,
        OWNERSHIP_FIELDS
    ),
    component_descriptor!(
        "MacroCustomHooksComponent",
        wxr_create_MacroCustomHooksComponent,
        wxr_destroy_MacroCustomHooksComponent,
        CUSTOM_FIELDS
    ),
    component_descriptor!(
        "MacroPosition",
        wxr_create_MacroPosition,
        wxr_destroy_MacroPosition,
        POSITION_FIELDS
    ),
    component_descriptor!(
        "MacroFailingComponent",
        wxr_create_MacroFailingComponent,
        wxr_destroy_MacroFailingComponent,
        FAILING_FIELDS
    ),
];

const COUNTER_GROUP_COMPONENTS: [*const std::ffi::c_char; 1] = [c"MacroCounter".as_ptr()];
const MARKER_GROUP_COMPONENTS: [*const std::ffi::c_char; 1] = [c"MacroMarker".as_ptr()];
static COUNTER_GROUPS: [WXRSystemEntityGroupDescriptor; 2] = [
    WXRSystemEntityGroupDescriptor {
        components: COUNTER_GROUP_COMPONENTS.as_ptr(),
        component_count: COUNTER_GROUP_COMPONENTS.len(),
    },
    WXRSystemEntityGroupDescriptor {
        components: MARKER_GROUP_COMPONENTS.as_ptr(),
        component_count: MARKER_GROUP_COMPONENTS.len(),
    },
];
static EMPTY_GROUPS: [WXRSystemEntityGroupDescriptor; 1] = [WXRSystemEntityGroupDescriptor {
    components: std::ptr::null(),
    component_count: 0,
}];
static SYSTEMS: [WXRSystemDescriptor; 4] = [
    WXRSystemDescriptor {
        name: c"macro_group_counter".as_ptr(),
        runner: Some(wxr_system_macro_group_counter),
        attach: None,
        detach: None,
        entity_groups: COUNTER_GROUPS.as_ptr(),
        entity_group_count: COUNTER_GROUPS.len(),
    },
    WXRSystemDescriptor {
        name: c"macro_empty_system".as_ptr(),
        runner: Some(wxr_system_macro_empty_system),
        attach: None,
        detach: None,
        entity_groups: std::ptr::null(),
        entity_group_count: 0,
    },
    WXRSystemDescriptor {
        name: c"macro_lifecycle_system".as_ptr(),
        runner: Some(wxr_system_macro_lifecycle_system),
        attach: Some(wxr_attach_macro_lifecycle_system),
        detach: Some(wxr_detach_macro_lifecycle_system),
        entity_groups: std::ptr::null(),
        entity_group_count: 0,
    },
    WXRSystemDescriptor {
        name: c"macro_explicit_empty_system".as_ptr(),
        runner: Some(wxr_system_macro_explicit_empty_system),
        attach: None,
        detach: None,
        entity_groups: EMPTY_GROUPS.as_ptr(),
        entity_group_count: EMPTY_GROUPS.len(),
    },
];
static PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"macro-tests".as_ptr(),
    components: COMPONENTS.as_ptr(),
    component_count: COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: SYSTEMS.as_ptr(),
    system_count: SYSTEMS.len(),
};

fn test_scene() -> Scene {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&PLUGIN) }.unwrap();
    scene
}

#[test]
fn component_macro_round_trip_keeps_component_behavior() {
    let mut scene = test_scene();
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
    let mut loaded = test_scene();
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
    let mut scene = test_scene();
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
        Err(SceneError::Component(ComponentError::FieldNotMutable))
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
        Err(SceneError::Component(ComponentError::FieldNotFound))
    );
    assert!(matches!(
        scene.query_mut::<(&mut i32,)>(entity, "MacroComponent", &["hidden"]),
        Err(SceneError::Component(ComponentError::FieldNotFound))
    ));
}

#[test]
fn component_macro_mutates_non_clone_field() {
    let mut scene = test_scene();
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
    let mut scene = test_scene();
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
    let mut loaded = test_scene();
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
fn component_macro_reports_failed_creator() {
    let mut scene = test_scene();
    let entity = scene.add_entity();

    assert_eq!(
        scene.add_component(entity, "MacroFailingComponent".to_owned()),
        Err(SceneError::Component(ComponentError::CreatorFailed))
    );
}

#[test]
fn component_macro_registers_custom_field_functions() {
    let mut scene = test_scene();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroCustomHooksComponent".to_owned())
        .unwrap();

    let (value,) = scene
        .query_mut::<(&mut usize,)>(entity, "MacroCustomHooksComponent", &["value"])
        .unwrap();
    *value = 12;

    let serialized = scene.serialize().unwrap();
    let mut loaded = test_scene();
    loaded.deserialize(&serialized).unwrap();

    let (value,) = loaded
        .query::<(&usize,)>(entity, "MacroCustomHooksComponent", &["value"])
        .unwrap();
    assert_eq!(*value, 100);
}

#[test]
fn manifest_can_register_structural_fields_with_custom_getters() {
    let mut scene = test_scene();
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
        Err(SceneError::Component(ComponentError::FieldNotMutable))
    );

    let serialized = scene.serialize().unwrap();
    let mut loaded = test_scene();
    loaded.deserialize(&serialized).unwrap();
    let (x,) = loaded
        .query::<(&f32,)>(entity, "MacroPosition", &["x"])
        .unwrap();
    assert_eq!(*x, 7.5);
}

#[test]
fn component_macro_registers_exact_field_types() {
    let mut scene = test_scene();
    let component_entity = scene.add_entity();
    scene
        .add_component(component_entity, "MacroComponent".to_owned())
        .unwrap();
    let hooks_entity = scene.add_entity();
    scene
        .add_component(hooks_entity, "MacroCustomHooksComponent".to_owned())
        .unwrap();
    let position_entity = scene.add_entity();
    scene
        .add_component(position_entity, "MacroPosition".to_owned())
        .unwrap();

    assert_eq!(
        scene
            .get_component_field_type(component_entity, "MacroComponent", "my_int")
            .unwrap(),
        FieldType::I32
    );
    assert_eq!(
        scene
            .get_component_field_type(component_entity, "MacroComponent", "my_string")
            .unwrap(),
        FieldType::String
    );
    assert_eq!(
        scene
            .get_component_field_type(component_entity, "MacroComponent", "enabled")
            .unwrap(),
        FieldType::Boolean
    );
    assert_eq!(
        scene
            .get_component_field_type(hooks_entity, "MacroCustomHooksComponent", "value")
            .unwrap(),
        FieldType::Usize
    );
    assert_eq!(
        scene
            .get_component_field_type(position_entity, "MacroPosition", "x")
            .unwrap(),
        FieldType::F32
    );
    assert_eq!(
        scene
            .get_component_field_type(position_entity, "MacroPosition", "position")
            .unwrap(),
        FieldType::F32Vec3
    );
}

#[test]
fn component_macro_renders_and_parses_boolean_field() {
    let mut scene = test_scene();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    assert_eq!(
        scene
            .render_field(entity, "MacroComponent", "enabled")
            .unwrap(),
        "false"
    );

    scene
        .parse_field(entity, "MacroComponent", "enabled", "tRuE")
        .unwrap();

    let (enabled,) = scene
        .query::<(&bool,)>(entity, "MacroComponent", &["enabled"])
        .unwrap();
    assert!(*enabled);
    assert_eq!(
        scene
            .render_field(entity, "MacroComponent", "enabled")
            .unwrap(),
        "true"
    );
}

#[test]
fn component_macro_parses_mutable_vector_field() {
    let mut scene = test_scene();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MacroPosition".to_owned())
        .unwrap();

    scene
        .parse_field(entity, "MacroPosition", "position", "1.0,2.0,3.0")
        .unwrap();

    let (position,) = scene
        .query::<(&[f32; 3],)>(entity, "MacroPosition", &["position"])
        .unwrap();
    assert_eq!(*position, [1.0, 2.0, 3.0]);

    scene
        .parse_field(entity, "MacroPosition", "position", "4.0, 5.0, 6.0")
        .unwrap();

    let (position,) = scene
        .query::<(&[f32; 3],)>(entity, "MacroPosition", &["position"])
        .unwrap();
    assert_eq!(*position, [4.0, 5.0, 6.0]);
}

#[test]
fn system_macro_registers_and_runs_static_system() {
    MACRO_SYSTEM_ENTITIES.lock().unwrap().clear();

    let mut scene = test_scene();
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
        vec![2, 2]
    );
    assert!(entities[0].contains(&counter));
    assert!(entities[0].contains(&both));
    assert!(entities[1].contains(&marker));
    assert!(entities[1].contains(&both));
}

#[test]
fn attacher_and_detacher_macros_run_system_lifecycle_hooks() {
    *MACRO_ATTACH_ENTITY.lock().unwrap() = None;
    *MACRO_DETACH_ENTITY.lock().unwrap() = None;

    let mut scene = test_scene();

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

    let mut scene = test_scene();
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
    assert_eq!(entities.as_ref().map(Vec::len), Some(1));
    assert_eq!(entities.as_ref().unwrap()[0], [entity]);
}

#[test]
fn scene_has_component_reports_component_presence() {
    let mut scene = test_scene();
    let entity = scene.add_entity();

    assert!(!scene.has_component(entity, "MacroCounter"));

    scene
        .add_component(entity, "MacroCounter".to_owned())
        .unwrap();

    assert!(scene.has_component(entity, "MacroCounter"));
    assert!(!scene.has_component(entity, "MacroMarker"));
}
