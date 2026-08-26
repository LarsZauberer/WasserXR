//! Utility helpers shared by WasserXR runtime modules.

/// Foreign-function interface helpers.
pub mod ffi;

/// Filesystem path resolution helpers for enhanced asset loading
pub mod paths;

/// Simple Ring Buffer of fixed size
pub mod ring;

/// Simple module which carries macros for internal use in WasserXR
pub(crate) mod macros;

/// Rotation-buffer utilities; see [`rotation_buffer`].
pub mod rotation_buffer;

/// Map abstraction utility
pub mod storage_backend;

/// Utility struct to define version numbers
pub mod version;
