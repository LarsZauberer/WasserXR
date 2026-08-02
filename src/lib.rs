//! WasserXR is the foundational ECS runtime for the WasserXR engine.
//!
//! WasserXR is built for dynamic, modular applications where components,
//! systems, and asset types can live in reloadable plugins. The main runtime
//! object is [`scene::Scene`]. A scene owns entities, components, systems,
//! assets, resources, and loaded plugins, and `Scene::tick` runs the active
//! systems.
//!
//! The full engine is usually split into two parts:
//!
//! - **WasserXR**: this crate, the foundational ECS and plugin runtime.
//! - **WasserXR-Core**: the standard set of engine components, systems, and
//!   asset types.
//!
//! A minimal application usually creates a scene, loads the core plugin, adds a
//! system, and ticks the scene:
//!
//! ```ignore
//! use wasserxr::scene::{logging::file_logger, Scene};
//!
//! fn main() {
//!     let mut scene = Scene::new();
//!     scene.register_callback_logger(file_logger);
//!
//!     scene
//!         .load_plugin("./libwasserxr_core.so".to_owned())
//!         .expect("Failed to load core plugin");
//!
//!     scene
//!         .add_system("console".to_owned(), 100)
//!         .expect("Failed to add system `console`");
//!
//!     while scene.tick() {}
//!     let _ = scene.reset();
//! }
//! ```
//!
//! This crate also re-exports the procedural macros used by Rust plugins, such
//! as [`component`], [`component_creator`], [`system`], [`attacher`],
//! [`detacher`], [`asset_type`], and [`asset_type_creator`].
//!
//! For tutorials and getting-started guides, see <https://wasserxr.com>.

extern crate self as wasserxr;

pub mod bindings;
pub mod scene;
/// Small utility modules used by the runtime and applications.
pub mod utils;

pub use uuid::Uuid;
pub use wasserxr_macros::{
    asset_type, asset_type_creator, attacher, component, component_creator, detacher, method,
    system,
};
