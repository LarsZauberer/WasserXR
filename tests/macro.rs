//! Integration coverage for the `#[component]`, `#[system]`, `#[attacher]`, and
//! `#[detacher]` macros. Every component/system under test is declared in the
//! shared fixture with the real macros; these tests assert the behavior the
//! macros must produce.

#[path = "../src/testing_plugin_fixture.rs"]
mod testing_plugin_fixture;

use rstest::rstest;
use testing_plugin_fixture::{self as fx, scene_with};
use wasserxr::{
    error::{ComponentError, SceneError},
    scene::{Scene, component::FieldType},
};

#[rstest]
fn component_macro_generates_working_getters_and_mutability() {
    let (mut scene, entity) = scene_with("Featured");

    // `reads_only` is immutable; `reads_writes` and `text` are mutable.
    assert_eq!(
        scene.query_mut::<(&mut i32,)>(entity, "Featured", &["reads_only"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotMutable
        ))
    );

    let (reads_writes, text) = scene
        .query_mut::<(&mut i32, &mut String)>(entity, "Featured", &["reads_writes", "text"])
        .unwrap();
    *reads_writes = 42;
    *text = "hello".to_owned();

    let (reads_only, reads_writes, text) = scene
        .query::<(&i32, &i32, &String)>(entity, "Featured", &["reads_only", "reads_writes", "text"])
        .unwrap();
    assert_eq!(
        (*reads_only, *reads_writes, text.as_str()),
        (0, 42, "hello")
    );

    // A `#[none]` field is not exposed as a gettable field.
    assert_eq!(
        scene.query::<(&i32,)>(entity, "Featured", &["hidden"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoGetter
        ))
    );
}

#[rstest]
fn component_macro_serializes_only_serializable_fields() {
    let (mut scene, entity) = scene_with("Featured");
    {
        let (reads_writes, text) = scene
            .query_mut::<(&mut i32, &mut String)>(entity, "Featured", &["reads_writes", "text"])
            .unwrap();
        *reads_writes = 12;
        *text = "persisted".to_owned();
    }

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    // `reads_writes` (generated serializer) and `text` (explicit serializer)
    // survive; the getter-only `enabled` is left at its default.
    let (reads_writes, text, enabled) = loaded
        .query::<(&i32, &String, &bool)>(entity, "Featured", &["reads_writes", "text", "enabled"])
        .unwrap();
    assert_eq!(
        (*reads_writes, text.as_str(), *enabled),
        (12, "persisted", false)
    );
}

#[rstest]
fn component_macro_mutates_non_clone_field_in_place() {
    let (mut scene, entity) = scene_with("Owner");

    scene
        .query_mut::<(&mut fx::OwnedValue,)>(entity, "Owner", &["writable"])
        .unwrap()
        .0
        .value
        .push_str("owned");

    let (value,) = scene
        .query::<(&fx::OwnedValue,)>(entity, "Owner", &["writable"])
        .unwrap();
    assert_eq!(value.value, "owned");
}

#[rstest]
fn component_macro_supports_manual_and_missing_schema() {
    // `ManualSchema` supplies its schema through a hand-written function.
    let (mut scene, entity) = scene_with("ManualSchema");
    let (value,) = scene
        .query_mut::<(&mut i32,)>(entity, "ManualSchema", &["value"])
        .unwrap();
    *value = 64;
    let (value,) = scene
        .query::<(&i32,)>(entity, "ManualSchema", &["value"])
        .unwrap();
    assert_eq!(*value, 64);

    // `NoSchema` registers nothing, so every field lookup misses.
    let (scene, entity) = scene_with("NoSchema");
    assert_eq!(
        scene.query::<(&i32,)>(entity, "NoSchema", &["value"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotFound
        ))
    );
}

#[rstest]
fn component_macro_reports_failed_creator() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert_eq!(
        scene.add_component(entity, "FailingCreator".to_owned()),
        Err(SceneError::ComponentCreatorFailed)
    );
}

#[rstest]
fn component_macro_registers_custom_field_functions() {
    let (mut scene, entity) = scene_with("CustomHooks");
    {
        let (value,) = scene
            .query_mut::<(&mut usize,)>(entity, "CustomHooks", &["value"])
            .unwrap();
        *value = 12;
    }

    let serialized = scene.serialize().unwrap();
    let mut loaded = Scene::new();
    loaded.deserialize(&serialized).unwrap();

    // The custom serializer always writes 99; the custom deserializer adds 1.
    let (value,) = loaded
        .query::<(&usize,)>(entity, "CustomHooks", &["value"])
        .unwrap();
    assert_eq!(*value, 100);
}

