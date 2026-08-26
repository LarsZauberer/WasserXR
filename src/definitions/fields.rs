//! This module provides a FieldDescription for AssetFields and ComponentFields

use std::ffi::{c_char, c_void};

use crate::definitions::{
    Definition,
    error::{AssetFieldDefinitionError, ComponentFieldDefinitionError},
};
use crate::utils::ffi::validate_string;

/// Function to get from a component or asset a pointer to the actual field data. This is the way
/// wasserxr provides access a field.
///
/// # Safety
///
/// The callback must only be called with a pointer to the component or asset that owns this field.
/// The returned pointer must point to that field and is only valid while its owner is alive.
pub type Getter = unsafe extern "C" fn(ptr: *const c_void) -> *mut c_void;

/// Provides a function for a component field that serializes the data into binary data.
// TODO: TBD the final return type here
///
/// # Safety
///
/// `ptr` may be null. If it is non-null, it must point to a live component instance returned by
/// the [`Creator`](crate::definitions::components::Creator) declared by the owning
/// [`ComponentDefinition`](crate::definitions::components::ComponentDefinition), and it must
/// remain valid for the duration of the call.
pub type Serializer = unsafe extern "C" fn(ptr: *const c_void);

/// Provides a function for a component field to turn binary data into the corresponding field data.
// TODO: TBD the final argument type here
///
/// # Safety
///
/// `ptr` may be null. If it is non-null, it must point to a live component instance returned by
/// the [`Creator`](crate::definitions::components::Creator) declared by the owning
/// [`ComponentDefinition`](crate::definitions::components::ComponentDefinition), and it must
/// remain valid for the duration of the call. The callback must only access the field using its
/// declared type.
pub type Deserializer = unsafe extern "C" fn(ptr: *const c_void);

/// This is a definition of a field for a component. It contains the name of the field, a typehint
/// and a access/serialization information/permissions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ComponentFieldDefinition {
    name: *const c_char,

    // Access
    getter: Option<Getter>,
    mutable: i32,

    // Serialization
    serializer: Option<Serializer>,
    serializable: i32,
    deserializer: Option<Deserializer>,
    deserializable: i32,
}

impl Definition for ComponentFieldDefinition {
    type Error = ComponentFieldDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };

        if self.mutable != 0 && self.getter.is_none() {
            return Err(ComponentFieldDefinitionError::MutableButNoGetter(name));
        }

        if self.serializable != 0 && self.serializer.is_none() {
            return Err(ComponentFieldDefinitionError::SerializableButNoSerializer(
                name,
            ));
        }

        if self.deserializable != 0 && self.deserializer.is_none() {
            return Err(ComponentFieldDefinitionError::DeserializableButNoDeserializer(name));
        }

        Ok(())
    }
}

impl ComponentFieldDefinition {
    pub(crate) unsafe fn name(&self) -> Result<String, ComponentFieldDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}

/// This is a definition of a field for an asset. It is pretty much similar to the
/// [`wasserxr::definitions::fields::ComponentFieldDescription`] but instead has less functionality
/// that provide mutability or serialization for that field.
///
/// The name and getter are represented as C-compatible values so this descriptor can cross the
/// plugin boundary.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetFieldDefinition {
    name: *const c_char,
    getter: Option<Getter>,
}

impl Definition for AssetFieldDefinition {
    type Error = AssetFieldDefinitionError;

    /// # Safety
    ///
    /// This implementation has no additional safety requirements beyond those of
    /// [`Definition::validate`].
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        if self.getter.is_none() {
            return Err(AssetFieldDefinitionError::GetterIsNull(name));
        }
        Ok(())
    }
}

impl AssetFieldDefinition {
    unsafe fn name(&self) -> Result<String, AssetFieldDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}

#[cfg(test)]
mod component_field_tests {
    use rstest::{fixture, rstest};

    use super::*;

    unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn serializer(_: *const c_void) {}

    unsafe extern "C" fn deserializer(_: *const c_void) {}

    #[fixture]
    fn field() -> ComponentFieldDefinition {
        static NAME: &[u8] = b"position\0";

        ComponentFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
            mutable: 0,
            serializer: Some(serializer),
            serializable: 0,
            deserializer: Some(deserializer),
            deserializable: 0,
        }
    }

    #[rstest]
    fn validates_field(field: ComponentFieldDefinition) {
        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_mutable_field_without_getter(mut field: ComponentFieldDefinition) {
        field.getter = None;
        field.mutable = 1;

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::MutableButNoGetter(
                "position".to_owned()
            ))
        );
    }

    #[rstest]
    fn rejects_serializable_field_without_serializer(mut field: ComponentFieldDefinition) {
        field.serializer = None;
        field.serializable = 1;

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::SerializableButNoSerializer(
                "position".to_owned()
            ))
        );
    }

    #[rstest]
    fn rejects_deserializable_field_without_deserializer(mut field: ComponentFieldDefinition) {
        field.deserializer = None;
        field.deserializable = 1;

        assert_eq!(
            unsafe { field.validate() },
            Err(
                ComponentFieldDefinitionError::DeserializableButNoDeserializer(
                    "position".to_owned()
                )
            )
        );
    }

    #[rstest]
    fn rejects_null_name(mut field: ComponentFieldDefinition) {
        field.name = std::ptr::null();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsNull)
        );
    }

    #[rstest]
    fn rejects_invalid_utf8_name(mut field: ComponentFieldDefinition) {
        let name = [0xff_u8, 0];
        field.name = name.as_ptr().cast();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsNotUtf8)
        );
    }

    #[rstest]
    fn rejects_empty_name(mut field: ComponentFieldDefinition) {
        let name = [0_u8];
        field.name = name.as_ptr().cast();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsEmpty)
        );
    }
}

#[cfg(test)]
mod asset_field_tests {
    use rstest::{fixture, rstest};

    use super::*;

    unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[fixture]
    fn field() -> AssetFieldDefinition {
        static NAME: &[u8] = b"material\0";

        AssetFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
        }
    }

    #[rstest]
    fn validates_field(field: AssetFieldDefinition) {
        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_missing_getter(mut field: AssetFieldDefinition) {
        field.getter = None;

        assert_eq!(
            unsafe { field.validate() },
            Err(AssetFieldDefinitionError::GetterIsNull(
                "material".to_owned()
            ))
        );
    }
}
