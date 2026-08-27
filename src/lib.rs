use slotmap::{SlotMap, new_key_type};

pub mod utils;

pub mod definitions;
pub mod entity;
pub mod manifests;
pub mod scene;

new_key_type! {pub struct EntityID;}
pub type WXREntityID = EntityID;
pub type WXREntity = entity::basic_entity::BasicEntity;
pub type WXREntityStorage = SlotMap<EntityID, WXREntity>;
pub type WXRScene = scene::basic_scene::BasicScene<WXREntityStorage>;
