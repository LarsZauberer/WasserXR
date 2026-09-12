//! The definition of systems in WasserXR.

use std::ffi::c_char;

use crate::{
    definitions::{Definition, error::SystemDefinitionError, type_id_requests::TypeIDRequests},
    ids::TypeID,
    scene::Scene,
    utils::ffi::validate_string,
};

/// Attaches a system to a scene.
pub type Attacher =
    unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize);

/// Runs a system for a scene.
pub type Runner =
    unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize);

/// Detaches a system from a scene.
pub type Detacher =
    unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize);

/// Defines a system and its scheduling and type-ID requirements.
///
/// All arrays use C-compatible pointer/count pairs. Their pointers must remain
/// valid for the duration of validation and while the definition is in use.
/// Each callback receives resolved type IDs in the same order as
/// `type_id_requests`; the ID pointer is only valid for the duration of the
/// callback.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SystemDefinition {
    pub name: *const c_char,

    pub attacher: Option<
        unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize),
    >,
    pub runner: Option<
        unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize),
    >,
    pub detacher: Option<
        unsafe extern "C" fn(scene: *const Scene, type_ids: *const TypeID, type_id_count: usize),
    >,

    pub requires: *const *const c_char,
    pub requires_count: usize,

    pub wanted_by: *const *const c_char,
    pub wanted_by_count: usize,

    pub type_id_requests: *const TypeIDRequests,
    pub type_id_request_count: usize,
}

impl Definition for SystemDefinition {
    type Error = SystemDefinitionError;

    /// # Safety
    ///
    /// This implementation has no additional safety requirements beyond those
    /// of [`Definition::validate`].
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };

        if self.runner.is_none() {
            return Err(SystemDefinitionError::RunnerIsNull(name));
        }

        let requires = if self.requires_count == 0 {
            &[]
        } else {
            if self.requires.is_null() {
                return Err(SystemDefinitionError::RequiresIsNull(name));
            }
            unsafe { std::slice::from_raw_parts(self.requires, self.requires_count) }
        };
        for &system in requires {
            unsafe { validate_string(system, |_| ()) }.map_err(|error| {
                SystemDefinitionError::RequiredSystemInvalid(name.clone(), error)
            })?;
        }

        let wanted_by = if self.wanted_by_count == 0 {
            &[]
        } else {
            if self.wanted_by.is_null() {
                return Err(SystemDefinitionError::WantedByIsNull(name));
            }
            unsafe { std::slice::from_raw_parts(self.wanted_by, self.wanted_by_count) }
        };
        for &system in wanted_by {
            unsafe { validate_string(system, |_| ()) }.map_err(|error| {
                SystemDefinitionError::WantedBySystemInvalid(name.clone(), error)
            })?;
        }

        let requests = if self.type_id_request_count == 0 {
            &[]
        } else {
            if self.type_id_requests.is_null() {
                return Err(SystemDefinitionError::TypeIDRequestsIsNull(name));
            }
            unsafe { std::slice::from_raw_parts(self.type_id_requests, self.type_id_request_count) }
        };
        for request in requests {
            unsafe { request.validate() }.map_err(|error| {
                SystemDefinitionError::TypeIDRequestInvalid(name.clone(), error)
            })?;
        }

        Ok(())
    }
}

impl SystemDefinition {
    /// Returns the validated system name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, SystemDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
