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

/// Handle for a system. It is unique within its plugin manifest, but cannot be
/// used across different plugin manifests.
pub struct SystemID;
}
