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
