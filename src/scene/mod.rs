use crate::{entity::Entity, scene::scene_error::SceneError};

pub mod basic_scene;
pub mod scene_error;

pub trait Scene {
    type Entity: Entity;
    type EntityID: Copy;

    fn add_entity(&mut self) -> Self::EntityID;
    fn remove_entity(&mut self, id: Self::EntityID) -> Result<(), SceneError>;

    fn reset(&mut self) -> Result<(), SceneError>;
}
