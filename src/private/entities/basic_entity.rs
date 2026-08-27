use super::Entity;

/// This is a basic implementation of an [`crate::private::entities::Entity`]
#[derive(Debug, Default)]
pub(crate) struct BasicEntity {}

impl Entity for BasicEntity {}
