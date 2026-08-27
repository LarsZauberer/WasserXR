//! Manifests are the validated, Rust-native interpretation of raw C-compatible
//! [`crate::definitions`] descriptors.

use crate::definitions::Definition;

/// Converts a raw definition into a validated manifest.
pub(crate) trait Manifest<D: Definition>: Sized {
    /// Validates and converts `definition`, consuming the raw descriptor.
    ///
    /// # Safety
    ///
    /// `definition` and all nested raw pointers must satisfy [`Definition::validate`]'s safety
    /// requirements for the duration of conversion. Callback code retained by the manifest must
    /// remain valid while that manifest is used.
    ///
    /// # Design Decision
    ///
    /// This function will call the [`Definition::validate`] function to verify that the definition
    /// is correct. This is why this function is unsafe. Only after the Manifest has been builded,
    /// the whole thing is considered to be safe.
    ///
    /// We don't put the validation logic directly in here, because validating the definition
    /// doesn't depend on the manifests. To keep code where it's responsibility is and it has closer
    /// coupling, we keep it there.
    unsafe fn checked_convert(definition: D) -> Result<Self, D::Error>;
}

pub(crate) mod components;
pub(crate) mod fields;
pub(crate) mod plugins;
