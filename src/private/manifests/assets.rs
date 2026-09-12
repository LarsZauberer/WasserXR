use crate::{
    definitions::{
        Definition,
        assets::AssetDefinition,
        components::{Creator, Destroyer},
        error::AssetDefinitionError,
    },
    ids::AssetFieldTypeID,
    private::{
        id_store::IDStore,
        manifests::{Manifest, fields::AssetFieldManifest},
    },
};

#[derive(Debug)]
pub(crate) struct AssetManifest {
    pub name: String,

    pub creator: Creator,
    pub destroyer: Destroyer,

    pub fields: IDStore<String, AssetFieldTypeID, AssetFieldManifest>,
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
            fields: {
                let mut fields = IDStore::default();
                let definitions = if value.field_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.fields, value.field_count) }
                };
                for field in definitions {
                    let manifest = unsafe { AssetFieldManifest::checked_convert(*field) }
                        .map_err(|error| (name.clone(), error))?;
                    fields.insert_named(manifest.name.clone(), manifest);
                }
                fields
            },
        })
    }
}
