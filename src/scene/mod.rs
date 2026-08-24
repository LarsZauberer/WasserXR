use crate::{entity::Entity, utils::storage_backend::StorageBackend};

pub mod basic_scene;

pub trait Scene {
    type EntityStorage: StorageBackend<Value: Entity>;

    fn new() -> Self;

    fn get_entity_storage(&self) -> &Self::EntityStorage;
    fn get_mut_entity_storage(&mut self) -> &mut Self::EntityStorage;
}
