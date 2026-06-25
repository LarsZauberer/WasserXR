extern crate self as wasserxr;

pub mod error;
pub mod scene;
pub mod utils;

pub use uuid::Uuid;
pub use wasserxr_macros::{asset_type, asset_type_creator, attacher, component, detacher, system};
