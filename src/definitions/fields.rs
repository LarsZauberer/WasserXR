//! This module provides a FieldDescription for AssetFields and ComponentFields

use std::ffi::{c_char, c_void};

use crate::definitions::{
    Definition,
    error::{AssetFieldDefinitionError, ComponentFieldDefinitionError},
};

/// Function to get from a component or asset a pointer to the actual field data. This is the way
/// wasserxr provides access a field.
pub type Getter = unsafe extern "C" fn(ptr: *const c_void) -> *mut c_void;

/// Provides a function for a component field that serializes the data into binary data.
// TODO: TBD the final return type here
pub type Serializer = unsafe extern "C" fn(ptr: *const c_void);

/// Provides a function for a component field to turn binary data into the corresponding field data.
// TODO: TBD the final argument type here
pub type Deserializer = unsafe extern "C" fn(ptr: *const c_void);

/// This is a definition of a field for a component. It contains the name of the field, a typehint
/// and a access/serialization information/permissions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ComponentFieldDefinition {
    name: *const c_char,

    // Access
    getter: Option<Getter>,
    mutable: i32,

    // Serialization
    serializer: Option<Serializer>,
    serializable: i32,
    deserializer: Option<Deserializer>,
    deserializable: i32,
}

impl Definition for ComponentFieldDefinition {
    type Error = ComponentFieldDefinitionError;

    unsafe fn validate(&self) -> Result<(), Self::Error> {
        // TODO: Check if the name is not null

        // TODO: Check that the name is valid

        // TODO: Check if mutable is nonzero, then getter may not be None

        // TODO: Check if serializable is nonzero, then serializer may not be None

        // TODO: Check if deserializer is nonzero, then deserializer may not be None

        // TODO: Create for all of the previous errors a DefinitionError
        todo!()
    }
}

/// This is a definition of a field for an asset. It is pretty much similar to the
/// [`wasserxr::definitions::fields::ComponentFieldDescription`] but instead has less functionality
/// that provide mutability or serialization for that field.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetFieldDefinition {}

impl Definition for AssetFieldDefinition {
    type Error = AssetFieldDefinitionError;

    unsafe fn validate(&self) -> Result<(), Self::Error> {
        todo!()
    }
}
