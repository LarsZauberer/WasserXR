use std::collections::HashMap;

use crate::{
    definitions::{
        Definition,
        components::{ComponentDefinition, Creator, Destroyer},
        error::ComponentDefinitionError,
    },
    private::manifests::{Manifest, fields::ComponentFieldManifest},
};

#[derive(Debug, Clone)]
pub(crate) struct ComponentManifest {
    pub name: String,

    pub creator: Creator,
    pub destroyer: Destroyer,

    pub fields: HashMap<String, ComponentFieldManifest>,
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
                HashMap::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.fields, value.field_count) }
                    .iter()
                    .copied()
                    .map(|field| {
                        unsafe { ComponentFieldManifest::checked_convert(field) }
                            .map(|manifest| (manifest.name.clone(), manifest))
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<HashMap<_, _>, ComponentDefinitionError>>()?
            },
        })
    }
}
