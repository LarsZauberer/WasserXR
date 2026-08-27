use crate::utils::version::Version;
use crate::{
    definitions::{Definition, error::PluginDefinitionError, plugins::PluginDefinition},
    private::manifests::{Manifest, components::ComponentManifest},
};

/// The plugin manifest is the main manifest of each wasserxr plugin. It contains all the
/// **content** of the plugin in a validated and rust native form.
///
/// In contrast to the direct wasserxr plugin, it doesn't deal with the I/O operations of loading
/// plugins. It just carries the content information.
#[derive(Debug, Clone)]
pub(crate) struct PluginManifest {
    pub name: String,
    pub engine_version: Version,

    pub components: Vec<ComponentManifest>,
}

impl Manifest<PluginDefinition> for PluginManifest {
    unsafe fn checked_convert(value: PluginDefinition) -> Result<Self, PluginDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated definitions have valid names");
        Ok(Self {
            name: name.clone(),
            engine_version: value.engine_version,
            components: if value.component_count == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.components, value.component_count) }
                    .iter()
                    .copied()
                    .map(|component| {
                        unsafe { ComponentManifest::checked_convert(component) }
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<_, PluginDefinitionError>>()?
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use rstest::{fixture, rstest};

    use super::*;
    use crate::definitions::{components::ComponentDefinition, fields::ComponentFieldDefinition};

    unsafe extern "C" fn creator() -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn destroyer(_: *mut c_void) {}

    #[fixture]
    fn plugin() -> PluginDefinition {
        static NAME: &[u8] = b"example\0";
        static COMPONENT_NAME: &[u8] = b"Transform\0";
        static FIELD_NAME: &[u8] = b"position\0";
        let fields = Box::leak(Box::new([ComponentFieldDefinition {
            name: FIELD_NAME.as_ptr().cast(),
            getter: None,
            mutable: 0,
            serializer: None,
            deserializer: None,
        }]));
        let components = Box::leak(Box::new([ComponentDefinition {
            name: COMPONENT_NAME.as_ptr().cast(),
            creator: Some(creator),
            destroyer: Some(destroyer),
            fields: fields.as_ptr(),
            field_count: 1,
        }]));

        PluginDefinition {
            name: NAME.as_ptr().cast(),
            engine_version: Version {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            },
            components: components.as_ptr(),
            component_count: 1,
        }
    }

    #[rstest]
    fn converts_plugin(plugin: PluginDefinition) {
        let manifest = unsafe { PluginManifest::checked_convert(plugin) }.unwrap();

        assert_eq!(manifest.name, "example");
        assert_eq!(manifest.components[0].name, "Transform");
        assert_eq!(manifest.components[0].fields[0].name, "position");
    }
}
