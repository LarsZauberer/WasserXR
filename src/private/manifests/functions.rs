use crate::definitions::{
    Definition,
    error::FunctionManifestError,
    functions::{Function, FunctionDefinition},
};
use crate::private::manifests::Manifest;

/// Owned function metadata and its validated callback.
#[derive(Debug)]
pub(crate) struct FunctionManifest {
    pub(crate) name: String,
    pub(crate) function: Function,
}

impl Manifest<FunctionDefinition> for FunctionManifest {
    type Error = FunctionManifestError;

    unsafe fn checked_convert(value: FunctionDefinition) -> Result<Self, FunctionManifestError> {
        unsafe { value.validate() }.map_err(FunctionManifestError::DefinitionInvalid)?;
        Ok(Self {
            name: unsafe { value.name() }.expect("validated function has a valid name"),
            function: value.function.expect("validated function has a callback"),
        })
    }
}
