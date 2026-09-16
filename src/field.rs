//! Public component-field query types.

/// The lock requested for an entire component during a query callback.
///
/// This controls access even though both modes expose `*mut c_void` pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRequest {
    /// Shared access. The callback must not write through the pointers.
    Read,
    /// Exclusive access. Requested fields must also be declared mutable.
    Write,
}
