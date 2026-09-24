use crate::definitions::{
    Definition,
    methods::{Method, MethodDefinition},
};
use crate::private::manifests::error::MethodManifestError;

/// Stores a validated method name and callback.
#[derive(Debug)]
pub(crate) struct MethodManifest {
    pub(crate) name: String,
    pub(crate) method: Method,
}

impl MethodManifest {
    /// Validates and copies a raw method definition.
    ///
    /// # Safety
    ///
    /// The raw definition must satisfy [`Definition::validate`]'s pointer
    /// requirements, and its callback must remain valid while the manifest is
    /// used.
    pub unsafe fn checked_convert(value: MethodDefinition) -> Result<Self, MethodManifestError> {
        unsafe { value.validate() }.map_err(MethodManifestError::InvalidDefinition)?;
        Ok(Self {
            name: unsafe { value.name() }.expect("validated methods have valid names"),
            method: value.method.expect("validated methods have a callback"),
        })
    }
}
