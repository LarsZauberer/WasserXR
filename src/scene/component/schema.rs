use std::{collections::HashMap, ffi::c_void};

use crate::scene::component::{
    ComponentError,
    field::{Deserializer, Field, Getter, Serializer},
    field_type::FieldType,
};

/// Runtime schema for one component type.
///
/// A component schema maps validated manifest field ids to host-owned metadata
/// and typed callbacks. It is never exposed across the plugin ABI.
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
    pub(crate) fn add_field(
        &mut self,
        id: String,
        type_hint: FieldType,
        getter: Option<Getter>,
        mutable: bool,
        serializer: Option<Serializer>,
        deserializer: Option<Deserializer>,
    ) {
        let field = Field::new(type_hint, getter, mutable, serializer, deserializer);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn is_mutable(&self, id: &str) -> Result<bool, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.is_mutable()),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn is_string_parsable(&self, id: &str) -> Result<bool, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.is_string_parsable()),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn get_serializer(&self, id: &str) -> Result<Serializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_serializer(),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn get_deserializer(&self, id: &str) -> Result<Deserializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_deserializer(),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) fn get_field_type(&self, id: &str) -> Result<FieldType, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.get_type()),
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) unsafe fn render_field(
        &self,
        id: &str,
        data: *mut c_void,
    ) -> Result<String, ComponentError> {
        match self.fields.get(id) {
            Some(field) => {
                let getter = field.get_getter()?;
                let field_ptr = unsafe { getter(data) };
                unsafe { field.render(field_ptr) }
            }
            None => Err(ComponentError::FieldNotFound),
        }
    }

    pub(crate) unsafe fn parse_field(
        &self,
        id: &str,
        data: *mut c_void,
        input: &str,
    ) -> Result<(), ComponentError> {
        match self.fields.get(id) {
            Some(field) => {
                let getter = field.get_getter()?;
                let field_ptr = unsafe { getter(data) };
                unsafe { field.parse(field_ptr, input) }
            }
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

    unsafe extern "C" fn test_getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn test_serializer(
        _data: *const c_void,
    ) -> crate::scene::component::SerializedBytes {
        crate::scene::component::SerializedBytes::from_vec(vec![1])
    }

    unsafe extern "C" fn test_deserializer(
        _data: *mut c_void,
        _value: crate::scene::component::SerializedBytes,
    ) {
    }

    #[test]
    fn schema_add_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::I64,
            Some(test_getter),
            true,
            Some(test_serializer),
            Some(test_deserializer),
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
            FieldType::I64,
            Some(test_getter),
            false,
            None,
            None,
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
    fn schema_is_mutable_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::I64,
            Some(test_getter),
            true,
            None,
            None,
        );

        assert!(schema.is_mutable("health").unwrap());
    }

    #[test]
    fn schema_is_not_mutable_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::I64,
            Some(test_getter),
            false,
            None,
            None,
        );

        assert!(!schema.is_mutable("health").unwrap());
    }

    #[test]
    fn schema_is_mutable_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.is_mutable("health"),
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn schema_get_serializer_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::I64,
            Some(test_getter),
            false,
            Some(test_serializer),
            Some(test_deserializer),
        );

        assert_eq!(
            schema.get_serializer("health").unwrap() as usize,
            test_serializer as *const () as usize
        );
    }

    #[test]
    fn schema_get_deserializer_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::I64,
            Some(test_getter),
            false,
            Some(test_serializer),
            Some(test_deserializer),
        );

        assert_eq!(
            schema.get_deserializer("health").unwrap() as usize,
            test_deserializer as *const () as usize
        );
    }
}
