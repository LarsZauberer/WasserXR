//! This module defines the core the scene in WasserXR. The scene is the most fundamental object in
//! WasserXR. It carries the state of the entire ECS. This means that all entities, components,
//! systems and plugins live under it. If it goes out of scope, everything else should be dropped.

use crate::scene::error::SceneError;

pub(crate) mod basic_scene;

/// The scene is the most fundamental object in WasserXR. It carries all the objects of the entire
/// ECS, like entities, components, systems and plugins. If the scene goes out of scope, everything
/// should be dropped.
///
/// This trait shows what operations a scene implements and what they do
pub(crate) trait Scene {
    /// The handle of the entities that is returned to the user to mention entities.
    type EntityID: Copy;

    /// Spawn a new entity in the scene. It will create a new one and create a new handle
    /// [`Self::EntityID`] for it. The handle is then returned to the user.
    fn add_entity(&mut self) -> Self::EntityID;

    /// Removes an entity and it's entire storage of components with it. If an entity with the
    /// handle [`Self::EntityID`] cannot be found, it will return a [`SceneError::EntityNotFound`].
    /// Otherwise, it will remove the entity from the scene.
    fn remove_entity(&mut self, id: Self::EntityID) -> Result<(), SceneError>;

    /// Returns all the entity handles currently stored in the scene
    fn get_entities(&self) -> Vec<Self::EntityID>;

    /// Removes all the entities, components, systems from the scene. Important to note that it
    /// doesn't remove any plugins. The plugins will persist and need to be manually unloaded unding
    /// the [`Self::unload_plugin`].
    fn reset(&mut self);
}
