//! Utility helpers shared by WasserXR runtime modules.

/// Foreign-function interface helpers.
pub mod ffi;

/// Rendering and parsing helpers for primitive field data.
pub mod field_render;

/// Filesystem path resolution helpers for enhanced asset loading
pub mod paths;

/// Simple Ring Buffer of fixed size
pub mod ring;

/// Rotation-buffer utilities; see [`rotation_buffer`].
pub mod rotation_buffer;

/// Utility struct to define version numbers
pub mod version;
