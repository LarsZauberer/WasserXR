use crate::{entity::Entity, scene::Scene, utils::storage_backend::StorageBackend};

pub struct BasicScene<ES>
where
    ES: StorageBackend<Value: Entity>,
{
    entity_storage: ES,
}

impl<ES> Scene for BasicScene<ES>
where
    ES: StorageBackend<Value: Entity>,
{
    type Entity = ES::Value;

    type EntityID = ES::Key;

    fn add_entity(&mut self) -> Self::EntityID {
        todo!()
    }

    fn remove_entity(&mut self, id: Self::EntityID) -> Result<(), super::scene_error::SceneError> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), super::scene_error::SceneError> {
        todo!()
    }
}

#[cfg(test)]
mod slot_map_test {
    use crate::entity::Entity;

    use super::*;
    use rstest::{fixture, rstest};
    use slotmap::{DefaultKey, SlotMap};

    struct MockEntity {}

    impl Entity for MockEntity {}

    type MockEntityID = DefaultKey;
    type MockEntityStorage = SlotMap<MockEntityID, MockEntity>;
    type MockScene = BasicScene<MockEntityStorage>;

    #[fixture]
    fn scene() -> MockScene {
        MockScene {
            entity_storage: SlotMap::<DefaultKey, MockEntity>::new(),
        }
    }

    #[rstest]
    fn test_add_remove(mut scene: MockScene) {
        let entity = scene.add_entity();
        scene
            .remove_entity(entity)
            .expect("Failed to remove the just created entity");
        scene
            .remove_entity(entity)
            .expect_err("The removed entity can still be removed");
    }

    #[rstest]
    fn test_reset(mut scene: MockScene) {
        let entity = scene.add_entity();
        scene.reset().expect("Failed to reset the scene");
        scene
            .remove_entity(entity)
            .expect_err("The removed entity can still be removed");
    }
}
