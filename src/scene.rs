//! The public scene API.
//!
//! A [`Scene`] owns its entities and returns opaque [`EntityID`] values for them.
//! Entity storage is an implementation detail: callers cannot select a storage backend or access
//! its keys.

use slotmap::{SlotMap, new_key_type};

use crate::private::{
    entities::basic_entity::BasicEntity,
    scene::{Scene as SceneImplementation, basic_scene::BasicScene},
};

pub use crate::private::scene::basic_scene::BasicSceneError;

new_key_type! {
    /// An opaque handle to an entity in the [`Scene`].
    ///
    /// Entity IDs are created by [`Scene::add_entity`] and become invalid when the entity is
    /// removed or the scene is reset.
    pub struct EntityID;
}

type DefaultScene = BasicScene<SlotMap<EntityID, BasicEntity>>;

/// The public facade for a WasserXR scene.
///
/// `Scene` deliberately has no storage type parameter. Its entity and storage implementations are
/// private so they can change without changing the public API.
pub struct Scene {
    implementation: DefaultScene,
}

impl Scene {
    /// Creates an empty scene.
    #[must_use]
    pub fn new() -> Self {
        Self {
            implementation: BasicScene::new(SlotMap::with_key()),
        }
    }

    /// Adds an entity and returns its ID.
    pub fn add_entity(&mut self) -> EntityID {
        self.implementation.add_entity()
    }

    /// Removes an entity.
    ///
    /// Returns [`SceneError::EntityNotFound`] if the entity was already removed or the scene was
    /// reset after the ID was created.
    pub fn remove_entity(&mut self, id: EntityID) -> Result<(), BasicSceneError> {
        self.implementation.remove_entity(id)
    }

    /// Returns the IDs of all entities currently in the scene.
    #[must_use]
    pub fn get_entities(&self) -> Vec<EntityID> {
        self.implementation.get_entities()
    }

    /// Removes every entity from the scene.
    pub fn reset(&mut self) {
        self.implementation.reset();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
