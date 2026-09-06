use std::collections::HashMap;

use crate::utils::version::Version;
use crate::{
    definitions::{Definition, error::PluginDefinitionError, plugins::PluginDefinition},
    private::manifests::{Manifest, assets::AssetManifest, components::ComponentManifest},
};

/// The plugin manifest is the main manifest of each wasserxr plugin. It
/// contains all the **content** of the plugin in a validated and rust native
/// form.
///
/// In contrast to the direct wasserxr plugin, it doesn't deal with the I/O
/// operations of loading plugins. It just carries the content information.
#[derive(Debug, Clone)]
pub(crate) struct PluginManifest {
    pub name: String,
    pub engine_version: Version,

    pub components: HashMap<String, ComponentManifest>,
    pub assets: HashMap<String, AssetManifest>,
}

impl Manifest<PluginDefinition> for PluginManifest {
    unsafe fn checked_convert(value: PluginDefinition) -> Result<Self, PluginDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated definitions have valid names");
        Ok(Self {
            name: name.clone(),
            engine_version: value.engine_version,
            components: if value.component_count == 0 {
                HashMap::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.components, value.component_count) }
                    .iter()
                    .copied()
                    .map(|component| {
                        unsafe { ComponentManifest::checked_convert(component) }
                            .map(|manifest| (manifest.name.clone(), manifest))
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<HashMap<_, _>, PluginDefinitionError>>()?
            },
            assets: if value.asset_count == 0 {
                HashMap::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.assets, value.asset_count) }
                    .iter()
                    .copied()
                    .map(|asset| {
                        unsafe { AssetManifest::checked_convert(asset) }
                            .map(|manifest| (manifest.name.clone(), manifest))
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<HashMap<_, _>, PluginDefinitionError>>()?
            },
        })
    }
}
