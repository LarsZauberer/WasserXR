use crate::{
    definitions::{
        Definition,
        components::{ComponentDefinition, Creator, Destroyer},
        error::ComponentDefinitionError,
    },
    ids::{FieldTypeSlot, MethodTypeSlot},
    private::{
        id_store::IDStore,
        manifests::{
            Manifest, error::MethodManifestError, fields::ComponentFieldManifest,
            methods::MethodManifest,
        },
    },
};

#[derive(Debug)]
pub(crate) struct ComponentManifest {
    pub name: String,

    pub creator: Creator,
    pub destroyer: Destroyer,

    pub fields: IDStore<String, FieldTypeSlot, ComponentFieldManifest>,
    pub methods: IDStore<String, MethodTypeSlot, MethodManifest>,
}

impl Manifest<ComponentDefinition> for ComponentManifest {
    type Error = ComponentDefinitionError;

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
            fields: {
                let mut fields = IDStore::default();
                let definitions = if value.field_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.fields, value.field_count) }
                };
                for field in definitions {
                    let manifest = unsafe { ComponentFieldManifest::checked_convert(*field) }
                        .map_err(|error| (name.clone(), error))?;
                    fields.insert_named(manifest.name.clone(), manifest);
                }
                fields
            },
            methods: {
                let mut methods = IDStore::default();
                let definitions = if value.method_count == 0 {
                    &[]
                } else {
                    unsafe { std::slice::from_raw_parts(value.methods, value.method_count) }
                };
                for method in definitions {
                    let manifest =
                        unsafe { MethodManifest::checked_convert(*method) }.map_err(|error| {
                            match error {
                                MethodManifestError::InvalidDefinition(error) => {
                                    ComponentDefinitionError::MethodInvalid(name.clone(), error)
                                }
                            }
                        })?;
                    methods.insert_named(manifest.name.clone(), manifest);
                }
                methods
            },
        })
    }
}
