use std::{collections::HashMap, ffi::c_void};

use crate::{
    error::AssetError,
    scene::{
        assets::field::{Field, Getter},
        component::FieldType,
    },
};

pub struct Schema {
    fields: HashMap<String, Field>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl Schema {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Schema::default()
    }

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

    pub fn get_field_type(&self, id: &str) -> Result<FieldType, AssetError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.get_type()),
            None => Err(AssetError::FieldNotFound),
        }
    }

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

    unsafe extern "C" fn test_getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    #[test]
    fn schema_add_field() {
        let mut schema = Schema::new();

        schema.add_field("content".to_owned(), FieldType::String, Some(test_getter));

        let fields = schema.get_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "content");
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

    #[test]
    fn schema_get_field_type() {
        let mut schema = Schema::new();

        schema.add_field("content".to_owned(), FieldType::String, Some(test_getter));

        assert_eq!(schema.get_field_type("content"), Ok(FieldType::String));
    }
}
