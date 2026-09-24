//! Definitions for plugin-provided global functions.

use std::ffi::{c_char, c_void};

use crate::{
    definitions::{Definition, error::FunctionDefinitionError},
    scene::Scene,
    utils::ffi::validate_string,
};

/// Runs a global function with an ordered array of opaque arguments.
pub type Function =
    unsafe extern "C" fn(scene: *const Scene, arguments: *const *mut c_void, argument_count: usize);

/// Raw C-compatible definition of a named global function.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FunctionDefinition {
    pub name: *const c_char,
    pub function: Option<
        unsafe extern "C" fn(
            scene: *const Scene,
            arguments: *const *mut c_void,
            argument_count: usize,
        ),
    >,
}

impl Definition for FunctionDefinition {
    type Error = FunctionDefinitionError;

    /// Validates the function's name and callback pointer.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        if self.function.is_none() {
            return Err(FunctionDefinitionError::FunctionIsNull(name));
        }
        Ok(())
    }
}

impl FunctionDefinition {
    /// Returns the validated function name.
    ///
    /// # Safety
    /// `self.name` must point to a valid, nul-terminated C string.
    pub(crate) unsafe fn name(&self) -> Result<String, FunctionDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
