//! Public component-field query types.

/// The access requested for a component field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldAccess {
    Read,
    Write,
}
