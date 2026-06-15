use std::{collections::HashMap, fmt::Display, os::raw::c_void, rc::Rc, str::FromStr};

use crate::{error::ComponentError, plugin::Plugin};

pub type Creator = unsafe extern "C" fn() -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut c_void);
pub type SchemaCreator = unsafe extern "C" fn(*mut ComponentSchema);
pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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
    pub fn new(id: &str, plugin: &Plugin) -> Result<Self, ComponentError> {
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

#[derive(Default)]
pub struct ComponentSchema {
    fields: HashMap<String, ComponentField>,
}

impl ComponentSchema {
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

    pub fn get<T>(&self, component: &Component, id: &str) -> Result<T, ComponentError> {
        todo!()
    }

    pub fn set<T>(
        &self,
        component: &mut Component,
        id: &str,
        data: &T,
    ) -> Result<T, ComponentError> {
        todo!()
    }

    pub fn get_fields(&self) -> Vec<&String> {
        self.fields.keys().collect()
    }
}

pub struct Component {
    id: String,
    plugin: Rc<Plugin>,
    functions: ComponentFunctions,
    data: *mut c_void,
    schema: ComponentSchema,
}

impl Component {
    pub fn new(id: String, plugin: Rc<Plugin>) -> Result<Self, ComponentError> {
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
        &self.plugin.get_id()
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroyer)(self as *mut Component as *mut c_void);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::c_void,
        ptr::{null, null_mut},
        rc::Rc,
        sync::atomic::AtomicUsize,
    };

    use super::*;

    // -- Test doubles (C-FFI symbols) ----------------------------------------

    // Nonexistent component (no creator symbol exists)

    // Nonexistent destroyer (creator exists, destroyer does not)
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_nonexistent_destroyer() -> *mut c_void {
        null_mut()
    }

    // Basic component
    static DESTROY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_basic_component() -> *mut c_void {
        null_mut()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_basic_component(ptr: *mut c_void) {
        assert!(ptr.is_null());
        DESTROY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    // Schema component
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_schema_component() -> *mut c_void {
        null_mut()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_schema_component(ptr: *mut c_void) {
        assert!(ptr.is_null());
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_schema_component(schema: *mut ComponentSchema) {
        assert!(!schema.is_null());

        let schema = unsafe { &mut *schema };

        schema.add_field(
            "x".to_owned(),
            FieldType::Long,
            Some(wxr_get_schema_component_x),
            Some(wxr_set_schema_component_x),
        );
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_get_schema_component_x(ptr: *const c_void) -> *const c_void {
        assert!(ptr.is_null());
        null()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_set_schema_component_x(ptr: *mut c_void, data: *const c_void) {
        assert!(ptr.is_null());
        let data: usize = unsafe { *(data as *const usize) };
        assert_eq!(data, 5);
    }

    // -- ComponentFunctions error paths -------------------------------------

    #[test]
    fn test_component_functions_missing_symbols() {
        // Missing creator symbol
        let plugin = Plugin::new_static();
        match ComponentFunctions::new("nonexistent", &plugin) {
            Ok(_) => {
                panic!("Creation of nonexistent component should not succeed");
            }
            Err(ComponentError::MissingSymbol(symbol)) => {
                assert_eq!(symbol, "wxr_create_nonexistent");
            }
            _ => {
                panic!("Creation of nonexistent failed with different error");
            }
        }

        // Creator exists but destroyer symbol is missing
        match ComponentFunctions::new("nonexistent_destroyer", &plugin) {
            Ok(_) => {
                panic!("Creation of nonexistent_destroyer component should not succeed");
            }
            Err(ComponentError::MissingSymbol(symbol)) => {
                assert_eq!(symbol, "wxr_destroy_nonexistent_destroyer");
            }
            _ => {
                panic!("Creation of nonexistent_destroyer failed with different error");
            }
        }
    }

    // -- Basic component lifecycle -------------------------------------------

    #[test]
    fn test_basic_component_lifecycle() {
        DESTROY_COUNTER.store(0, std::sync::atomic::Ordering::SeqCst);
        let plugin = Plugin::new_static();

        // ComponentFunctions creation
        let functions = ComponentFunctions::new("basic_component", &plugin)
            .expect("Failed to create basic_component");
        assert!(functions.schema_creator.is_some());

        // Component creation and identity
        let plugin = Rc::new(Plugin::new_static());
        let component = Component::new("basic_component".to_owned(), plugin.clone())
            .expect("Component creation should have succeeded");

        assert_eq!(component.get_id(), "basic_component");
        assert_eq!(component.schema.get_fields().len(), 0);

        // Drop invokes destroyer
        drop(component);
        assert_eq!(DESTROY_COUNTER.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // -- Schema component: all operations ------------------------------------

    #[test]
    fn test_schema_component() {
        let plugin = Rc::new(Plugin::new_static());
        let component = Component::new("schema_component".to_owned(), plugin.clone())
            .expect("Component creation should have succeeded");

        // Schema metadata
        assert_eq!(component.schema.get_fields().len(), 1);
        assert_eq!(component.schema.get_fields()[0], "x");
        assert!(component.schema.fields["x"].getter.is_some());
        assert!(component.schema.fields["x"].setter.is_some());
        assert_eq!(component.schema.fields["x"].type_hint, FieldType::Long);

        // Get existing field
        let data: *const c_void = component.get("x").expect("Failed to get field x");
        assert!(data.is_null());

        // Get nonexistent field
        let _ = component
            .get::<*const c_void>("a")
            .expect_err("Got a non existing field a");

        // Set nonexistent field
        let mut component = Component::new("schema_component".to_owned(), plugin.clone())
            .expect("Component creation should have succeeded");
        let value = 5;
        let _ = component
            .set::<usize>("a", &value)
            .expect_err("Set nonexistent field should have failed");

        // Set existing field
        component
            .set::<usize>("x", &value)
            .expect("Failed to set field x");
    }
}
