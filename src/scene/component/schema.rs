use std::{collections::HashMap, ffi::c_void};

use crate::{
    error::ComponentError,
    scene::{
        component::{
            field::{Deserializer, Field, Getter, Serializer},
            field_type::FieldType,
        },
        logging::LogManager,
    },
};

pub struct Schema {
    fields: HashMap<String, Field>,
    log_manager: LogManager,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
            log_manager: LogManager::new("WasserXR".to_owned()),
        }
    }
}

impl Schema {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Schema::default()
    }

    pub(crate) fn with_logger(log_manager: LogManager) -> Self {
        Self {
            fields: HashMap::new(),
            log_manager,
        }
    }

    pub fn add_field(
        &mut self,
        id: String,
        type_hint: FieldType,
        getter: Option<Getter>,
        mutable: bool,
        serializer: Option<Serializer>,
        deserializer: Option<Deserializer>,
    ) {
        let field = Field::new(type_hint, getter, mutable, serializer, deserializer);
        crate::debug!(self.log_manager, "Schema field `{}` added", id);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(&self.log_manager),
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for read",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn is_mutable(&self, id: &str) -> Result<bool, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.is_mutable()),
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for mutability lookup",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_serializer(&self, id: &str) -> Result<Serializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_serializer(&self.log_manager),
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for serialization",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_deserializer(&self, id: &str) -> Result<Deserializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_deserializer(&self.log_manager),
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for deserialization",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_field_type(&self, id: &str) -> Result<FieldType, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.get_type()),
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for type lookup",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) unsafe fn render_field(
        &self,
        id: &str,
        data: *mut c_void,
        logger: &str,
    ) -> Result<String, ComponentError> {
        match self.fields.get(id) {
            Some(field) => {
                let getter = field.get_getter(&self.log_manager)?;
                self.log_manager.set_logger(logger.to_owned());
                let field_ptr = unsafe { getter(data) };
                self.log_manager.set_logger("WasserXR".to_owned());
                unsafe { field.render(field_ptr) }
            }
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for render",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) unsafe fn parse_field(
        &self,
        id: &str,
        data: *mut c_void,
        input: &str,
        logger: &str,
    ) -> Result<(), ComponentError> {
        match self.fields.get(id) {
            Some(field) => {
                let getter = field.get_getter(&self.log_manager)?;
                self.log_manager.set_logger(logger.to_owned());
                let field_ptr = unsafe { getter(data) };
                self.log_manager.set_logger("WasserXR".to_owned());
                unsafe { field.parse(field_ptr, input) }
            }
            None => {
                crate::debug!(
                    self.log_manager,
                    "Schema field `{}` was not found for parse",
                    id
                );
                Err(ComponentError::FieldNotFound)
            }
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
