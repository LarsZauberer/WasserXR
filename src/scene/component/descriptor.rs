//! C-compatible bindings for component declarations used by plugin manifests.

use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_void},
};

use crate::{
    bindings::scene::WXRScene,
    scene::{
        Scene,
        component::methods::WXRMethodResult,
        plugin::manifest::{
            ManifestError, copy_name, descriptor_slice, field_type, missing_callback,
            register_local,
        },
    },
};

use super::{
    ComponentDefinition, SerializedBytes,
    methods::{MethodArgument, MethodDefinition},
    schema::Schema,
};

pub type Creator = unsafe extern "C" fn(*mut Scene) -> *mut c_void;
pub type Destroyer = unsafe extern "C" fn(*mut c_void);

/// C-compatible declaration of one component field.
#[repr(C)]
pub struct WXRComponentFieldDescriptor {
    pub name: *const c_char,
    pub field_type: u32,
    pub getter: Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void>,
    pub mutable: u8,
    pub serializer: Option<unsafe extern "C" fn(*const c_void) -> SerializedBytes>,
    pub deserializer: Option<unsafe extern "C" fn(*mut c_void, SerializedBytes)>,
}

/// C-compatible declaration of one required method argument.
#[repr(C)]
pub struct WXRMethodArgumentDescriptor {
    pub name: *const c_char,
    pub field_type: u32,
    pub nullable: u8,
}

/// C-compatible declaration of one component method.
#[repr(C)]
pub struct WXRComponentMethodDescriptor {
    pub name: *const c_char,
    // Expanded for cbindgen; aliases inside Option emit incomplete C types.
    pub callback: Option<
        unsafe extern "C" fn(
            *mut WXRScene,
            *mut c_void,
            *const *mut c_void,
            usize,
        ) -> WXRMethodResult,
    >,
    pub arguments: *const WXRMethodArgumentDescriptor,
    pub argument_count: usize,
}

/// C-compatible declaration of one plugin-provided component type.
#[repr(C)]
pub struct WXRComponentDescriptor {
    pub name: *const c_char,
    // Expanded for cbindgen; aliases inside Option emit incomplete C types.
    pub creator: Option<unsafe extern "C" fn(*mut Scene) -> *mut c_void>,
    pub destroyer: Option<unsafe extern "C" fn(*mut c_void)>,
    pub fields: *const WXRComponentFieldDescriptor,
    pub field_count: usize,
    pub methods: *const WXRComponentMethodDescriptor,
    pub method_count: usize,
}

// Descriptors are immutable process-lifetime declarations. Loading their raw
// pointers remains unsafe and validation copies all data used by the host.
unsafe impl Sync for WXRComponentFieldDescriptor {}
unsafe impl Sync for WXRMethodArgumentDescriptor {}
unsafe impl Sync for WXRComponentMethodDescriptor {}
unsafe impl Sync for WXRComponentDescriptor {}

impl WXRComponentDescriptor {
    pub(crate) unsafe fn validate(
        &self,
        plugin: &str,
    ) -> Result<ComponentDefinition, ManifestError> {
        let name = unsafe { copy_name(self.name, "component") }?;
        let creator = self
            .creator
            .ok_or_else(|| missing_callback("component", &name, "creator"))?;
        let destroyer = self
            .destroyer
            .ok_or_else(|| missing_callback("component", &name, "destroyer"))?;
        let fields =
            unsafe { descriptor_slice(self.fields, self.field_count, "component fields") }?;
        let methods =
            unsafe { descriptor_slice(self.methods, self.method_count, "component methods") }?;

        let mut field_names = HashSet::new();
        let mut schema = Schema::default();
        for field in fields {
            let field_name = unsafe { copy_name(field.name, "component field") }?;
            register_local(&mut field_names, &field_name, "component field")?;
            let field_type = field_type(field.field_type)?;
            if field.mutable != 0 && field.getter.is_none() {
                return Err(ManifestError::MutableWithoutGetter(field_name));
            }
            schema.add_field(
                field_name,
                field_type,
                field.getter,
                field.mutable != 0,
                field.serializer,
                field.deserializer,
            );
        }

        let mut method_names = HashSet::new();
        let mut validated_methods = HashMap::with_capacity(methods.len());
        for method in methods {
            let method_name = unsafe { copy_name(method.name, "component method") }?;
            register_local(&mut method_names, &method_name, "component method")?;
            let callback = method
                .callback
                .ok_or_else(|| missing_callback("method", &method_name, "callback"))?;
            let arguments = unsafe {
                descriptor_slice(method.arguments, method.argument_count, "method arguments")
            }?;
            let mut argument_names = HashSet::new();
            let mut validated_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let argument_name = unsafe { copy_name(argument.name, "method argument") }?;
                register_local(&mut argument_names, &argument_name, "method argument")?;
                validated_arguments.push(MethodArgument {
                    name: argument_name,
                    field_type: field_type(argument.field_type)?,
                    nullable: argument.nullable != 0,
                });
            }
            validated_methods.insert(
                method_name,
                MethodDefinition::new(callback, validated_arguments),
            );
        }

        Ok(ComponentDefinition::new(
            name,
            plugin.to_owned(),
            creator,
            destroyer,
            schema,
            validated_methods,
        ))
    }
}
