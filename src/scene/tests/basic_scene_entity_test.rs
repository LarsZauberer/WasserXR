use rstest::{fixture, rstest};

use crate::scene::Scene;

#[fixture]
fn scene() -> Scene {
    Scene::new()
}

#[rstest]
fn test_entity_lifecycle(mut scene: Scene) {
    let entity_id = scene.add_entity();
    assert!(scene.get_entities().contains(&entity_id));

    scene
        .remove_entity(entity_id)
        .expect("Failed to remove existing entity");

    scene
        .remove_entity(entity_id)
        .expect_err("Was able to remove entity twice");
}

#[rstest]
fn test_entity_reset(mut scene: Scene) {
    let entity_id = scene.add_entity();

    scene.reset();

    assert!(!scene.get_entities().contains(&entity_id));

    scene
        .remove_entity(entity_id)
        .expect_err("Was able to remove entity twice");
}
