use crate::utils::version::Version;
use crate::{
    definitions::{Definition, error::PluginDefinitionError, plugins::PluginDefinition},
    ids::{AssetTypeID, ComponentTypeID, SystemTypeID},
    private::{
        id_store::IDStore,
        manifests::{
            Manifest, assets::AssetManifest, components::ComponentManifest, systems::SystemManifest,
        },
    },
};

/// The plugin manifest is the main manifest of each wasserxr plugin. It
/// contains all the **content** of the plugin in a validated and rust native
/// form.
///
/// In contrast to the direct wasserxr plugin, it doesn't deal with the I/O
/// operations of loading plugins. It just carries the content information.
///
/// # Design Decision
///
/// Inside of the plugin everything has object type has it's own ID (see
/// [`ComponentTypeID`], [`AssetTypeID`], ...). They are used to make allow
/// cached resolution of the these types. For example, if a system want to
/// always add a component, it can cache these TypeIDs and pass it to the system
/// function.
#[derive(Debug)]
pub(crate) struct PluginManifest {
    pub name: String,
    pub engine_version: Version,

    pub components: IDStore<String, ComponentTypeID, ComponentManifest>,
    pub assets: IDStore<String, AssetTypeID, AssetManifest>,
    pub systems: IDStore<String, SystemTypeID, SystemManifest>,
}

impl Manifest<PluginDefinition> for PluginManifest {
    unsafe fn checked_convert(value: PluginDefinition) -> Result<Self, PluginDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated definitions have valid names");
        Ok(Self {
            name: name.clone(),
            engine_version: value.engine_version,
            components: {
                let mut components = IDStore::default();
                let definitions = if value.component_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.components, value.component_count) }
                };
                for component in definitions {
                    let manifest = unsafe { ComponentManifest::checked_convert(*component) }
                        .map_err(|error| (name.clone(), error))?;
                    components.insert_named(manifest.name.clone(), manifest);
                }
                components
            },
            assets: {
                let mut assets = IDStore::default();
                let definitions = if value.asset_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.assets, value.asset_count) }
                };
                for asset in definitions {
                    let manifest = unsafe { AssetManifest::checked_convert(*asset) }
                        .map_err(|error| (name.clone(), error))?;
                    assets.insert_named(manifest.name.clone(), manifest);
                }
                assets
            },
            systems: {
                let mut systems = IDStore::default();
                let definitions = if value.system_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.systems, value.system_count) }
                };
                for system in definitions {
                    let manifest = unsafe { SystemManifest::checked_convert(*system) }
                        .map_err(|error| (name.clone(), error))?;
                    systems.insert_named(manifest.name.clone(), manifest);
                }
                systems
            },
        })
    }
}
