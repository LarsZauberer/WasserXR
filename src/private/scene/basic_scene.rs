//! This module implements and tests a basic [`crate::private::scene::Scene`] implementation.

use std::{error::Error, fmt::Display};

use crate::private::{entities::Entity, scene::Scene, utils::storage_backend::StorageBackend};

/// This is a basic implementation of a [`crate::private::scene::Scene`]. It will store Entities
/// using a storage backend where all the values implement the [`crate::private::entities::Entity`]
/// trait.
pub(crate) struct BasicScene<ES>
where
    ES: StorageBackend<Value: Entity + Default>,
{
    entity_storage: ES,
}

impl<ES> BasicScene<ES>
where
    ES: StorageBackend<Value: Entity + Default>,
{
    pub(crate) fn new(entity_storage: ES) -> Self {
        Self { entity_storage }
    }
}

impl<ES> Scene for BasicScene<ES>
where
    ES: StorageBackend<Value: Entity + Default>,
{
    type Error = BasicSceneError;
    type EntityID = ES::Key;

    fn add_entity(&mut self) -> Self::EntityID {
        self.entity_storage.insert(ES::Value::default())
    }

    fn remove_entity(&mut self, id: Self::EntityID) -> Result<(), Self::Error> {
        self.entity_storage
            .remove(id)
            .map(|_| ())
            .ok_or(BasicSceneError::EntityNotFound)
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

#[derive(Debug)]
pub enum BasicSceneError {
    EntityNotFound,
}

impl Display for BasicSceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BasicSceneError {}

#[cfg(test)]
mod slot_map_test {
    use crate::private::entities::Entity;
    use crate::scene::EntityID;

    use super::*;
    use rstest::{fixture, rstest};
    use slotmap::SlotMap;

    #[derive(Default)]
    struct MockEntity {}

    impl Entity for MockEntity {}

    type MockEntityStorage = SlotMap<EntityID, MockEntity>;
    type MockScene = BasicScene<MockEntityStorage>;

    #[fixture]
    fn scene() -> MockScene {
        MockScene::new(SlotMap::with_key())
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
