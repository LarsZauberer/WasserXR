use wasserxr::{errors::SceneError, scene::Scene};

#[test]
fn added_entities_are_reported() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert!(scene.get_entities().contains(&entity));
}

#[test]
fn removed_entity_is_absent_and_cannot_be_removed_again() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene.remove_entity(entity).unwrap();

    assert!(!scene.get_entities().contains(&entity));
    assert!(matches!(
        scene.remove_entity(entity),
        Err(SceneError::EntityNotFound)
    ));
}

#[test]
fn removing_one_entity_preserves_others() {
    let mut scene = Scene::new();
    let removed = scene.add_entity();
    let retained = scene.add_entity();

    scene.remove_entity(removed).unwrap();

    assert_eq!(scene.get_entities(), vec![retained]);
}

#[test]
fn reset_invalidates_existing_entity_ids() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene.reset().unwrap();

    assert!(scene.get_entities().is_empty());
    assert!(matches!(
        scene.remove_entity(entity),
        Err(SceneError::EntityNotFound)
    ));
}
