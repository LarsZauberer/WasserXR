//! Public component-field query types.

/// The access requested for a component field. Any write access makes the
/// containing component exclusive for the complete query callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldAccess {
    Read,
    Write,
}
