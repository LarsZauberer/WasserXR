//! This module implements and tests a basic [`crate::private::scene::Scene`] implementation.

use crate::private::{
    entities::Entity,
    scene::{Scene, scene_error::SceneError},
    utils::storage_backend::StorageBackend,
};

/// This is a basic implementation of a [`crate::private::scene::Scene`]. It will store Entities
/// using a storage backend where all the values implement the [`crate::private::entities::Entity`]
/// trait.
pub(crate) struct BasicScene<ES>
where
    ES: StorageBackend<Value: Entity + Default>,
{
    entity_storage: ES,
}

impl<ES> Scene for BasicScene<ES>
where
    ES: StorageBackend<Value: Entity + Default>,
{
    type EntityID = ES::Key;

    fn add_entity(&mut self) -> Self::EntityID {
        self.entity_storage.insert(ES::Value::default())
    }

    fn remove_entity(&mut self, id: Self::EntityID) -> Result<(), SceneError> {
        self.entity_storage
            .remove(id)
            .map(|_| ())
            .ok_or(SceneError::EntityNotFound)
    }

    fn reset(&mut self) {
        let ids: Vec<_> = self.entity_storage.iter_key().collect();
        for id in ids {
            self.entity_storage.remove(id);
        }
    }

    fn get_entities(&self) -> Vec<Self::EntityID> {
        self.entity_storage.iter_key().collect()
    }
}

#[cfg(test)]
mod slot_map_test {
    use crate::private::entities::Entity;

    use super::*;
    use rstest::{fixture, rstest};
    use slotmap::{DefaultKey, SlotMap};

    #[derive(Default)]
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
        scene.reset();
        scene
            .remove_entity(entity)
            .expect_err("The removed entity can still be removed");
    }
}
