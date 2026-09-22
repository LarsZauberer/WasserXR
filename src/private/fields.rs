use std::ffi::c_void;

use crate::{
    definitions::fields::{Deserializer, Getter, Serializer},
    errors::FieldError,
    field::AccessRequest,
    private::manifests::fields::{AssetFieldManifest, ComponentFieldManifest},
};

/// A field is the concrete implementation of a field in a concrete
/// [`Component`](crate::private::components::Component). It contains the raw
/// data structure that represents a field inside of a component. It groups
/// together all the important information of a field.
///
/// It's responsibility is to create it from a [`ComponentFieldManifest`] and be
/// concrete.
#[derive(Debug)]
pub(crate) struct ComponentField {
    name: String,
    getter: Option<Getter>,
    mutable: bool,
    serializer: Option<Serializer>,
    deserializer: Option<Deserializer>,
}

impl ComponentField {
    /// Checks whether a query may access this field without invoking its
    /// getter.
    pub(crate) fn validate_access(&self, access: AccessRequest) -> Result<(), FieldError> {
        if access == AccessRequest::Write && !self.mutable {
            return Err(FieldError::NotMutable);
        }
        self.getter.map(drop).ok_or(FieldError::NoGetter)
    }

    /// Get the field data from a component object
    pub(crate) fn get(&self, ptr: *const c_void) -> Result<*const c_void, FieldError> {
        match self.getter {
            Some(getter) => Ok(unsafe { getter(ptr) }),
            None => Err(FieldError::NoGetter),
        }
    }

    /// Get the field data from a component object but also checking if the data
    /// is allowed to be mutated
    pub(crate) fn get_mut(&self, ptr: *mut c_void) -> Result<*mut c_void, FieldError> {
        if !self.mutable {
            return Err(FieldError::NotMutable);
        }

        // This should be guaranteed by the invariant in the definition check but is
        // repeated here for security reasons.
        match self.getter {
            Some(getter) => Ok(unsafe { getter(ptr) }),
            None => Err(FieldError::NoGetter),
        }
    }

    /// Get the name of the field
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

impl From<&ComponentFieldManifest> for ComponentField {
    fn from(value: &ComponentFieldManifest) -> Self {
        ComponentField {
            name: value.name.clone(),
            getter: value.getter,
            mutable: value.mutable,
            serializer: value.serializer,
            deserializer: value.deserializer,
        }
    }
}

/// A concrete field implementation of an asset field. It is created from a
/// [`AssetFieldManifest`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AssetField {
    name: String,
    getter: Getter,
}

impl AssetField {
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get(&self, data: *const c_void) -> *const c_void {
        let ptr = unsafe { (self.getter)(data) };
        ptr.cast_const()
    }
}

impl From<&AssetFieldManifest> for AssetField {
    fn from(value: &AssetFieldManifest) -> Self {
        AssetField {
            name: value.name.clone(),
            getter: value.getter,
        }
    }
}
