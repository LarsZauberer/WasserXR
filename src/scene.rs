use slotmap::{SlotMap, new_key_type};

use crate::errors::SceneError;

new_key_type! {
/// EntityID is the main handle how you will address an entity. It uniquely identifies an entity
/// within a Scene. It is not a globally unique identifier across multiple scenes (if you are
/// maintaining multiple scenes)
///
/// It is designed to be cheaply copyable
pub struct EntityID;
}

type EntityStorage = SlotMap<EntityID, ()>;

/// The scene is the core object in WasserXR. It contains the main public API to access and maintain all ECS
/// objects.
///
/// While it is possible to have mutliple scenes per application, the scene is designed to only have
/// one Scene per application maintaining all the entities, components, systems, assets and plugins
/// currently active.
#[derive(Debug, Default)]
pub struct Scene {
    entities: EntityStorage,
}

impl Scene {
    /// Creates a new empty scene
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new entity and returns it's handle. The handle will be unique to every other
    /// entity ever created within this scene.
    pub fn add_entity(&mut self) -> EntityID {
        self.entities.insert(())
    }

    /// Removes a previsouly created entity from the scene. This will also delete all the associated
    /// components of the entity.
    ///
    /// If the entity couldn't be found with the handle, the function will return a
    /// [`SceneError::EntityNotFound`]
    pub fn remove_entity(&mut self, id: EntityID) -> Result<(), SceneError> {
        self.entities
            .remove(id)
            .map(|_| ())
            .ok_or(SceneError::EntityNotFound)
    }

    /// Returns a [`Vec<EntityID>`] of all the entity handles that are currently active in the
    /// scene.
    pub fn get_entities(&self) -> Vec<EntityID> {
        self.entities.keys().collect()
    }

    /// This will reset the scene's main objects. Meaning it will remove all the entities,
    /// components and systems
    ///
    /// It will **not** unload any plugins or remove cached assets
    pub fn reset(&mut self) -> Result<(), SceneError> {
        self.entities.clear();
        Ok(())
    }
}
