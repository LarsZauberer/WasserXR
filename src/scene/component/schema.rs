use std::collections::HashMap;

use crate::{
    error::ComponentError,
    scene::component::{
        field::{Field, Getter, Setter},
        field_type::FieldType,
    },
};

#[derive(Default)]
pub struct Schema {
    fields: HashMap<String, Field>,
}

impl Schema {
    pub(crate) fn new() -> Self {
        Schema::default()
    }

    pub fn add_field(
        &mut self,
        id: String,
        type_hint: FieldType,
        getter: Option<Getter>,
        setter: Option<Setter>,
    ) {
        let field = Field::new(type_hint, getter, setter);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn get_setter(&self, id: &str) -> Result<Setter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_setter(),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn get_fields(&self) -> Vec<&String> {
        self.fields.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    unsafe extern "C" fn test_getter(_data: *const c_void) -> *const c_void {
        std::ptr::null()
    }

    unsafe extern "C" fn test_setter(_data: *mut c_void, _value: *const c_void) {}

    #[test]
    fn schema_add_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            Some(test_getter),
            Some(test_setter),
        );

        let fields = schema.get_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "health");
    }

    #[test]
    fn schema_get_getter_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            Some(test_getter),
            Some(test_setter),
        );

        assert_eq!(
            schema.get_getter("health").unwrap() as usize,
            test_getter as *const () as usize
        );
    }

    #[test]
    fn schema_get_getter_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.get_getter("health"),
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn schema_get_setter_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            Some(test_getter),
            Some(test_setter),
        );

        assert_eq!(
            schema.get_setter("health").unwrap() as usize,
            test_setter as *const () as usize
        );
    }

    #[test]
    fn schema_get_setter_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.get_setter("health"),
            Err(ComponentError::FieldNotFound)
        );
    }
}