#[rstest]
fn component_macro_registers_virtual_fields_and_parses_vectors() {
    let (mut scene, entity) = scene_with("Position");

    // Virtual field `x` is mutable and aliases `position[0]`; `y` is read-only.
    {
        let (x,) = scene
            .query_mut::<(&mut f32,)>(entity, "Position", &["x"])
            .unwrap();
        *x = 7.5;
    }
    let (x, y, position) = scene
        .query::<(&f32, &f32, &[f32; 3])>(entity, "Position", &["x", "y", "position"])
        .unwrap();
    assert_eq!((*x, *y, position[0]), (7.5, 0.0, 7.5));
    assert_eq!(
        scene.query_mut::<(&mut f32,)>(entity, "Position", &["y"]),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNotMutable
        ))
    );

    // The vector field parses from text.
    scene
        .parse_field(entity, "Position", "position", "1.0, 2.0, 3.0")
        .unwrap();
    let (position,) = scene
        .query::<(&[f32; 3],)>(entity, "Position", &["position"])
        .unwrap();
    assert_eq!(*position, [1.0, 2.0, 3.0]);
}

#[rstest]
#[case("Featured", "reads_only", FieldType::I32)]
#[case("Featured", "text", FieldType::String)]
#[case("Featured", "enabled", FieldType::Boolean)]
#[case("CustomHooks", "value", FieldType::Usize)]
#[case("Position", "x", FieldType::F32)]
#[case("Position", "position", FieldType::F32Vec3)]
fn component_macro_registers_exact_field_types(
    #[case] component: &str,
    #[case] field: &str,
    #[case] expected: FieldType,
) {
    let (scene, entity) = scene_with(component);

    assert_eq!(
        scene
            .get_component_field_type(entity, component, field)
            .unwrap(),
        expected
    );
}

#[rstest]
fn component_macro_renders_and_parses_boolean_field() {
    let (mut scene, entity) = scene_with("Featured");

    assert_eq!(
        scene.render_field(entity, "Featured", "enabled").unwrap(),
        "false"
    );
    scene
        .parse_field(entity, "Featured", "enabled", "tRuE")
        .unwrap();

    let (enabled,) = scene
        .query::<(&bool,)>(entity, "Featured", &["enabled"])
        .unwrap();
    assert!(*enabled);
    assert_eq!(
        scene.render_field(entity, "Featured", "enabled").unwrap(),
        "true"
    );
}

#[rstest]
fn scene_has_component_reports_presence() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert!(!scene.has_component(entity, "Counter"));
    scene.add_component(entity, "Counter".to_owned()).unwrap();
    assert!(scene.has_component(entity, "Counter"));
    assert!(!scene.has_component(entity, "Pair"));
}

#[rstest]
fn system_macro_selects_entities_by_group() {
    fx::GROUP_ENTITIES.lock().unwrap().clear();

    let mut scene = Scene::new();
    let counter = scene.add_entity();
    scene.add_component(counter, "Counter".to_owned()).unwrap();
    let owner = scene.add_entity();
    scene.add_component(owner, "Owner".to_owned()).unwrap();
    let both = scene.add_entity();
    scene.add_component(both, "Counter".to_owned()).unwrap();
    scene.add_component(both, "Owner".to_owned()).unwrap();

    scene.add_system("group_counter".to_owned(), 1).unwrap();
    scene.tick();

    let entities = fx::GROUP_ENTITIES.lock().unwrap();
    assert_eq!(entities.len(), 2);
    assert_eq!(
        entities.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(entities[0].contains(&counter));
    assert!(entities[0].contains(&both));
    assert_eq!(entities[1], vec![owner]);
}

#[rstest]
fn system_macro_allows_empty_entity_filter() {
    *fx::EMPTY_ENTITIES.lock().unwrap() = None;

    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene.add_component(entity, "Counter".to_owned()).unwrap();
    scene.add_system("empty_system".to_owned(), 1).unwrap();

    scene.tick();

    assert_eq!(
        fx::EMPTY_ENTITIES.lock().unwrap().as_ref(),
        Some(&Vec::new())
    );
}

#[rstest]
fn attacher_and_detacher_macros_run_system_lifecycle_hooks() {
    *fx::LIFECYCLE_ATTACH_ENTITY.lock().unwrap() = None;
    *fx::LIFECYCLE_DETACH_ENTITY.lock().unwrap() = None;

    let mut scene = Scene::new();
    scene.add_system("lifecycle".to_owned(), 1).unwrap();

    let attach_entity = fx::LIFECYCLE_ATTACH_ENTITY.lock().unwrap().unwrap();
    assert_eq!(scene.get_entity_name(attach_entity), Ok("attached"));
    assert_eq!(*fx::LIFECYCLE_DETACH_ENTITY.lock().unwrap(), None);

    scene.remove_system("lifecycle").unwrap();

    let detach_entity = fx::LIFECYCLE_DETACH_ENTITY.lock().unwrap().unwrap();
    assert_eq!(scene.get_entity_name(detach_entity), Ok("detached"));
}
