//! The definition of components in WasserXR

use std::ffi::{c_char, c_void};

use crate::definitions::{
    Definition, error::ComponentDefinitionError, fields::ComponentFieldDefinition,
};
use crate::utils::ffi::validate_string;

/// Creator function for a component. It is a constructor for the component. It allocates the
/// component created onto the heap and then returns the raw pointer in form of a null pointer. The
/// creator function should not fail.
///
/// # Safety
///
/// There are no immediate preconditions for the function to be safe. The function might be some
/// function defined in some foreign language and is therefore inherently unsafe.
pub type Creator = unsafe extern "C" fn() -> *mut c_void;

/// Destroyer function for a component. It is basically a destructor call to the component. The
/// destroyer function should not fail.
/// It takes the pointer to the object created by the [`Creator`]
/// and destroys the object.
///
/// # Safety
///
/// The function requires a pointer that was created by the corresponding creator function of the
/// component.
pub type Destroyer = unsafe extern "C" fn(ptr: *mut c_void);

/// This is the definition defining a component in WasserXR. It contains a pointer to all the
/// functions to create, destroy the actual components and what kind of fields are included.
///
/// The field array uses a C-compatible pointer/count pair. The pointer must remain valid for the
/// duration of validation and while the definition is in use.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ComponentDefinition {
    pub name: *const c_char,
    pub creator: Option<Creator>,
    pub destroyer: Option<Destroyer>,

    pub fields: *const ComponentFieldDefinition,
    pub field_count: usize,
}

impl Definition for ComponentDefinition {
    type Error = ComponentDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };

        if self.creator.is_none() {
            return Err(ComponentDefinitionError::CreatorIsNull(name.clone()));
        }
        if self.destroyer.is_none() {
            return Err(ComponentDefinitionError::DestroyerIsNull(name.clone()));
        }

        // Convert the fields into a slice
        let fields = if self.field_count == 0 {
            &[]
        } else {
            if self.fields.is_null() {
                return Err(ComponentDefinitionError::FieldsIsNull(name.clone()));
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

impl ComponentDefinition {
    /// Returns the validated component name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, ComponentDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
