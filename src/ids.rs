//! Scene and manifest handles used throughout WasserXR.

use slotmap::{Key, KeyData, new_key_type};

new_key_type! {
    /// Storage slot for an entity within a scene.
    pub(crate) struct EntitySlot;
    /// Storage slot for a loaded plugin within a scene.
    pub(crate) struct PluginSlot;
    /// Storage slot for a component within an entity.
    pub(crate) struct ComponentSlot;
    /// Storage slot for a field within a concrete component.
    pub(crate) struct FieldSlot;
    /// Storage slot for a loaded asset within a scene.
    pub(crate) struct AssetSlot;
    /// Storage slot for a field within a loaded asset.
    pub(crate) struct AssetFieldSlot;
    /// Storage slot for a component type within a plugin manifest.
    pub(crate) struct ComponentTypeSlot;
    /// Storage slot for a field type within a component manifest.
    pub(crate) struct FieldTypeSlot;
    /// Storage slot for an asset type within a plugin manifest.
    pub(crate) struct AssetTypeSlot;
    /// Storage slot for a field type within an asset manifest.
    pub(crate) struct AssetFieldTypeSlot;
    /// Storage slot for a concrete system within a scene.
    pub(crate) struct SystemSlot;
    /// Storage slot for a system type within a plugin manifest.
    pub(crate) struct SystemTypeSlot;
}

/// Handle for an entity within a scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityID(pub(crate) EntitySlot);

/// Handle for a loaded plugin within a scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginID(pub(crate) PluginSlot);

/// Handle for a component type within a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentTypeID(pub(crate) PluginSlot, pub(crate) ComponentTypeSlot);

/// Handle for a field type within a component type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldTypeID(
    pub(crate) PluginSlot,
    pub(crate) ComponentTypeSlot,
    pub(crate) FieldTypeSlot,
);

/// Handle for an asset type within a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetTypeID(pub(crate) PluginSlot, pub(crate) AssetTypeSlot);

/// Handle for a field type within an asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetFieldTypeID(
    pub(crate) PluginSlot,
    pub(crate) AssetTypeSlot,
    pub(crate) AssetFieldTypeSlot,
);

/// Handle for a system type within a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTypeID(pub(crate) PluginSlot, pub(crate) SystemTypeSlot);

/// Handle for a component attached to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentID(pub(crate) EntitySlot, pub(crate) ComponentSlot);

/// Handle for a field within a concrete component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldID(
    pub(crate) EntitySlot,
    pub(crate) ComponentSlot,
    pub(crate) FieldSlot,
);

/// Handle for a loaded asset of a particular type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetID(
    pub(crate) PluginSlot,
    pub(crate) AssetTypeSlot,
    pub(crate) AssetSlot,
);

/// Handle for a field within a loaded asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetFieldID(
    pub(crate) PluginSlot,
    pub(crate) AssetTypeSlot,
    pub(crate) AssetSlot,
    pub(crate) AssetFieldSlot,
);

/// Handle for a concrete system of a particular type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemID(
    pub(crate) PluginSlot,
    pub(crate) SystemTypeSlot,
    pub(crate) SystemSlot,
);

/// A resolved type ID passed to a system callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TypeID {
    ComponentTypeID(u64, u64),
    FieldTypeID(u64, u64, u64),
    AssetTypeID(u64, u64),
    AssetFieldTypeID(u64, u64, u64),
}

fn ffi(key: impl Key) -> u64 {
    key.data().as_ffi()
}

fn key<T: From<KeyData>>(value: u64) -> T {
    KeyData::from_ffi(value).into()
}

impl From<ComponentTypeID> for TypeID {
    fn from(ComponentTypeID(plugin, component): ComponentTypeID) -> Self {
        Self::ComponentTypeID(ffi(plugin), ffi(component))
    }
}

impl From<FieldTypeID> for TypeID {
    fn from(FieldTypeID(plugin, component, field): FieldTypeID) -> Self {
        Self::FieldTypeID(ffi(plugin), ffi(component), ffi(field))
    }
}

impl From<AssetTypeID> for TypeID {
    fn from(AssetTypeID(plugin, asset): AssetTypeID) -> Self {
        Self::AssetTypeID(ffi(plugin), ffi(asset))
    }
}

impl From<AssetFieldTypeID> for TypeID {
    fn from(AssetFieldTypeID(plugin, asset, field): AssetFieldTypeID) -> Self {
        Self::AssetFieldTypeID(ffi(plugin), ffi(asset), ffi(field))
    }
}

impl TryFrom<TypeID> for ComponentTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::ComponentTypeID(plugin, component) => Ok(Self(key(plugin), key(component))),
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for FieldTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::FieldTypeID(plugin, component, field) => {
                Ok(Self(key(plugin), key(component), key(field)))
            }
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for AssetTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::AssetTypeID(plugin, asset) => Ok(Self(key(plugin), key(asset))),
            other => Err(other),
        }
    }
}

impl TryFrom<TypeID> for AssetFieldTypeID {
    type Error = TypeID;

    fn try_from(id: TypeID) -> Result<Self, Self::Error> {
        match id {
            TypeID::AssetFieldTypeID(plugin, asset, field) => {
                Ok(Self(key(plugin), key(asset), key(field)))
            }
            other => Err(other),
        }
    }
}
