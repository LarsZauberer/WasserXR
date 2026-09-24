use crate::{
    definitions::{
        Definition,
        error::SystemDefinitionError,
        systems::{Attacher, Detacher, Runner, SystemDefinition},
    },
    private::manifests::{Manifest, type_id_requests::TypeIDRequestManifest},
    utils::ffi::validate_string,
};

/// Validated, Rust-owned description of a plugin-provided system.
///
/// Its core responsibility is to copy scheduling relationships and type ID
/// requests across the FFI boundary into data the engine can retain safely.
/// Scheduling and type ID resolution are intentionally handled elsewhere.
#[derive(Debug)]
pub(crate) struct SystemManifest {
    pub name: String,
    pub attacher: Option<Attacher>,
    pub runner: Runner,
    pub detacher: Option<Detacher>,
    pub requires: Vec<String>,
    pub wanted_by: Vec<String>,
    pub type_id_requests: Vec<TypeIDRequestManifest>,
}

impl Manifest<SystemDefinition> for SystemManifest {
    type Error = SystemDefinitionError;

    unsafe fn checked_convert(value: SystemDefinition) -> Result<Self, SystemDefinitionError> {
        unsafe { value.validate()? };
        let name = unsafe { value.name() }.expect("validated system definitions have valid names");
        let string = |pointer| {
            unsafe { validate_string(pointer, str::to_owned) }
                .expect("validated system definitions have valid names")
        };

        let requires = if value.requires_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(value.requires, value.requires_count) }
        };
        let wanted_by = if value.wanted_by_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(value.wanted_by, value.wanted_by_count) }
        };
        let type_id_requests = if value.type_id_request_count == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(value.type_id_requests, value.type_id_request_count)
            }
        };
        let type_id_requests = type_id_requests
            .iter()
            .map(|&request| unsafe { TypeIDRequestManifest::checked_convert(request) })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| SystemDefinitionError::TypeIDRequestInvalid(name.clone(), error))?;

        Ok(Self {
            name,
            attacher: value.attacher,
            runner: value
                .runner
                .expect("validated system definitions have a runner"),
            detacher: value.detacher,
            requires: requires.iter().map(|&name| string(name)).collect(),
            wanted_by: wanted_by.iter().map(|&name| string(name)).collect(),
            type_id_requests,
        })
    }
}
