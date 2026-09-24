use crate::definitions::error::MethodDefinitionError;

/// Reports a failure while converting a raw method to its manifest.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MethodManifestError {
    InvalidDefinition(MethodDefinitionError),
}
