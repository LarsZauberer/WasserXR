use std::{os::raw::c_void, ptr::null};

use crate::scene::plugin::Plugin;

pub type Creator = unsafe extern "C" fn() -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut c_void);
pub type SchemaCreator = unsafe extern "C" fn(*mut ComponentSchema);
pub type Getter = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub type Setter = unsafe extern "C" fn(*mut c_void, *const c_void);

#[repr(C)]
pub enum FieldType {
    WXRLong,
    WXRFloat,
    WXRChar,
    WXRString,
    WXRBlob,
}

impl Default for FieldType {
    fn default() -> Self {
        Self::WXRBlob
    }
}

pub struct Component {
    id: String,
    component: *mut c_void,
    destroyer: Destroyer,
    schema: ComponentSchema,
}

pub struct ComponentSchema {
    fields: Vec<ComponentField>,
}

pub struct ComponentField {
    id: String,
    type_hint: FieldType,
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl Component {
    pub fn new(plugin: &Plugin, id: &str) -> Option<Self> {
        let creator_symbol = "wxr_create_".to_owned() + id;
        let destroyer_symbol = "wxr_destroy_".to_owned() + id;
        let schema_creator_symbol = "wxr_schema_".to_owned() + id;

        let creator: Option<Creator> = plugin.get_abi_symbol(&creator_symbol);
        let destroyer: Option<Destroyer> = plugin.get_abi_symbol(&destroyer_symbol);
        let schema_creator: Option<SchemaCreator> = plugin.get_abi_symbol(&schema_creator_symbol);

        let Some(creator) = creator else {
            return None;
        };
        let Some(destroyer) = destroyer else {
            return None;
        };

        let component = unsafe { creator() };

        let mut schema = ComponentSchema::new();

        if let Some(schema_creator) = schema_creator {
            unsafe {
                schema_creator(&mut schema as *mut ComponentSchema);
            }
        }

        Some(Component {
            id: id.to_owned(),
            component,
            destroyer,
            schema,
        })
    }

    pub fn get(&self, id: &str) -> *const c_void {
        if let Some(field) = self
            .schema
            .get_fields()
            .iter()
            .find(|field| field.get_id() == id)
        {
            field.get(&self)
        } else {
            null()
        }
    }

    pub fn set(&mut self, id: &str, data: *const c_void) -> bool {
        let schema = std::mem::take(&mut self.schema);

        let res = if let Some(field) = schema
            .get_fields()
            .iter()
            .find(|field| field.get_id() == id)
        {
            field.set(self, data);
            true
        } else {
            false
        };

        self.schema = schema;
        res
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.destroyer)(self.component) }
    }
}

impl ComponentSchema {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_field(
        &mut self,
        id: &str,
        type_hint: FieldType,
        getter: Option<Getter>,
        setter: Option<Setter>,
    ) {
        self.fields
            .push(ComponentField::new(id, type_hint, getter, setter));
    }

    pub fn get_fields(&self) -> Vec<&ComponentField> {
        self.fields.iter().collect()
    }
}

impl Default for ComponentSchema {
    fn default() -> Self {
        ComponentSchema::new()
    }
}

impl ComponentField {
    pub fn new(
        id: &str,
        type_hint: FieldType,
        getter: Option<Getter>,
        setter: Option<Setter>,
    ) -> Self {
        Self {
            id: id.to_owned(),
            type_hint,
            getter,
            setter,
        }
    }

    pub fn get(&self, component: &Component) -> *const c_void {
        if let Some(getter) = self.getter {
            unsafe { getter(component.component as *const c_void) }
        } else {
            null()
        }
    }

    pub fn set(&self, component: &mut Component, data: *const c_void) {
        if let Some(setter) = self.setter {
            unsafe { setter(component.component as *mut c_void, data) }
        }
    }

    pub fn get_id(&self) -> String {
        self.id.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::CStr,
        os::raw::c_char,
        sync::atomic::{AtomicUsize, Ordering},
    };

    // Testing Component Struct
    #[repr(C)]
    struct TestComponent {
        value: i64,
    }

    // Global Constants to visualize change during tests
    static BASIC_COMPONENT_DESTROYS: AtomicUsize = AtomicUsize::new(0);
    static FIELD_COMPONENT_DESTROYS: AtomicUsize = AtomicUsize::new(0);

    // Utility
    fn read_i64(ptr: *const c_void) -> i64 {
        assert!(!ptr.is_null());
        unsafe { *(ptr as *const i64) }
    }

    // Non Existent Component Test
    #[test]
    fn create_component_non_existent() {
        let plugin = Plugin::new_static();
        let component = Component::new(&plugin, "does_not_exist");
        assert!(component.is_none());
    }

    // Missing Destroyer Test
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_missing_destroyer_component() -> *mut c_void {
        Box::into_raw(Box::new(TestComponent { value: 0 })) as *mut c_void
    }

    #[test]
    fn create_component_missing_destroyer() {
        let plugin = Plugin::new_static();
        let component = Component::new(&plugin, "missing_destroyer_component");
        assert!(component.is_none());
    }

    // Basic Component Test
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_basic_component() -> *mut c_void {
        Box::into_raw(Box::new(TestComponent { value: 0 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_basic_component(component: *mut c_void) {
        BASIC_COMPONENT_DESTROYS.fetch_add(1, Ordering::SeqCst);
        unsafe {
            drop(Box::from_raw(component as *mut TestComponent));
        }
    }

    #[test]
    fn create_component() {
        BASIC_COMPONENT_DESTROYS.store(0, Ordering::SeqCst);

        let plugin = Plugin::new_static();
        let component = Component::new(&plugin, "basic_component");
        assert!(component.is_some());
        drop(component);

        assert_eq!(BASIC_COMPONENT_DESTROYS.load(Ordering::SeqCst), 1);
    }

    // Component Field Testing
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_field_component() -> *mut c_void {
        Box::into_raw(Box::new(TestComponent { value: 5 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_field_component(component: *mut c_void) {
        FIELD_COMPONENT_DESTROYS.fetch_add(1, Ordering::SeqCst);
        unsafe {
            drop(Box::from_raw(component as *mut TestComponent));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_field_component(schema: *mut ComponentSchema) {
        let schema = unsafe { &mut *schema };

        schema.add_field(
            "value",
            FieldType::WXRLong,
            Some(wxr_get_field_component_value),
            Some(wxr_set_field_component_value),
        );
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_get_field_component_value(component: *const c_void) -> *const c_void {
        let component = unsafe { &*(component as *const TestComponent) };
        &component.value as *const i64 as *const c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_set_field_component_value(
        component: *mut c_void,
        data: *const c_void,
    ) {
        let component = unsafe { &mut *(component as *mut TestComponent) };
        component.value = unsafe { *(data as *const i64) };
    }

    #[test]
    fn get_and_set_component_field() {
        FIELD_COMPONENT_DESTROYS.store(0, Ordering::SeqCst);

        let plugin = Plugin::new_static();
        let mut component = Component::new(&plugin, "field_component").unwrap();

        assert_eq!(read_i64(component.get("value")), 5);

        let value = 15_i64;
        assert!(component.set("value", &value as *const i64 as *const c_void));
        assert_eq!(read_i64(component.get("value")), 15);

        drop(component);
        assert_eq!(FIELD_COMPONENT_DESTROYS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn missing_component_field() {
        let plugin = Plugin::new_static();
        let mut component = Component::new(&plugin, "field_component").unwrap();
        let value = 15_i64;

        assert!(component.get("missing").is_null());
        assert!(!component.set("missing", &value as *const i64 as *const c_void));
    }
}
