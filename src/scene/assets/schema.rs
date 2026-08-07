use std::{collections::HashMap, ffi::c_void};

use crate::scene::{
    assets::{
        AssetError,
        field::{Field, Getter},
    },
    component::FieldType,
};

/// Runtime schema for one asset type.
///
/// Asset schemas are built from validated manifest descriptors and later used
/// by `Scene::asset_query`. They are not exposed across the plugin ABI.
#[derive(Clone, Default)]
pub(crate) struct Schema {
    fields: HashMap<String, Field>,
}

impl Schema {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Schema::default()
    }

    /// Adds one already-validated manifest field to the internal schema.
    pub(crate) fn add_field(&mut self, id: String, type_hint: FieldType, getter: Option<Getter>) {
        let field = Field::new(type_hint, getter);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, AssetError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(),
            None => Err(AssetError::FieldNotFound),
        }
    }

    pub(crate) unsafe fn get_field_ptr(
        &self,
        id: &str,
        data: *mut c_void,
    ) -> Result<*mut c_void, AssetError> {
        let getter = self.get_getter(id)?;
        Ok(unsafe { getter(data) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[test]
    fn schema_get_getter_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field("content".to_owned(), FieldType::String, Some(test_getter));

        assert_eq!(
            schema.get_getter("content").unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn schema_get_getter_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(schema.get_getter("content"), Err(AssetError::FieldNotFound));
    }
}
