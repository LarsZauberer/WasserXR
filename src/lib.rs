extern crate self as wasserxr;

pub mod error;
pub mod scene;

pub use uuid::Uuid;
pub use wasserxr_macros::{component, system};
