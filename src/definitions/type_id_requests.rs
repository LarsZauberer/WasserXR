//! Requests for resolving WasserXR type IDs.

use std::ffi::c_char;

use crate::{definitions::error::TypeIDRequestError, utils::ffi::validate_string};

/// Describes the names needed to resolve one of WasserXR's type IDs.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum TypeIDRequests {
    ComponentTypeID {
        component: *const c_char,
    },
    FieldTypeID {
        component: *const c_char,
        field: *const c_char,
    },
    AssetTypeID {
        asset: *const c_char,
    },
    AssetFieldTypeID {
        asset: *const c_char,
        field: *const c_char,
    },
}

impl TypeIDRequests {
    /// Validates every name needed to resolve this request.
    ///
    /// # Safety
    ///
    /// Every pointer in this request must point to a valid, NUL-terminated C
    /// string for the duration of the call.
    pub unsafe fn validate(&self) -> Result<(), TypeIDRequestError> {
        match *self {
            Self::ComponentTypeID { component } => unsafe {
                validate_string(component, |_| ()).map_err(TypeIDRequestError::Component)
            },
            Self::FieldTypeID { component, field } => {
                unsafe { validate_string(component, |_| ()) }
                    .map_err(TypeIDRequestError::Component)?;
                unsafe { validate_string(field, |_| ()) }.map_err(TypeIDRequestError::Field)
            }
            Self::AssetTypeID { asset } => unsafe {
                validate_string(asset, |_| ()).map_err(TypeIDRequestError::Asset)
            },
            Self::AssetFieldTypeID { asset, field } => {
                unsafe { validate_string(asset, |_| ()) }.map_err(TypeIDRequestError::Asset)?;
                unsafe { validate_string(field, |_| ()) }.map_err(TypeIDRequestError::Field)
            }
        }
    }
}
