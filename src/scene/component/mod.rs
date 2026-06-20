mod field;
pub mod field_type;
pub mod schema;
pub mod serialized_bytes;

use std::ffi::c_void;

use crate::{error::ComponentError, scene::plugin::Plugin};

pub use field::{Deserializer, Serializer};
pub use field::{Getter, Setter};
pub use field_type::FieldType;
pub use schema::Schema;
pub use serialized_bytes::SerializedBytes;

use crate::scene::serialization::{ComponentData, FieldData};

pub(crate) type Creator = unsafe extern "C" fn() -> *mut c_void;
pub(crate) type Destroyer = unsafe extern "C" fn(*mut c_void);
pub(crate) type SchemaCreator = unsafe extern "C" fn(*mut Schema);

// Default Schema creator
unsafe extern "C" fn default_schema(_schema: *mut Schema) {}

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
    pub(crate) fn new(id: String, plugin: &Plugin) -> Result<Self, ComponentError> {
        let plugin_id = plugin.get_id().to_owned();

        let creator_symbol = "wxr_create_".to_owned() + &id;
        let destroyer_symbol = "wxr_destroy_".to_owned() + &id;
        let schema_symbol = "wxr_schema_".to_owned() + &id;

        let creator: Creator = plugin
            .get_symbol::<Creator>(&creator_symbol)
            .map_err(|error| {
                log::debug!("Component `{}` has no creator function", id);
                ComponentError::NoCreator(error)
            })?;

        let destroyer: Destroyer = plugin.get_symbol(&destroyer_symbol).map_err(|error| {
            log::debug!("Component `{}` has no destroyer function", id);
            ComponentError::NoDestroyer(error)
        })?;

        let schema_creator: SchemaCreator = plugin
            .get_symbol::<SchemaCreator>(&schema_symbol)
            .unwrap_or_else(|_| {
                log::debug!("Component `{}` has no schema function", id);
                default_schema
            });

        let data = unsafe { (creator)() };
        let mut schema = Schema::new();
        unsafe {
            schema_creator(&mut schema as *mut Schema);
        }

        log::info!("Component `{}` created", id);
        Ok(Self {
            id,
            plugin_id,

            destroyer,

            data,
            schema,
        })
    }

    pub(crate) fn get<T>(&self, id: &str) -> Result<&T, ComponentError> {
        let getter = self.schema.get_getter(id)?;

        unsafe {
            let data = getter(self.data as *const c_void);
            Ok(&*(data as *const T))
        }
    }

    pub(crate) fn set<T>(&mut self, id: &str, data: &T) -> Result<(), ComponentError> {
        let setter = self.schema.get_setter(id)?;

        unsafe {
            setter(self.data, data as *const T as *const c_void);
        }
        Ok(())
    }

    pub(crate) fn serialize(&self, entity_id: uuid::Uuid) -> ComponentData {
        let fields = self
            .schema
            .get_fields()
            .into_iter()
            .filter_map(|field_id| {
                let serializer = self.schema.get_serializer(field_id).ok()?;
                let value = unsafe { serializer(self.data as *const c_void).into_vec() };
                Some(FieldData {
                    name: field_id.to_owned(),
                    value,
                })
            })
            .collect();

        ComponentData {
            id: self.id.clone(),
            entity_id,
            fields,
        }
    }

    pub(crate) fn deserialize(id: String, plugin: &Plugin) -> Result<Self, ComponentError> {
        Self::new(id, plugin)
    }

    pub(crate) fn deserialize_fields(&mut self, fields: Vec<FieldData>) {
        for field in fields {
            let Ok(deserializer) = self.schema.get_deserializer(&field.name) else {
                continue;
            };

            unsafe {
                deserializer(self.data, SerializedBytes::from_vec(field.value));
            }
        }
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
        log::info!("Component `{}` destroyed", self.id);
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

    // Getter and Setter
    unsafe extern "C" fn unit_counter_getter(data: *const c_void) -> *const c_void {
        unsafe { &(*(data as *const TestCounter)).value as *const i64 as *const c_void }
    }

    unsafe extern "C" fn unit_counter_setter(data: *mut c_void, value: *const c_void) {
        unsafe {
            (*(data as *mut TestCounter)).value = *(value as *const i64);
        }
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
    unsafe extern "C" fn wxr_create_unit_counter() -> *mut c_void {
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
                FieldType::Long,
                Some(unit_counter_getter),
                Some(unit_counter_setter),
                Some(unit_counter_serializer),
                Some(unit_counter_deserializer),
            );
        }
    }

    // Faulty component, that doesn't define a destroyer
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_missing_destroyer() -> *mut c_void {
        Box::into_raw(Box::new(TestCounter { value: 0 })) as *mut c_void
    }

    // Component that doesn't define a schema => `schema_less_counter`
    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_schema_less_counter() -> *mut c_void {
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
    unsafe extern "C" fn wxr_create_drop_counter() -> *mut c_void {
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
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin).unwrap();

        assert_eq!(component.get_id(), "unit_counter");
        assert_eq!(component.get_plugin_id(), "");
    }

    #[test]
    fn component_get_existing_field() {
        let plugin = Plugin::new_static();
        let component = Component::new("unit_counter".to_owned(), &plugin).unwrap();

        assert_eq!(*component.get::<i64>("value").unwrap(), 5);
    }

    #[test]
    fn component_set_existing_field() {
        let plugin = Plugin::new_static();
        let mut component = Component::new("unit_counter".to_owned(), &plugin).unwrap();

        component.set("value", &12_i64).unwrap();

        assert_eq!(*component.get::<i64>("value").unwrap(), 12);
    }

    #[test]
    fn component_serialize_round_trip() {
        let plugin = Plugin::new_static();
        let mut component = Component::new("unit_counter".to_owned(), &plugin).unwrap();
        component.set("value", &12_i64).unwrap();

        let data = component.serialize(uuid::Uuid::now_v7());
        let mut deserialized = Component::deserialize("unit_counter".to_owned(), &plugin).unwrap();
        deserialized.deserialize_fields(data.fields);

        assert_eq!(*deserialized.get::<i64>("value").unwrap(), 12);
    }

    #[test]
    fn component_get_missing_field() {
        let plugin = Plugin::new_static();
        let component = Component::new("schema_less_counter".to_owned(), &plugin).unwrap();

        assert_eq!(
            component.get::<i64>("value"),
            Err(ComponentError::FieldNotFound)
        );
    }

    #[test]
    fn component_new_without_creator() {
        let plugin = Plugin::new_static();

        assert!(matches!(
            Component::new("missing_creator".to_owned(), &plugin),
            Err(ComponentError::NoCreator(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_create_missing_creator"
        ));
    }

    #[test]
    fn component_new_without_destroyer() {
        let plugin = Plugin::new_static();

        assert!(matches!(
            Component::new("missing_destroyer".to_owned(), &plugin),
            Err(ComponentError::NoDestroyer(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_destroy_missing_destroyer"
        ));
    }

    #[test]
    fn component_drop_existing_component() {
        DROP_COUNTER_DESTROYED.store(0, Ordering::SeqCst);
        let plugin = Plugin::new_static();

        {
            let _component = Component::new("drop_counter".to_owned(), &plugin).unwrap();
        }

        assert_eq!(DROP_COUNTER_DESTROYED.load(Ordering::SeqCst), 1);
    }
}
