//! C-compatible component method definitions.

use std::ffi::{c_char, c_void};

use crate::{
    definitions::{Definition, error::MethodDefinitionError},
    scene::Scene,
    utils::ffi::validate_string,
};

/// A callback for a component method.
///
/// # Safety
///
/// `scene` must point to a live scene, `component` to the owning component's
/// data, and `arguments` must describe `argument_count` valid argument
/// pointers. The callback must not unwind across the C ABI boundary.
pub type Method = unsafe extern "C" fn(
    scene: *const Scene,
    component: *mut c_void,
    arguments: *const *mut c_void,
    argument_count: usize,
);

/// A named component method. The callback and name must remain valid while
/// the owning plugin is loaded.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MethodDefinition {
    pub name: *const c_char,
    pub method: Option<
        unsafe extern "C" fn(
            scene: *const Scene,
            component: *mut c_void,
            arguments: *const *mut c_void,
            argument_count: usize,
        ),
    >,
}

impl Definition for MethodDefinition {
    type Error = MethodDefinitionError;

    /// Validates the method name and callback pointer.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        if self.method.is_none() {
            return Err(MethodDefinitionError::MethodIsNull(name));
        }
        Ok(())
    }
}

impl MethodDefinition {
    /// Returns the method name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string.
    pub(crate) unsafe fn name(&self) -> Result<String, MethodDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
