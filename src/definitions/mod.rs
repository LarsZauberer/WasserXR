//! A set of definitions used to define a full WasserXR Plugin. There are
//! definitions for all objects that can be created with a plugin, e.g. systems,
//! components, assets, ...
//!
//! Definitions are raw C-ABI compatible input. They are later converted with
//! WasserXR's internal manifests into validated rust native definitions.

use std::error::Error;

/// Defines all errors that might occur when handling definitions
pub mod error;

/// The standard implementations that a definition has to implement
pub trait Definition {
    /// The validation error that the definition throws out
    type Error: Error;

    /// Function that validates the definition and sees if it is valid. If not,
    /// it will return a validation error which is of the type defined in
    /// the error type of the trait.
    ///
    /// # Safety
    ///
    /// All non-null C string pointers in the definition and its nested
    /// definitions must point to readable, nul-terminated strings that
    /// remain valid for the duration of this call. Any pointer/count array
    /// pair must describe a valid initialized array, and every function
    /// pointer must point to a valid function with the declared C ABI.
    ///
    /// # Design Decisions (for Devs)
    ///
    /// We keep the validation code here and not in the
    /// WasserXR internal manifests because the validation only depends on
    /// the definition itself and nothing from the manifests. The
    /// manifests might later create their extra validation logic but this
    /// shouldn't be necessary and probably avoided.
    unsafe fn validate(&self) -> Result<(), Self::Error>;
}

/// Defines the raw plugin information
pub mod plugins;

/// Define the raw component information
pub mod components;

/// Define the raw fields in components and assets
pub mod fields;

pub mod assets;
