//! Integration coverage for round-tripping a nested, serde-backed component
//! field through a reload. The component lives in the shared fixture.

#[path = "../src/testing_plugin_fixture.rs"]
mod testing_plugin_fixture;

use rstest::rstest;
use testing_plugin_fixture::Nested;
use wasserxr::scene::Scene;

#[rstest]
fn nested_component_field_survives_reload() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "NestedComponent".to_owned())
        .expect("Failed to create NestedComponent");

    {
        let (data,) = scene
            .query_mut::<(&mut Nested,)>(entity, "NestedComponent", &["data"])
            .expect("Failed to get data field");
        data.numbers.push(5);
        data.vector[1] = 0.5;
        data.vector[2] = -1.0;
    }

    scene.reload().unwrap();

    let (data,) = scene
        .query::<(&Nested,)>(entity, "NestedComponent", &["data"])
        .expect("Failed to get data field");
    assert_eq!(data.numbers, vec![5]);
    assert_eq!(data.vector, [1.0, 0.5, -1.0]);
}
