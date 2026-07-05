//! Component schema support used by component plugins and scene queries.

mod field;
/// Runtime field type hints.
pub mod field_type;
/// Component schema storage.
pub mod schema;
/// ABI-safe serialized field bytes.
pub mod serialized_bytes;

use std::ffi::c_void;

use crate::{
    error::ComponentError,
    scene::{Scene, plugin::Plugin},
};

pub use field::Getter;
pub use field::{Deserializer, Serializer};
pub use field_type::FieldType;
pub use schema::Schema;
pub use serialized_bytes::SerializedBytes;

use crate::scene::serialization::{ComponentData, FieldData};

pub(crate) type Creator = unsafe extern "C" fn(*mut Scene) -> *mut c_void;
pub(crate) type Destroyer = unsafe extern "C" fn(*mut c_void);
pub(crate) type SchemaCreator = unsafe extern "C" fn(*mut Schema);

// Default Schema creator
unsafe extern "C" fn default_schema(_schema: *mut Schema) {}

pub(crate) struct ComponentSymbols {
    plugin_id: String,
    creator: Creator,
    destroyer: Destroyer,
    schema_creator: SchemaCreator,
}

pub(crate) struct Component {
    // Metadata
    id: String,
    plugin_id: String,

    // Functions
    destroyer: Destroyer,

    // Raw data
    data: *mut c_void,
    schema: Schema,
}

impl Component {
    #[cfg(test)]
    pub(crate) fn new(
        id: String,
        plugin: &Plugin,
        scene: &mut Scene,
    ) -> Result<Self, ComponentError> {
        let symbols = Self::symbols(&id, plugin, scene)?;
        Ok(Self::create_with(id, symbols, scene).expect("test component creator returned null"))
    }

    pub(crate) fn symbols(
        id: &str,
        plugin: &Plugin,
        scene: &Scene,
    ) -> Result<ComponentSymbols, ComponentError> {
        let plugin_id = plugin.get_id().to_owned();

        let creator_symbol = "wxr_create_".to_owned() + id;
        let destroyer_symbol = "wxr_destroy_".to_owned() + id;
        let schema_symbol = "wxr_schema_".to_owned() + id;

        let creator: Creator = plugin
            .get_symbol::<Creator>(&creator_symbol)
            .map_err(|error| {
                crate::debug!(scene, "Component `{}` has no creator function", id);
                ComponentError::NoCreator(error)
            })?;

        let destroyer: Destroyer = plugin.get_symbol(&destroyer_symbol).map_err(|error| {
            crate::debug!(scene, "Component `{}` has no destroyer function", id);
            ComponentError::NoDestroyer(error)
        })?;

        let schema_creator: SchemaCreator = plugin
            .get_symbol::<SchemaCreator>(&schema_symbol)
            .unwrap_or_else(|_| {
                crate::debug!(scene, "Component `{}` has no schema function", id);
                default_schema
            });

        Ok(ComponentSymbols {
            plugin_id,
            creator,
            destroyer,
            schema_creator,
        })
    }

    pub(crate) fn create_with(
        id: String,
        symbols: ComponentSymbols,
        scene: &mut Scene,
    ) -> Option<Self> {
        let data = unsafe { (symbols.creator)(scene as *mut Scene) };
        if data.is_null() {
            return None;
        }

        let mut schema = Schema::default();
        unsafe {
            (symbols.schema_creator)(&mut schema as *mut Schema);
        }

        crate::info!(scene, "Component `{}` created", id);
        Some(Self {
            id,
            plugin_id: symbols.plugin_id,

            destroyer: symbols.destroyer,

            data,
            schema,
        })
    }

    pub(crate) unsafe fn get_field_ptr(&self, id: &str) -> Result<*mut c_void, ComponentError> {
        let getter = self.schema.get_getter(id)?;

        let field_ptr = unsafe { getter(self.data) };

        Ok(field_ptr)
    }

    pub(crate) fn is_field_mutable(&self, id: &str) -> Result<bool, ComponentError> {
        self.schema.is_mutable(id)
    }

    pub(crate) fn is_field_string_parsable(&self, id: &str) -> Result<bool, ComponentError> {
        self.schema.is_string_parsable(id)
    }

    pub(crate) fn serialize(&self, entity_id: uuid::Uuid) -> ComponentData {
        let fields = self
            .schema
            .get_fields()
            .into_iter()
            .filter_map(|field_id| {
                let serializer = self.schema.get_serializer(field_id).ok()?;
                let value = unsafe { serializer(self.data as *const c_void) };
                Some(FieldData {
                    name: field_id.to_owned(),
                    value: unsafe { value.into_vec() },
                })
            })
            .collect();

        ComponentData {
            id: self.id.clone(),
            entity_id,
            fields,
        }
    }

    #[cfg(test)]
    pub(crate) fn deserialize(
        id: String,
        plugin: &Plugin,
        scene: &mut Scene,
    ) -> Result<Self, ComponentError> {
        Self::new(id, plugin, scene)
    }

    pub(crate) fn deserialize_fields(&mut self, fields: Vec<FieldData>) {
        for field in fields {
            let Ok(deserializer) = self.schema.get_deserializer(&field.name) else {
                continue;
            };

            let value = SerializedBytes::from_vec(field.value);
            unsafe {
                deserializer(self.data, value);
            }
        }
    }

    pub(crate) fn get_fields(&self) -> Vec<String> {
        self.schema
            .get_fields()
            .into_iter()
            .map(String::to_owned)
            .collect()
    }

