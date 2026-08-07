//! Component schema support used by component plugins and scene queries.

/// C-compatible component declarations used by plugin manifests.
pub mod descriptor;
mod error;
mod field;
/// Runtime field type hints.
pub mod field_type;
/// Synchronous component method calls over the plugin ABI.
pub mod methods;
pub(crate) mod schema;
/// ABI-safe serialized field bytes.
pub mod serialized_bytes;

use std::{collections::HashMap, ffi::c_void, rc::Rc};

use crate::scene::Scene;

pub use error::ComponentError;

pub use descriptor::{Creator, Destroyer, WXRComponentDescriptor, WXRComponentFieldDescriptor};
pub use field::Getter;
pub use field::{Deserializer, Serializer};
pub use field_type::FieldType;
pub use serialized_bytes::SerializedBytes;

use crate::scene::serialization::{ComponentData, FieldData};

pub(crate) struct ComponentDefinition {
    id: String,
    plugin_id: String,
    creator: Creator,
    destroyer: Destroyer,
    schema: schema::Schema,
    methods: HashMap<String, Rc<methods::MethodDefinition>>,
}

impl ComponentDefinition {
    pub(crate) fn new(
        id: String,
        plugin_id: String,
        creator: Creator,
        destroyer: Destroyer,
        schema: schema::Schema,
        methods: HashMap<String, methods::MethodDefinition>,
    ) -> Self {
        Self {
            id,
            plugin_id,
            creator,
            destroyer,
            schema,
            methods: methods
                .into_iter()
                .map(|(name, method)| (name, Rc::new(method)))
                .collect(),
        }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    #[cfg(test)]
    pub(crate) fn field_is_mutable(&self, name: &str) -> bool {
        self.schema.is_mutable(name).expect("test field exists")
    }

    #[cfg(test)]
    pub(crate) fn method_argument_is_nullable(&self, method: &str, index: usize) -> bool {
        self.methods[method].argument_is_nullable(index)
    }
}

pub(crate) struct Component {
    definition: Rc<ComponentDefinition>,
    data: *mut c_void,
}

impl Component {
    pub(crate) fn create(definition: Rc<ComponentDefinition>, scene: &mut Scene) -> Option<Self> {
        let data = unsafe { (definition.creator)(scene as *mut Scene) };
        if data.is_null() {
            return None;
        }

        crate::info!(scene, "Component `{}` created", definition.id);
        Some(Self { definition, data })
    }

    pub(crate) unsafe fn get_field_ptr(&self, id: &str) -> Result<*mut c_void, ComponentError> {
        let getter = self.definition.schema.get_getter(id)?;

        let field_ptr = unsafe { getter(self.data) };

        Ok(field_ptr)
    }

    pub(crate) fn is_field_mutable(&self, id: &str) -> Result<bool, ComponentError> {
        self.definition.schema.is_mutable(id)
    }

    pub(crate) fn is_field_string_parsable(&self, id: &str) -> Result<bool, ComponentError> {
        self.definition.schema.is_string_parsable(id)
    }

    pub(crate) fn serialize(&self, entity_id: uuid::Uuid) -> ComponentData {
        let fields = self
            .definition
            .schema
            .get_fields()
            .into_iter()
            .filter_map(|field_id| {
                let serializer = self.definition.schema.get_serializer(field_id).ok()?;
                let value = unsafe { serializer(self.data as *const c_void) };
                Some(FieldData {
                    name: field_id.to_owned(),
                    value: unsafe { value.into_vec() },
                })
            })
            .collect();

        ComponentData {
            id: self.definition.id.clone(),
            entity_id,
            fields,
        }
    }

    pub(crate) fn deserialize_fields(&mut self, fields: Vec<FieldData>) {
        for field in fields {
            let Ok(deserializer) = self.definition.schema.get_deserializer(&field.name) else {
                continue;
            };

            let value = SerializedBytes::from_vec(field.value);
            unsafe {
                deserializer(self.data, value);
            }
        }
    }

    pub(crate) fn get_fields(&self) -> Vec<String> {
        self.definition
            .schema
            .get_fields()
            .into_iter()
            .map(String::to_owned)
            .collect()
    }

    pub(crate) fn get_field_type(&self, id: &str) -> Result<FieldType, ComponentError> {
        self.definition.schema.get_field_type(id)
    }

    pub(crate) fn render_field(&self, id: &str) -> Result<String, ComponentError> {
        unsafe { self.definition.schema.render_field(id, self.data) }
    }

    pub(crate) fn parse_field(&self, id: &str, input: &str) -> Result<(), ComponentError> {
        if !self.definition.schema.is_mutable(id)? {
            return Err(ComponentError::FieldNotMutable);
        }

        unsafe { self.definition.schema.parse_field(id, self.data, input) }
    }

    pub(crate) fn get_data(&self) -> *mut c_void {
        self.data
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.definition.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.definition.plugin_id
    }

    pub(crate) fn get_method(&self, name: &str) -> Option<Rc<methods::MethodDefinition>> {
        self.definition.methods.get(name).cloned()
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe {
            (self.definition.destroyer)(self.data);
        }
    }
}
