use std::{collections::HashMap, fmt::Display, os::raw::c_void, str::FromStr};

use crate::{error::ComponentError, plugin::Plugin};

pub type Creator = unsafe extern "C" fn() -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut c_void);
pub type SchemaCreator = unsafe extern "C" fn(*mut ComponentSchema);
pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);

#[derive(Clone, Copy)]
pub enum FieldType {
    Long,
    Float,
    Char,
    String,
    Blob,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::Blob
    }
}

#[derive(Clone, Copy)]
pub struct ComponentFunctions {
    destroyer: Destroyer,
    schema_creator: Option<SchemaCreator>,
}

impl ComponentFunctions {
    pub fn new(id: &str, plugin: &Plugin) -> Self {
        todo!()
    }

    pub fn destroy(&self, component: &mut Component) {
        todo!()
    }

    pub fn create_schema(&self) -> ComponentSchema {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct ComponentField {
    type_hint: FieldType,
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl ComponentField {
    pub fn new(type_hint: FieldType, getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self {
            type_hint,
            getter,
            setter,
        }
    }

    pub fn get<T>(&self, component: &Component) -> Result<T, ComponentError> {
        todo!()
    }

    pub fn set<T>(&self, component: &mut Component, data: &T) -> Result<(), ComponentError> {
        todo!()
    }
}

impl FromStr for ComponentField {
    type Err = ComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for ComponentField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct ComponentSchema {
    fields: HashMap<String, ComponentField>,
}

impl ComponentSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(
        &mut self,
        id: String,
        type_hint: FieldType,
        getter: Option<Getter>,
        setter: Option<Setter>,
    ) {
        let field = ComponentField::new(type_hint, getter, setter);
        self.fields.insert(id, field);
    }
}

pub struct Component {
    id: String,
    plugin_id: String,
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

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroyer)(self as *mut Component as *mut c_void);
        }
    }
}