    pub(crate) fn get_field_type(&self, id: &str) -> Result<FieldType, ComponentError> {
        self.schema.get_field_type(id)
    }

    pub(crate) fn render_field(&self, id: &str) -> Result<String, ComponentError> {
        unsafe { self.schema.render_field(id, self.data) }
    }

    pub(crate) fn parse_field(&self, id: &str, input: &str) -> Result<(), ComponentError> {
        if !self.schema.is_mutable(id)? {
            return Err(ComponentError::FieldNotMutable);
        }

        unsafe { self.schema.parse_field(id, self.data, input) }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.destroyer)(self.data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PluginError;
    use std::{
        ffi::c_void,
        sync::atomic::{AtomicUsize, Ordering},
    };

    // Testing Struct that will be turned into a Component
    #[repr(C)]
    struct TestCounter {
        value: i64,
    }

    unsafe extern "C" fn unit_counter_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut TestCounter)).value as *mut i64 as *mut c_void }
    }

    unsafe extern "C" fn unit_counter_serializer(data: *const c_void) -> SerializedBytes {
        unsafe {
            SerializedBytes::from_vec((*(data as *const TestCounter)).value.to_le_bytes().to_vec())
        }
    }

    unsafe extern "C" fn unit_counter_deserializer(data: *mut c_void, value: SerializedBytes) {
        unsafe {
            let bytes = value.into_vec();
            if let Ok(bytes) = <[u8; 8]>::try_from(bytes.as_slice()) {
                (*(data as *mut TestCounter)).value = i64::from_le_bytes(bytes);
            }
        }
    }

    // Basic working component => `unit_counter`
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_unit_counter(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(TestCounter { value: 5 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_unit_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut TestCounter));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_schema_unit_counter(schema: *mut Schema) {
        unsafe {
            (*schema).add_field(
                "value".to_owned(),
                FieldType::I64,
                Some(unit_counter_getter),
                true,
                Some(unit_counter_serializer),
                Some(unit_counter_deserializer),
            );
        }
    }

    // Faulty component, that doesn't define a destroyer
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_missing_destroyer(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(TestCounter { value: 0 })) as *mut c_void
    }

    // Component that doesn't define a schema => `schema_less_counter`
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_schema_less_counter(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(TestCounter { value: 0 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_schema_less_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut TestCounter));
        }
    }

    // Component that checks clean destruction => `drop_counter`
    static DROP_COUNTER_DESTROYED: AtomicUsize = AtomicUsize::new(0);

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_drop_counter(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(TestCounter { value: 0 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_drop_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut TestCounter));
        }
        DROP_COUNTER_DESTROYED.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn component_new_with_static_symbols() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin, &mut scene).unwrap();

        assert_eq!(component.get_id(), "unit_counter");
        assert_eq!(component.get_plugin_id(), "");
    }

    #[test]
    fn component_get_field_ptr_existing_field() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin, &mut scene).unwrap();

        let field = unsafe { component.get_field_ptr("value").unwrap() };

        assert_eq!(unsafe { *(field as *const i64) }, 5);
    }

    #[test]
    fn component_get_field_ptr_allows_raw_mutation() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin, &mut scene).unwrap();

        let field = unsafe { component.get_field_ptr("value").unwrap() };
        unsafe {
            *(field as *mut i64) = 11;
        }

        let field = unsafe { component.get_field_ptr("value").unwrap() };
        assert_eq!(unsafe { *(field as *const i64) }, 11);
    }

    #[test]
    fn component_reports_field_mutability() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin, &mut scene).unwrap();

        assert!(component.is_field_mutable("value").unwrap());
    }

    #[test]
    fn component_serialize_round_trip() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin, &mut scene).unwrap();
        let field = unsafe { component.get_field_ptr("value").unwrap() };
        unsafe {
            *(field as *mut i64) = 12;
        }

        let data = component.serialize(uuid::Uuid::now_v7());
        let mut deserialized =
            Component::deserialize("unit_counter".to_owned(), &plugin, &mut scene).unwrap();
        deserialized.deserialize_fields(data.fields);

        let field = unsafe { deserialized.get_field_ptr("value").unwrap() };
        assert_eq!(unsafe { *(field as *const i64) }, 12);
    }

    #[test]
    fn component_get_field_ptr_missing_field() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let component =
            Component::new("schema_less_counter".to_owned(), &plugin, &mut scene).unwrap();

        assert_eq!(
            unsafe { component.get_field_ptr("value") },
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn component_new_without_creator() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();

        assert!(matches!(
            Component::new("missing_creator".to_owned(), &plugin, &mut scene),
            Err(ComponentError::NoCreator(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_create_missing_creator"
        ));
    }

    #[test]
    fn component_new_without_destroyer() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();

        assert!(matches!(
            Component::new("missing_destroyer".to_owned(), &plugin, &mut scene),
            Err(ComponentError::NoDestroyer(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_destroy_missing_destroyer"
        ));
    }

    #[test]
    fn component_drop_existing_component() {
        let mut scene = Scene::new();
        DROP_COUNTER_DESTROYED.store(0, Ordering::SeqCst);
        let plugin = Plugin::new_static();

        {
            let _component =
                Component::new("drop_counter".to_owned(), &plugin, &mut scene).unwrap();
        }

        assert_eq!(DROP_COUNTER_DESTROYED.load(Ordering::SeqCst), 1);
    }
}
