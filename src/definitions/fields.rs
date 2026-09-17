//! This module provides a FieldDescription for AssetFields and ComponentFields

use std::ffi::{c_char, c_void};

use crate::definitions::{
    Definition,
    error::{AssetFieldDefinitionError, ComponentFieldDefinitionError},
};
use crate::utils::ffi::validate_string;

/// Primitive type used to render and parse field data across the C ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TypeHint {
    I8 = 0,
    I16 = 1,
    I32 = 2,
    I64 = 3,
    I128 = 4,
    Isize = 5,
    U8 = 6,
    U16 = 7,
    U32 = 8,
    U64 = 9,
    U128 = 10,
    Usize = 11,
    F32 = 12,
    F64 = 13,
    Char = 14,
    Boolean = 15,
}

impl TryFrom<u32> for TypeHint {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::I8),
            1 => Ok(Self::I16),
            2 => Ok(Self::I32),
            3 => Ok(Self::I64),
            4 => Ok(Self::I128),
            5 => Ok(Self::Isize),
            6 => Ok(Self::U8),
            7 => Ok(Self::U16),
            8 => Ok(Self::U32),
            9 => Ok(Self::U64),
            10 => Ok(Self::U128),
            11 => Ok(Self::Usize),
            12 => Ok(Self::F32),
            13 => Ok(Self::F64),
            14 => Ok(Self::Char),
            15 => Ok(Self::Boolean),
            value => Err(value),
        }
    }
}

impl From<TypeHint> for u32 {
    fn from(value: TypeHint) -> Self {
        value as Self
    }
}

/// Function to get from a component or asset a pointer to the actual field
/// data. This is the way wasserxr provides access a field.
///
/// # Safety
///
/// The callback must only be called with a pointer to the component or asset
/// that owns this field. The returned pointer must point to that field and is
/// only valid while its owner is alive.
pub type Getter = unsafe extern "C" fn(ptr: *const c_void) -> *mut c_void;

/// Provides a function for a component field that serializes the data into
/// binary data.
// TODO: TBD the final return type here
///
/// # Safety
///
/// `ptr` may be null. If it is non-null, it must point to a live component
/// instance returned by
/// the [`Creator`](crate::definitions::components::Creator) declared by the
/// owning
/// [`ComponentDefinition`](crate::definitions::components::ComponentDefinition), and it must
/// remain valid for the duration of the call.
pub type Serializer = unsafe extern "C" fn(ptr: *const c_void);

/// Provides a function for a component field to turn binary data into the
/// corresponding field data.
// TODO: TBD the final argument type here
///
/// # Safety
///
/// `ptr` may be null. If it is non-null, it must point to a live component
/// instance returned by
/// the [`Creator`](crate::definitions::components::Creator) declared by the
/// owning
/// [`ComponentDefinition`](crate::definitions::components::ComponentDefinition), and it must
/// remain valid for the duration of the call. The callback must only access the
/// field using its declared type.
pub type Deserializer = unsafe extern "C" fn(ptr: *const c_void);

/// This is a definition of a field for a component. It contains the name of the
/// field, a typehint and a access/serialization information/permissions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ComponentFieldDefinition {
    pub name: *const c_char,
    /// A [`TypeHint`] discriminant, validated when the definition is loaded.
    pub type_hint: u32,

    // Access
    pub getter: Option<unsafe extern "C" fn(ptr: *const c_void) -> *mut c_void>,
    pub mutable: i32,

    // Serialization
    pub serializer: Option<unsafe extern "C" fn(ptr: *const c_void)>,
    pub deserializer: Option<unsafe extern "C" fn(ptr: *const c_void)>,
}

impl Definition for ComponentFieldDefinition {
    type Error = ComponentFieldDefinitionError;

    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };

        TypeHint::try_from(self.type_hint)
            .map_err(ComponentFieldDefinitionError::InvalidTypeHint)?;

        if self.mutable != 0 && self.getter.is_none() {
            return Err(ComponentFieldDefinitionError::MutableButNoGetter(name));
        }

        Ok(())
    }
}

impl ComponentFieldDefinition {
    /// Returns the validated field name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, ComponentFieldDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}

/// This is a definition of a field for an asset. It is pretty much similar to
/// the [`ComponentFieldDefinition`] but instead has less functionality
/// that provide mutability or serialization for that field.
///
/// The name and getter are represented as C-compatible values so this
/// descriptor can cross the plugin boundary.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetFieldDefinition {
    pub name: *const c_char,
    /// A [`TypeHint`] discriminant, validated when the definition is loaded.
    pub type_hint: u32,
    pub getter: Option<unsafe extern "C" fn(ptr: *const c_void) -> *mut c_void>,
}

impl Definition for AssetFieldDefinition {
    type Error = AssetFieldDefinitionError;

    /// # Safety
    ///
    /// This implementation has no additional safety requirements beyond those
    /// of [`Definition::validate`].
    unsafe fn validate(&self) -> Result<(), Self::Error> {
        let name = unsafe { self.name()? };
        TypeHint::try_from(self.type_hint).map_err(AssetFieldDefinitionError::InvalidTypeHint)?;
        if self.getter.is_none() {
            return Err(AssetFieldDefinitionError::GetterIsNull(name));
        }
        Ok(())
    }
}

impl AssetFieldDefinition {
    /// Returns the validated field name as an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.name` must point to a valid, NUL-terminated C string for the
    /// duration of the call.
    pub(crate) unsafe fn name(&self) -> Result<String, AssetFieldDefinitionError> {
        unsafe { validate_string(self.name, str::to_owned) }.map_err(Into::into)
    }
}
