use crate::{
    definitions::{
        Definition,
        components::{ComponentDefinition, Creator, Destroyer},
        error::ComponentDefinitionError,
    },
    manifests::{Manifest, fields::ComponentFieldManifest},
};

#[derive(Debug, Clone)]
pub struct ComponentManifest {
    pub name: String,

    pub creator: Creator,
    pub destroyer: Destroyer,

    pub fields: Vec<ComponentFieldManifest>,
}

impl Manifest<ComponentDefinition> for ComponentManifest {
    unsafe fn checked_convert(
        value: ComponentDefinition,
    ) -> Result<Self, ComponentDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated definitions have valid names");
        Ok(Self {
            name: name.clone(),
            creator: value
                .creator
                .expect("validated component definitions have a creator"),
            destroyer: value
                .destroyer
                .expect("validated component definitions have a destroyer"),
            fields: if value.field_count == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.fields, value.field_count) }
                    .iter()
                    .copied()
                    .map(|field| {
                        unsafe { ComponentFieldManifest::checked_convert(field) }
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<_, ComponentDefinitionError>>()?
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use rstest::{fixture, rstest};

    use crate::definitions::fields::ComponentFieldDefinition;

    use super::*;

    unsafe extern "C" fn creator() -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn destroyer(_: *mut c_void) {}

    #[fixture]
    fn component() -> ComponentDefinition {
        static NAME: &[u8] = b"Transform\0";
        static FIELD_NAME: &[u8] = b"position\0";
        let fields = Box::leak(Box::new([ComponentFieldDefinition {
            name: FIELD_NAME.as_ptr().cast(),
            getter: None,
            mutable: 0,
            serializer: None,
            deserializer: None,
        }]));

        ComponentDefinition {
            name: NAME.as_ptr().cast(),
            creator: Some(creator),
            destroyer: Some(destroyer),
            fields: fields.as_ptr(),
            field_count: 1,
        }
    }

    #[rstest]
    fn converts_component(component: ComponentDefinition) {
        let manifest = unsafe { ComponentManifest::checked_convert(component) }.unwrap();

        assert_eq!(manifest.name, "Transform");
        assert!(std::ptr::fn_addr_eq(manifest.creator, creator as Creator));
        assert!(std::ptr::fn_addr_eq(
            manifest.destroyer,
            destroyer as Destroyer
        ));
        assert_eq!(manifest.fields[0].name, "position");
    }
}
