use crate::definitions::{
    Definition,
    fields::{AssetFieldDefinition, ComponentFieldDefinition, Deserializer, Getter, Serializer},
};

#[derive(Debug, Clone)]
pub struct ComponentFieldManifest {
    name: String,

    mutable: bool,
    getter: Option<Getter>,
    serializer: Option<Serializer>,
    deserializer: Option<Deserializer>,
}

impl From<ComponentFieldDefinition> for ComponentFieldManifest {
    fn from(value: ComponentFieldDefinition) -> Self {
        debug_assert!(unsafe { value.validate() }.is_ok());
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct AssetFieldManifest {
    name: String,
    getter: Getter,
}

impl From<AssetFieldDefinition> for AssetFieldManifest {
    fn from(value: AssetFieldDefinition) -> Self {
        debug_assert!(unsafe { value.validate() }.is_ok());
        todo!()
    }
}
