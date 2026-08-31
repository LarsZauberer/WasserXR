use std::path::Path;

use slotmap::{SlotMap, new_key_type};

use crate::{
    definitions::plugins::PluginDefinition,
    errors::{PluginCompatibilityError, PluginError, SceneError},
    private::{
        manifests::{Manifest, plugins::PluginManifest},
        plugins::Plugin,
        utils::storage_backend::StorageBackend,
    },
};

new_key_type! {
/// EntityID is the main handle how you will address an entity. It uniquely identifies an entity
/// within a Scene. It is not a globally unique identifier across multiple scenes (if you are
/// maintaining multiple scenes)
///
/// It is designed to be cheaply copyable
pub struct EntityID;

/// Handle for a loaded plugin. It describes a plugin uniquely to the scene and cannot like the [`EntityID`] be used
/// in different scenes. This behavior is not supported.
pub struct PluginID;
}

type EntityStorage = SlotMap<EntityID, ()>;
type PluginStorage = SlotMap<PluginID, Plugin>;

/// The scene is the core object in WasserXR. It contains the main public API to
/// access and maintain all ECS objects.
///
/// While it is possible to have mutliple scenes per application, the scene is
/// designed to only have one Scene per application maintaining all the
/// entities, components, systems, assets and plugins currently active.
///
/// # Invariants
///
/// adsf asdf
///
/// asdf as dfdsa ffasd f dsafa dsa fsd fasdf sadf dfsad fsa df sad fsa afsfds
/// fdsa afds a fsdfsa asdf asfd asdf asdf asdf asdf asdf asdf dfa sfa
#[derive(Debug, Default)]
pub struct Scene {
    entities: EntityStorage,
    plugins: PluginStorage,
}

impl Scene {
    /// Creates a new empty scene
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new entity and returns it's handle. The handle will be unique
    /// to every other entity ever created within this scene.
    pub fn add_entity(&mut self) -> EntityID {
        self.entities.insert(())
    }

    /// Removes a previsouly created entity from the scene. This will also
    /// delete all the associated components of the entity.
    ///
    /// If the entity couldn't be found with the handle, the function will
    /// return a [`SceneError::EntityNotFound`]
    pub fn remove_entity(&mut self, id: EntityID) -> Result<(), SceneError> {
        self.entities
            .remove(id)
            .map(|_| ())
            .ok_or(SceneError::EntityNotFound)
    }

    /// Returns a [`Vec<EntityID>`] of all the entity handles that are currently
    /// active in the scene.
    pub fn get_entities(&self) -> Vec<EntityID> {
        self.entities.keys().collect()
    }

    /// This will reset the scene's main objects. Meaning it will remove all the
    /// entities, components and systems
    ///
    /// It will **not** unload any plugins or remove cached assets
    pub fn reset(&mut self) -> Result<(), SceneError> {
        self.entities.clear();
        Ok(())
    }

    /// Checks if the plugin that should be added to the scene is valid to be
    /// added to the scene. It checks the following conditions
    ///
    /// - Is a plugin with the same name already loaded?
    fn check_plugin_compatibility(&self, new_plugin: &Plugin) -> Result<(), SceneError> {
        if self.get_plugin(new_plugin.get_name()).is_some() {
            return Err(SceneError::from(
                PluginCompatibilityError::PluginWithSameNameExists,
            ));
        }

        Ok(())
    }

    /// Load a plugin from a shared object library.
    ///
    /// It is not allowed to have a plugin with the same name already loaded in
    /// the scene.
    ///
    /// # Safety
    ///
    /// The path must point to a valid shared object that can be read and has a
    /// globally defined variable called `wxr_plugin`. The `wxr_plugin`
    /// variable has to be of type [`PluginDefinition`] as has to be valid.
    /// Furthermore, the [`PluginDefinition`] musn't have any malformed content
    /// within it.
    pub unsafe fn load_plugin(&mut self, path: &Path) -> Result<PluginID, SceneError> {
        let plugin = unsafe { Plugin::load_shared(path) }.map_err(SceneError::from)?;

        // Check if the plugin can be combined with other plugins in the scene
        self.check_plugin_compatibility(&plugin)?;

        let id = self.plugins.insert(plugin);
        Ok(id)
    }

    /// Load a plugin from a statically linked and already [`PluginDefinition`]
    ///
    /// It is not allowed to have a plugin with the same name already loaded in
    /// the scene.
    ///
    /// # Safety
    ///
    /// The [`PluginDefinition`] must be valid and not have not any malformed
    /// content within.
    pub unsafe fn load_static_plugin(
        &mut self,
        definition: PluginDefinition,
    ) -> Result<PluginID, SceneError> {
        let manifest: PluginManifest = unsafe { Manifest::checked_convert(definition) }
            .map_err(|err| SceneError::from(PluginError::from(err)))?;
        let plugin = Plugin::load_static(manifest);

        // Check if the plugin can be combined with other plugins in the scene
        self.check_plugin_compatibility(&plugin)?;

        // Add plugin to scene
        let id = self.plugins.insert(plugin);
        Ok(id)
    }

    /// Get the handle of a plugin ([`PluginID`]) by searching for the name of a
    /// plugin
    pub fn get_plugin(&self, name: &str) -> Option<PluginID> {
        self.plugins
            .iter()
            .find(|(_, v)| v.get_name() == name)
            .map(|(k, _)| k)
    }

    /// Get all the [`PluginID`] of the currently actively loaded plugins in the
    /// scene
    pub fn get_plugins(&self) -> Vec<PluginID> {
        self.plugins.iter_key().collect()
    }
}
