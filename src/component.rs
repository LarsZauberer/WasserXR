use std::{collections::HashMap, os::raw::c_void};

use crate::{error::ComponentError, plugin::Plugin};

pub type Creator = unsafe extern "C" fn() -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut c_void);
pub type SchemaCreator = unsafe extern "C" fn(*mut ComponentSchema);
pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);

#[derive(Clone, Copy)]
pub struct ComponentFunctions {
    destroyer: Destroyer,
    schema_creator: Option<SchemaCreator>,
}

impl ComponentFunctions {
    pub fn new(destroyer: Destroyer, schema_creator: Option<SchemaCreator>) -> Self {
        Self {
            destroyer,
            schema_creator,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ComponentField {
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl ComponentField {
    pub fn new(getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self { getter, setter }
    }

    pub fn get<T>(&self, component: &Component) -> Result<T, ComponentError> {
        todo!()
    }

    pub fn set<T>(&self, component: &mut Component, data: &T) -> Result<(), ComponentError> {
        todo!()
    }
}

pub struct ComponentSchema {
    fields: HashMap<String, ComponentField>,
}

pub struct Component {
    id: String,
    functions: ComponentFunctions,
    data: *mut c_void,
    schema: ComponentSchema,
}

impl Component {
    pub fn new(id: String, plugin: &Plugin) -> Result<Self, ComponentError> {
        todo!()
    }

    pub fn get<T>(&self, id: &str) -> Result<T, ComponentError> {
        if let Some(field) = self.schema.fields.get(id) {
            field.get(self)
        } else {
            Err(ComponentError::FieldNotFound)
        }
    }

    pub fn set<T>(&mut self, id: &str, data: &T) -> Result<(), ComponentError> {
        if let Some(field) = self.schema.fields.get(id) {
            field.clone().set(self, data)
        } else {
            Err(ComponentError::FieldNotFound)
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroyer)(self as *mut Component as *mut c_void);
        }
    }
}
