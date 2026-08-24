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
