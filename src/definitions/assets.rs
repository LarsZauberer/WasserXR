use std::ffi::c_char;

use crate::{
    definitions::{
        Definition,
        components::{Creator, Destroyer},
        error::AssetDefinitionError,
        fields::AssetFieldDefinition,
    },
    utils::ffi::validate_string,
};

/// AssetDefinition defines an asset type in a plugin.
///
/// For it to be valid, it needs to have a creator, destroyer and a valid name
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AssetDefinition {
    pub name: *const c_char,
    pub creator: Option<Creator>,
    pub destroyer: Option<Destroyer>,

    pub fields: *const AssetFieldDefinition,
    pub field_count: usize,
}

impl Definition for AssetDefinition {
    type Error = AssetDefinitionError;

    /// # Safety
    ///
    /// This implementation has no additional safety requirements beyond those
    /// of [`Definition::validate`].
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };

        if self.creator.is_none() {
            return Err(AssetDefinitionError::CreatorIsNull(name.clone()));
        }
        if self.destroyer.is_none() {
            return Err(AssetDefinitionError::DestroyerIsNull(name.clone()));
        }

        // Convert the fields into a slice
        let fields = if self.field_count == 0 {
            &[]
        } else {
            if self.fields.is_null() {
                return Err(AssetDefinitionError::FieldsIsNull(name.clone()));
            }
            unsafe { std::slice::from_raw_parts(self.fields, self.field_count) }
        };

        for field in fields {
            if let Err(violation) = unsafe { field.validate() } {
                return Err((name, violation).into());
            }
        }

        Ok(())
    }
}

impl AssetDefinition {
    /// Returns the validated component name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, AssetDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
