use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum PluginDefinitionError {
    NameIsNull,
    ComponentInvalid(String, ComponentDefinitionError),
}

#[derive(Debug)]
pub enum ComponentDefinitionError {
    NameIsNull,
    FieldInvalid(String, ComponentFieldDefinitionError),
}

#[derive(Debug)]
pub enum ComponentFieldDefinitionError {
    NameIsNull,
    MutableButNoGetter(String),
    SerializableButNoSerializer(String),
    DeserializableButNoDeserializer(String),
}

#[derive(Debug)]
pub enum AssetFieldDefinitionError {
    NameIsNull,
}

impl Display for PluginDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PluginDefinitionError {}

impl Display for ComponentDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ComponentDefinitionError {}

impl Display for ComponentFieldDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ComponentFieldDefinitionError {}

impl Display for AssetFieldDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for AssetFieldDefinitionError {}
