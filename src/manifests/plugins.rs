use crate::definitions::Definition;
use crate::definitions::plugins::PluginDefinition;
use crate::manifests::components::ComponentManifest;
use crate::utils::version::Version;

/// The plugin manifest is the main manifest of each wasserxr plugin. It contains all the
/// **content** of the plugin in a validated and rust native form.
///
/// In contrast to the direct wasserxr plugin, it doesn't deal with the I/O operations of loading
/// plugins. It just carries the content information.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    name: String,
    engine_version: Version,

    components: Vec<ComponentManifest>,
}

impl From<PluginDefinition> for PluginManifest {
    /// # Safety
    ///
    /// To call this, it is **required** that a call to [`wasserxr::definitions::Definition::validate`]
    /// succeeds.
    fn from(value: PluginDefinition) -> Self {
        debug_assert!(unsafe { value.validate() }.is_ok());
        todo!()
    }
}
