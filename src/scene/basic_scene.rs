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
    type EntityStorage = ES;

    fn new() -> Self {
        todo!()
    }

    fn get_entity_storage(&self) -> &Self::EntityStorage {
        todo!()
    }

    fn get_mut_entity_storage(&mut self) -> &mut Self::EntityStorage {
        todo!()
    }
}
