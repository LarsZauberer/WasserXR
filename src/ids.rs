//! Scene and manifest handles used throughout WasserXR.

use slotmap::{Key, KeyData, new_key_type};

new_key_type! {
/// EntityID is a cheap copyable handle for entities. It uniquely identifies an entity
/// within a Scene. It is not a globally unique identifier across multiple scenes (if you are
/// maintaining multiple scenes)
pub struct EntityID;

/// Handle for a loaded plugin. It describes a plugin uniquely to the scene and cannot like the [`EntityID`] be used
/// in different scenes. This behavior is not supported.
pub struct PluginID;

/// Handle that is cheap to copy and address a component. It is only unique within a
/// single entity and cannot be used across multiple entity.
pub struct ComponentID;

/// Handle that is cheap to copy and address a field in a component. It is only unique within a
/// single entity and component. It is not unique across multiple components.
pub struct FieldID;

/// Handle that is cheap to copy to address assets. An AssetID is unique to an asset type and it's
/// data string. Meaning two assets of the same type but have different data strings will have
/// different ID's
pub struct AssetID;

/// Handle that is cheap to copy and uniquely identifies a field inside of an asset. It is only
/// unique inside of a single Asset and it's data string.
pub struct AssetFieldID;

/// Handle for a component type. It is unique within its plugin manifest, but
/// cannot be used across different plugin manifests.
pub struct ComponentTypeID;

/// Handle for a component field type. It is unique within its component type
/// manifest, but cannot be used across different component type manifests.
pub struct FieldTypeID;

/// Handle for an asset field type. It is unique within its asset type manifest,
/// but cannot be used across different asset type manifests.
pub struct AssetFieldTypeID;

/// Handle for an asset type. It is unique within its plugin manifest, but
/// cannot be used across different plugin manifests.
pub struct AssetTypeID;

/// Handle for a concrete system. It is unique within its scene, but cannot be
/// used across different scenes.
pub struct SystemID;

/// Handle for a system type. It is unique within its plugin manifest, but
/// cannot be used across different plugin manifests.
pub struct SystemTypeID;
}

/// A resolved type ID passed to a system callback.
///
/// The variant matches the corresponding
/// [`crate::definitions::type_id_requests::TypeIDRequests`] entry. The inner
/// value uses SlotMap's stable FFI representation rather than exposing a
/// Rust-specific key layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TypeID {
    ComponentTypeID(u64),
    FieldTypeID(u64),
    AssetTypeID(u64),
    AssetFieldTypeID(u64),
}

impl From<ComponentTypeID> for TypeID {
    fn from(id: ComponentTypeID) -> Self {
        Self::ComponentTypeID(id.data().as_ffi())
    }
}

impl From<FieldTypeID> for TypeID {
    fn from(id: FieldTypeID) -> Self {
        Self::FieldTypeID(id.data().as_ffi())
    }
}

impl From<AssetTypeID> for TypeID {
    fn from(id: AssetTypeID) -> Self {
        Self::AssetTypeID(id.data().as_ffi())
    }
}

impl From<AssetFieldTypeID> for TypeID {
    fn from(id: AssetFieldTypeID) -> Self {
        Self::AssetFieldTypeID(id.data().as_ffi())
    }
}

impl TryFrom<TypeID> for ComponentTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::ComponentTypeID(value) => Ok(KeyData::from_ffi(value).into()),
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for FieldTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::FieldTypeID(value) => Ok(KeyData::from_ffi(value).into()),
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for AssetTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::AssetTypeID(value) => Ok(KeyData::from_ffi(value).into()),
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for AssetFieldTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::AssetFieldTypeID(value) => Ok(KeyData::from_ffi(value).into()),
            other => Err(other),
        }
    }
}
