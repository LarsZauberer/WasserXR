mod field;
pub mod field_type;
pub mod schema;

use std::ffi::c_void;

use crate::{error::ComponentError, scene::plugin::Plugin};

pub use field::{Getter, Setter};
pub use field_type::FieldType;
pub use schema::Schema;

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

        let creator = plugin
            .get_symbol::<Creator>(&creator_symbol)
            .map_err(|error| ComponentError::NoCreator(error))?;

        let destroyer = plugin
            .get_symbol(&destroyer_symbol)
            .map_err(|error| ComponentError::NoDestroyer(error))?;

        let schema_creator = plugin.get_symbol(&schema_symbol).unwrap_or_else(|_| {
            log::debug!("Component `{}` has no schema creator defined", id);
            default_schema
        });

        let data = unsafe { (creator)() };
        let mut schema = Schema::new();
        unsafe {
            schema_creator(&mut schema as *mut Schema);
        }

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
            std::mem::transmute_copy(&data)
        }
    }

    pub(crate) fn set<T>(&mut self, id: &str, data: &T) -> Result<(), ComponentError> {
        let setter = self.schema.get_setter(id)?;

        unsafe {
            setter(self.data, data as *const T as *const c_void);
        }
        Ok(())
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub(crate) fn get_fields(&self) -> Vec<&String> {
        self.schema.get_fields()
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.destroyer)(self.data);
        }
    }
}
