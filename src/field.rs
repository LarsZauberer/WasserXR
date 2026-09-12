//! Public component-field query types.

use std::ffi::c_void;

use crate::ids::FieldID;

/// The access requested for a component field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldAccess {
    Read,
    Write,
}

/// The public API representation of an actively used component field.
///
/// Its corresponding field lock is held for the duration of the query
/// callback in which this value is provided.
#[derive(Debug, Clone, Copy)]
pub enum Field {
    Read(FieldID, *const c_void),
    Write(FieldID, *mut c_void),
}
