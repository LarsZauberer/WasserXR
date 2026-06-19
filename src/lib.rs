extern crate self as wasserxr;

pub mod error;
pub mod scene;

#[cfg(test)]
mod r#macro;

pub use scene::Scene;
pub use uuid::Uuid;
pub use wasserxr_macros::{Component, System};
