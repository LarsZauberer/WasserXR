use crate::{
    definitions::{
        Definition,
        components::{ComponentDefinition, Creator, Destroyer},
    },
    manifests::fields::ComponentFieldManifest,
};

#[derive(Debug, Clone)]
pub struct ComponentManifest {
    name: String,

    creator: Creator,
    destroyer: Destroyer,

    fields: Vec<ComponentFieldManifest>,
}

impl From<ComponentDefinition> for ComponentManifest {
    fn from(value: ComponentDefinition) -> Self {
        debug_assert!(unsafe { value.validate() }.is_ok());
        todo!()
    }
}
