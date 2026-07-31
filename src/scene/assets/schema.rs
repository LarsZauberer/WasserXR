use std::{collections::HashMap, ffi::c_void};

use crate::{
    error::AssetError,
    scene::{
        assets::field::{Field, Getter},
        component::FieldType,
    },
};

/// Runtime schema for one asset type.
///
/// Asset schemas are filled by generated `wxr_asset_schema_<Asset>` functions
/// and later used by `Scene::asset_query`.
#[derive(Default)]
pub struct Schema {
    fields: HashMap<String, Field>,
}

impl Schema {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Schema::default()
    }

    /// Registers one asset field in the schema.
    pub fn add_field(&mut self, id: String, type_hint: FieldType, getter: Option<Getter>) {
        let field = Field::new(type_hint, getter);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, AssetError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(),
            None => Err(AssetError::FieldNotFound),
        }
    }

    /// Returns the type hint for a registered asset field.
    pub fn get_field_type(&self, id: &str) -> Result<FieldType, AssetError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.get_type()),
            None => Err(AssetError::FieldNotFound),
        }
    }

    /// Returns all registered field ids.
    pub fn get_fields(&self) -> Vec<&String> {
        self.fields.keys().collect()
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
    use crate::testing_plugin_fixture::null_getter;
    use rstest::rstest;

    fn schema_with_content() -> Schema {
        let mut schema = Schema::new();
        schema.add_field("content".to_owned(), FieldType::String, Some(null_getter));
        schema
    }

    #[rstest]
    fn schema_add_field_registers_field_name_type_and_getter() {
        let schema = schema_with_content();

        assert_eq!(schema.get_fields(), vec!["content"]);
        assert_eq!(schema.get_field_type("content"), Ok(FieldType::String));
        assert_eq!(
            schema.get_getter("content").unwrap() as usize,
            null_getter as *const () as usize
        );
    }

    #[rstest]
    fn schema_get_getter_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(schema.get_getter("content"), Err(AssetError::FieldNotFound));
    }
}
