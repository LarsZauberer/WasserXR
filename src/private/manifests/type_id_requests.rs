use crate::{
    definitions::{Definition, error::TypeIDRequestError, type_id_requests::TypeIDRequests},
    private::manifests::Manifest,
    utils::ffi::validate_string,
};

/// Rust-owned form of a type ID request from a system definition.
///
/// It retains the names required for later resolution without retaining any
/// pointers into plugin-owned C data.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TypeIDRequestManifest {
    ComponentTypeID { component: String },
    FieldTypeID { component: String, field: String },
    AssetTypeID { asset: String },
    AssetFieldTypeID { asset: String, field: String },
    FunctionTypeID { function: String },
}

impl Manifest<TypeIDRequests> for TypeIDRequestManifest {
    type Error = TypeIDRequestError;

    unsafe fn checked_convert(value: TypeIDRequests) -> Result<Self, TypeIDRequestError> {
        unsafe { value.validate()? };
        let string = |pointer| {
            unsafe { validate_string(pointer, str::to_owned) }
                .expect("validated type ID requests have valid names")
        };

        Ok(match value {
            TypeIDRequests::ComponentTypeID { component } => Self::ComponentTypeID {
                component: string(component),
            },
            TypeIDRequests::FieldTypeID { component, field } => Self::FieldTypeID {
                component: string(component),
                field: string(field),
            },
            TypeIDRequests::AssetTypeID { asset } => Self::AssetTypeID {
                asset: string(asset),
            },
            TypeIDRequests::AssetFieldTypeID { asset, field } => Self::AssetFieldTypeID {
                asset: string(asset),
                field: string(field),
            },
            TypeIDRequests::FunctionTypeID { function } => Self::FunctionTypeID {
                function: string(function),
            },
        })
    }
}
