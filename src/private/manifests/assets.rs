use std::collections::HashMap;

use crate::{
    definitions::{
        Definition,
        assets::AssetDefinition,
        components::{Creator, Destroyer},
        error::AssetDefinitionError,
    },
    private::manifests::{Manifest, fields::AssetFieldManifest},
};

#[derive(Debug, Clone)]
pub(crate) struct AssetManifest {
    pub name: String,

    pub creator: Creator,
    pub destroyer: Destroyer,

    pub fields: HashMap<String, AssetFieldManifest>,
}

impl Manifest<AssetDefinition> for AssetManifest {
    unsafe fn checked_convert(value: AssetDefinition) -> Result<Self, AssetDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated definitions have valid names");
        Ok(Self {
            name: name.clone(),
            creator: value
                .creator
                .expect("validated asset definitions have a creator"),
            destroyer: value
                .destroyer
                .expect("validated asset definitions have a destroyer"),
            fields: if value.field_count == 0 {
                HashMap::new()
            } else {
                unsafe { std::slice::from_raw_parts(value.fields, value.field_count) }
                    .iter()
                    .copied()
                    .map(|field| {
                        unsafe { AssetFieldManifest::checked_convert(field) }
                            .map(|manifest| (manifest.name.clone(), manifest))
                            .map_err(|error| (name.clone(), error).into())
                    })
                    .collect::<Result<HashMap<_, _>, AssetDefinitionError>>()?
            },
        })
    }
}
