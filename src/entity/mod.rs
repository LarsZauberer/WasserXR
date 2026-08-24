//! This module defines an entity

pub mod basic_entity;

/// Entities are a fundamental object in the ECS architecture. This trait shows what operations an
/// entity needs to fulfill if it wants to be used as an ECS entity.
///
/// Such an entity will carry all the components.
pub trait Entity {}
