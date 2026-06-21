use std::collections::HashMap;

use crate::{
    error::ComponentError,
    scene::component::{
        field::{Deserializer, Field, Getter, GetterMut, Mover, Serializer, Setter, Taker},
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
        getter_mut: Option<GetterMut>,
        setter: Option<Setter>,
        mover: Option<Mover>,
        taker: Option<Taker>,
        serializer: Option<Serializer>,
        deserializer: Option<Deserializer>,
    ) {
        let field = Field::new(
            type_hint,
            getter,
            getter_mut,
            setter,
            mover,
            taker,
            serializer,
            deserializer,
        );
        log::debug!("Schema field `{}` added", id);
        self.fields.insert(id, field);
    }

    pub(crate) fn get_getter(&self, id: &str) -> Result<Getter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter(),
            None => {
                log::debug!("Schema field `{}` was not found for read", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_getter_mut(&self, id: &str) -> Result<GetterMut, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_getter_mut(),
            None => {
                log::debug!("Schema field `{}` was not found for mutable read", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_setter(&self, id: &str) -> Result<Setter, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_setter(),
            None => {
                log::debug!("Schema field `{}` was not found for update", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_mover(&self, id: &str) -> Result<Mover, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_mover(),
            None => {
                log::debug!("Schema field `{}` was not found for move", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_taker(&self, id: &str) -> Result<Taker, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_taker(),
            None => {
                log::debug!("Schema field `{}` was not found for take", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_serializer(&self, id: &str) -> Result<Serializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_serializer(),
            None => {
                log::debug!("Schema field `{}` was not found for serialization", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_deserializer(&self, id: &str) -> Result<Deserializer, ComponentError> {
        match self.fields.get(id) {
            Some(field) => field.get_deserializer(),
            None => {
                log::debug!("Schema field `{}` was not found for deserialization", id);
                Err(ComponentError::FieldNotFound)
            }
        }
    }

    pub(crate) fn get_field_type(&self, id: &str) -> Result<FieldType, ComponentError> {
        match self.fields.get(id) {
            Some(field) => Ok(field.get_type()),
            None => {
                log::debug!("Schema field `{}` was not found for type lookup", id);
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

    unsafe extern "C" fn test_getter(_data: *const c_void) -> *const c_void {
        std::ptr::null()
    }

    unsafe extern "C" fn test_getter_mut(_data: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn test_setter(_data: *mut c_void, _value: *const c_void) {}

    unsafe extern "C" fn test_mover(_data: *mut c_void, _value: *mut c_void) {}

    unsafe extern "C" fn test_taker(_data: *mut c_void, _out: *mut c_void) {}

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
            FieldType::Long,
            Some(test_getter),
            Some(test_getter_mut),
            Some(test_setter),
            Some(test_mover),
            Some(test_taker),
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
            FieldType::Long,
            Some(test_getter),
            None,
            Some(test_setter),
            None,
            None,
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
    fn schema_get_getter_mut_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            None,
            Some(test_getter_mut),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            schema.get_getter_mut("health").unwrap() as usize,
            test_getter_mut as *const () as usize
        );
    }

    #[test]
    fn schema_get_getter_mut_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.get_getter_mut("health"),
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
            None,
            Some(test_setter),
            None,
            None,
            None,
            None,
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

    #[test]
    fn schema_get_mover_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            None,
            None,
            None,
            Some(test_mover),
            Some(test_taker),
            None,
            None,
        );

        assert_eq!(
            schema.get_mover("health").unwrap() as usize,
            test_mover as *const () as usize
        );
    }

    #[test]
    fn schema_get_mover_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.get_mover("health"),
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn schema_get_taker_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            None,
            None,
            None,
            Some(test_mover),
            Some(test_taker),
            None,
            None,
        );

        assert_eq!(
            schema.get_taker("health").unwrap() as usize,
            test_taker as *const () as usize
        );
    }

    #[test]
    fn schema_get_taker_for_missing_field() {
        let schema = Schema::new();

        assert_eq!(
            schema.get_taker("health"),
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn schema_get_serializer_for_existing_field() {
        let mut schema = Schema::new();

        schema.add_field(
            "health".to_owned(),
            FieldType::Long,
            Some(test_getter),
            None,
            Some(test_setter),
            None,
            None,
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
            FieldType::Long,
            Some(test_getter),
            None,
            Some(test_setter),
            None,
            None,
            Some(test_serializer),
            Some(test_deserializer),
        );

        assert_eq!(
            schema.get_deserializer("health").unwrap() as usize,
            test_deserializer as *const () as usize
        );
    }
}
