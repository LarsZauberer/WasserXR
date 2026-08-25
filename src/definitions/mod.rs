//! A set of definitions used to define a full WasserXR Plugin. There are definitions for all
//! objects that can be created with a plugin, e.g. systems, components, assets, ...
//!
//! These definitions are validated when you call [`wasserxr::scene::Scene::load_plugin`] and they
//! are also then turned into their corresponding [`wasserxr::manifests`] variant.
//!
//! The difference to the [`wasserxr::manifests`] is that the definitions contain raw C-ABI
//! compatible input while the [`wasserxr::manifests`] contain already validated and more converted
//! notions.

use std::error::Error;

/// Defines all errors that might occur when handling definitions
pub mod error;

/// The standard implementations that a definition has to implement
pub trait Definition {
    /// The validation error that the definition throws out
    type Error: Error;

    /// Function that validates the definition and sees if it is valid. If not, it will return a
    /// validation error which is of the type defined in the error type of the trait.
    ///
    /// # Safety
    ///
    /// All raw pointers contained in `self` and its nested definitions must be valid for the
    /// duration of the call. Any C string pointers must point to valid, NUL-terminated strings.
    unsafe fn validate(&self) -> Result<(), Self::Error>;
}

/// Defines the raw plugin information
pub mod plugins;

/// Define the raw component information
pub mod components;

/// Define the raw fields in components and assets
pub mod fields;
