//! Validation and host-owned copying for plugin descriptor graphs.

mod error;

pub use error::ManifestError;

use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, c_char},
    mem::size_of,
    rc::Rc,
    slice,
};

use crate::scene::{
    assets::AssetType,
    component::{ComponentDefinition, FieldType},
    system::SystemDefinition,
};

use super::WXRPluginDescriptor;

pub(crate) struct ValidatedManifest {
    name: String,
    components: HashMap<String, Rc<ComponentDefinition>>,
    assets: HashMap<String, Rc<AssetType>>,
    systems: HashMap<String, Rc<SystemDefinition>>,
}

impl ValidatedManifest {
    pub(crate) fn new(
        name: String,
        components: HashMap<String, Rc<ComponentDefinition>>,
        assets: HashMap<String, Rc<AssetType>>,
        systems: HashMap<String, Rc<SystemDefinition>>,
    ) -> Self {
        Self {
            name,
            components,
            assets,
            systems,
        }
    }

    /// Copies and validates an entire process-lifetime descriptor graph.
    ///
    /// # Safety
    /// Every non-null pointer and count in the graph must describe readable,
    /// immutable process-lifetime storage. Function pointers must have their
    /// declared signatures and satisfy the callback contracts documented by
    /// [`crate::scene::Scene::load_plugin`].
    pub(crate) unsafe fn from_descriptor(
        descriptor: *const WXRPluginDescriptor,
    ) -> Result<Self, ManifestError> {
        let descriptor = unsafe { descriptor.as_ref() }.ok_or(ManifestError::NullDescriptor)?;
        unsafe { descriptor.validate() }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.name
    }

    pub(crate) fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.components
            .keys()
            .chain(self.assets.keys())
            .chain(self.systems.keys())
            .map(String::as_str)
    }

    pub(crate) fn component(&self, id: &str) -> Option<Rc<ComponentDefinition>> {
        self.components.get(id).cloned()
    }

    pub(crate) fn asset(&self, id: &str) -> Option<Rc<AssetType>> {
        self.assets.get(id).cloned()
    }

    pub(crate) fn system(&self, id: &str) -> Option<Rc<SystemDefinition>> {
        self.systems.get(id).cloned()
    }

    pub(crate) fn asset_names(&self) -> impl Iterator<Item = &str> {
        self.assets.keys().map(String::as_str)
    }

    pub(crate) fn is_consistent(&self) -> bool {
        self.components.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        }) && self.assets.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        }) && self.systems.iter().all(|(id, definition)| {
            id == definition.get_id() && definition.get_plugin_id() == self.name
        })
    }
}

pub(crate) fn field_type(value: u32) -> Result<FieldType, ManifestError> {
    FieldType::try_from(value).map_err(ManifestError::UnknownFieldType)
}

pub(crate) fn register_definition(
    names: &mut HashSet<String>,
    name: &str,
) -> Result<(), ManifestError> {
    if names.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(ManifestError::DuplicateDefinition(name.to_owned()))
    }
}

pub(crate) fn register_local(
    names: &mut HashSet<String>,
    name: &str,
    kind: &'static str,
) -> Result<(), ManifestError> {
    if names.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(ManifestError::DuplicateName {
            kind,
            name: name.to_owned(),
        })
    }
}

pub(crate) fn missing_callback(kind: &str, name: &str, callback: &str) -> ManifestError {
    ManifestError::MissingCallback(format!("{kind} `{name}` {callback}"))
}

pub(crate) unsafe fn copy_name(
    pointer: *const c_char,
    kind: &'static str,
) -> Result<String, ManifestError> {
    if pointer.is_null() {
        return Err(ManifestError::NullName(kind));
    }
    let name = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map_err(|_| ManifestError::InvalidUtf8(kind))?;
    if name.is_empty() {
        return Err(ManifestError::EmptyName(kind));
    }
    Ok(name.to_owned())
}

pub(crate) unsafe fn descriptor_slice<'a, T>(
    pointer: *const T,
    count: usize,
    kind: &'static str,
) -> Result<&'a [T], ManifestError> {
    if count == 0 {
        return if pointer.is_null() {
            Ok(&[])
        } else {
            Err(ManifestError::InvalidPointerCount(kind))
        };
    }
    if pointer.is_null() || size_of::<T>() != 0 && count > isize::MAX as usize / size_of::<T>() {
        return Err(ManifestError::InvalidPointerCount(kind));
    }
    Ok(unsafe { slice::from_raw_parts(pointer, count) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    use crate::{
        bindings::scene::{WXREntity, WXRScene},
        scene::{
            Scene,
            assets::{WXRAssetDescriptor, WXRAssetFieldDescriptor},
            component::{
                WXRComponentDescriptor, WXRComponentFieldDescriptor,
                methods::{
                    WXRComponentMethodDescriptor, WXRMethodArgumentDescriptor, WXRMethodResult,
                    WXRMethodStatus,
                },
            },
            plugin::Version,
            system::{WXRSystemDescriptor, WXRSystemEntityGroupDescriptor},
        },
    };

    unsafe extern "C" fn component_creator(_scene: *mut Scene) -> *mut c_void {
        std::ptr::dangling_mut::<u8>().cast()
    }
    unsafe extern "C" fn component_destroyer(_data: *mut c_void) {}
    unsafe extern "C" fn getter(_data: *mut c_void) -> *mut c_void {
        std::ptr::dangling_mut::<u8>().cast()
    }
    unsafe extern "C" fn asset_creator(_scene: *mut Scene, _data: *const c_char) -> *mut c_void {
        std::ptr::dangling_mut::<u8>().cast()
    }
    unsafe extern "C" fn asset_destroyer(_scene: *mut Scene, _data: *mut c_void) {}
    unsafe extern "C" fn method_callback(
        _scene: *mut WXRScene,
        _component: *mut c_void,
        _arguments: *const *mut c_void,
        _argument_count: usize,
    ) -> WXRMethodResult {
        WXRMethodResult {
            status: WXRMethodStatus::Success,
            action_error: 0,
            value: std::ptr::null_mut(),
        }
    }
    unsafe extern "C" fn system_runner(
        _scene: *mut Scene,
        _delta: f32,
        _entities: *const *const WXREntity,
        _counts: *const usize,
        _group_count: usize,
    ) {
    }

    fn slice_ptr<T>(values: &[T]) -> *const T {
        if values.is_empty() {
            std::ptr::null()
        } else {
            values.as_ptr()
        }
    }

    fn component<'a>(
        fields: &'a [WXRComponentFieldDescriptor],
        methods: &'a [WXRComponentMethodDescriptor],
    ) -> WXRComponentDescriptor {
        WXRComponentDescriptor {
            name: c"component".as_ptr(),
            creator: Some(component_creator),
            destroyer: Some(component_destroyer),
            fields: slice_ptr(fields),
            field_count: fields.len(),
            methods: slice_ptr(methods),
            method_count: methods.len(),
        }
    }

    fn asset(fields: &[WXRAssetFieldDescriptor]) -> WXRAssetDescriptor {
        WXRAssetDescriptor {
            name: c"asset".as_ptr(),
            creator: Some(asset_creator),
            destroyer: Some(asset_destroyer),
            fields: slice_ptr(fields),
            field_count: fields.len(),
        }
    }

    fn system(groups: &[WXRSystemEntityGroupDescriptor]) -> WXRSystemDescriptor {
        WXRSystemDescriptor {
            name: c"system".as_ptr(),
            runner: Some(system_runner),
            attach: None,
            detach: None,
            entity_groups: slice_ptr(groups),
            entity_group_count: groups.len(),
        }
    }

    #[test]
    fn version_compatibility_boundaries() {
        assert!(
            Version {
                major: 0,
                minor: 2,
                patch: 4
            }
            .is_compatible(Version {
                major: 0,
                minor: 2,
                patch: 99
            })
        );
        assert!(
            !Version {
                major: 0,
                minor: 2,
                patch: 4
            }
            .is_compatible(Version {
                major: 0,
                minor: 3,
                patch: 0
            })
        );
        assert!(
            Version {
                major: 1,
                minor: 2,
                patch: 4
            }
            .is_compatible(Version {
                major: 1,
                minor: 99,
                patch: 99
            })
        );
        assert!(
            !Version {
                major: 1,
                minor: 2,
                patch: 4
            }
            .is_compatible(Version {
                major: 2,
                minor: 0,
                patch: 0
            })
        );
    }

    #[test]
    fn pointer_count_requires_canonical_empty_pair() {
        let value = 1_u8;
        assert!(unsafe { descriptor_slice(&value, 0, "test") }.is_err());
        assert!(unsafe { descriptor_slice::<u8>(std::ptr::null(), 1, "test") }.is_err());
        assert!(
            unsafe { descriptor_slice::<u8>(std::ptr::null(), 0, "test") }
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn names_reject_null_empty_and_invalid_utf8() {
        static INVALID_UTF8: [u8; 2] = [0xff, 0];

        assert_eq!(
            unsafe { copy_name(std::ptr::null(), "test") },
            Err(ManifestError::NullName("test"))
        );
        assert_eq!(
            unsafe { copy_name(c"".as_ptr(), "test") },
            Err(ManifestError::EmptyName("test"))
        );
        assert_eq!(
            unsafe { copy_name(INVALID_UTF8.as_ptr().cast(), "test") },
            Err(ManifestError::InvalidUtf8("test"))
        );
    }

    #[test]
    fn component_nested_shapes_callbacks_duplicates_and_flags_are_validated() {
        let mut descriptor = component(&[], &[]);
        descriptor.fields = std::ptr::dangling();
        assert_eq!(
            unsafe { descriptor.validate("plugin") }.err().unwrap(),
            ManifestError::InvalidPointerCount("component fields")
        );
        descriptor = component(&[], &[]);
        descriptor.destroyer = None;
        assert!(matches!(
            unsafe { descriptor.validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));

        let mutable_without_getter = [WXRComponentFieldDescriptor {
            name: c"field".as_ptr(),
            field_type: FieldType::U8 as u32,
            getter: None,
            mutable: 1,
            serializer: None,
            deserializer: None,
        }];
        assert_eq!(
            unsafe { component(&mutable_without_getter, &[]).validate("plugin") }
                .err()
                .unwrap(),
            ManifestError::MutableWithoutGetter("field".to_owned())
        );

        let duplicate_fields = [
            WXRComponentFieldDescriptor {
                name: c"field".as_ptr(),
                field_type: FieldType::U8 as u32,
                getter: Some(getter),
                mutable: 0,
                serializer: None,
                deserializer: None,
            },
            WXRComponentFieldDescriptor {
                name: c"field".as_ptr(),
                field_type: FieldType::U8 as u32,
                getter: Some(getter),
                mutable: 0,
                serializer: None,
                deserializer: None,
            },
        ];
        assert!(matches!(
            unsafe { component(&duplicate_fields, &[]).validate("plugin") },
            Err(ManifestError::DuplicateName {
                kind: "component field",
                ..
            })
        ));

        let arguments = [
            WXRMethodArgumentDescriptor {
                name: c"argument".as_ptr(),
                field_type: FieldType::U8 as u32,
                nullable: 9,
            },
            WXRMethodArgumentDescriptor {
                name: c"argument".as_ptr(),
                field_type: FieldType::U8 as u32,
                nullable: 0,
            },
        ];
        let method = WXRComponentMethodDescriptor {
            name: c"method".as_ptr(),
            callback: Some(method_callback),
            arguments: arguments.as_ptr(),
            argument_count: arguments.len(),
        };
        assert!(matches!(
            unsafe { component(&[], &[method]).validate("plugin") },
            Err(ManifestError::DuplicateName {
                kind: "method argument",
                ..
            })
        ));

        let argument = [WXRMethodArgumentDescriptor {
            name: c"argument".as_ptr(),
            field_type: FieldType::U8 as u32,
            nullable: 9,
        }];
        let methods = [WXRComponentMethodDescriptor {
            name: c"method".as_ptr(),
            callback: Some(method_callback),
            arguments: argument.as_ptr(),
            argument_count: 1,
        }];
        let fields = [WXRComponentFieldDescriptor {
            name: c"field".as_ptr(),
            field_type: FieldType::U8 as u32,
            getter: Some(getter),
            mutable: 9,
            serializer: None,
            deserializer: None,
        }];
        let validated = unsafe { component(&fields, &methods).validate("plugin") }
            .expect("nonzero flags are accepted");
        assert!(validated.field_is_mutable("field"));
        assert!(validated.method_argument_is_nullable("method", 0));

        let missing_method = [WXRComponentMethodDescriptor {
            name: c"method".as_ptr(),
            callback: None,
            arguments: std::ptr::null(),
            argument_count: 0,
        }];
        assert!(matches!(
            unsafe { component(&[], &missing_method).validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));

        let mut invalid_arguments = WXRComponentMethodDescriptor {
            name: c"method".as_ptr(),
            callback: Some(method_callback),
            arguments: std::ptr::dangling(),
            argument_count: 0,
        };
        assert_eq!(
            unsafe { component(&[], std::slice::from_ref(&invalid_arguments)).validate("plugin") }
                .err()
                .unwrap(),
            ManifestError::InvalidPointerCount("method arguments")
        );
        invalid_arguments.arguments = std::ptr::null();
        let duplicate_methods = [
            invalid_arguments,
            WXRComponentMethodDescriptor {
                name: c"method".as_ptr(),
                callback: Some(method_callback),
                arguments: std::ptr::null(),
                argument_count: 0,
            },
        ];
        assert!(matches!(
            unsafe { component(&[], &duplicate_methods).validate("plugin") },
            Err(ManifestError::DuplicateName {
                kind: "component method",
                ..
            })
        ));
    }

    #[test]
    fn asset_and_system_require_nested_callbacks_and_canonical_pairs() {
        let field = [WXRAssetFieldDescriptor {
            name: c"field".as_ptr(),
            field_type: FieldType::U8 as u32,
            getter: None,
        }];
        assert!(matches!(
            unsafe { asset(&field).validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));
        let mut descriptor = asset(&[]);
        descriptor.creator = None;
        assert!(matches!(
            unsafe { descriptor.validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));
        descriptor.creator = Some(asset_creator);
        descriptor.destroyer = None;
        assert!(matches!(
            unsafe { descriptor.validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));
        descriptor.destroyer = Some(asset_destroyer);
        descriptor.fields = std::ptr::dangling();
        assert_eq!(
            unsafe { descriptor.validate("plugin") }.err().unwrap(),
            ManifestError::InvalidPointerCount("asset fields")
        );
        let duplicate_asset_fields = [
            WXRAssetFieldDescriptor {
                name: c"field".as_ptr(),
                field_type: FieldType::U8 as u32,
                getter: Some(getter),
            },
            WXRAssetFieldDescriptor {
                name: c"field".as_ptr(),
                field_type: FieldType::U8 as u32,
                getter: Some(getter),
            },
        ];
        assert!(matches!(
            unsafe { asset(&duplicate_asset_fields).validate("plugin") },
            Err(ManifestError::DuplicateName {
                kind: "asset field",
                ..
            })
        ));
        descriptor.creator = Some(asset_creator);
        descriptor.destroyer = None;
        assert!(matches!(
            unsafe { descriptor.validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));

        let mut system_descriptor = system(&[]);
        system_descriptor.runner = None;
        assert!(matches!(
            unsafe { system_descriptor.validate("plugin") },
            Err(ManifestError::MissingCallback(_))
        ));
        system_descriptor = system(&[]);
        system_descriptor.entity_groups = std::ptr::dangling();
        assert_eq!(
            unsafe { system_descriptor.validate("plugin") }
                .err()
                .unwrap(),
            ManifestError::InvalidPointerCount("system entity groups")
        );
        let invalid_group = [WXRSystemEntityGroupDescriptor {
            components: std::ptr::dangling(),
            component_count: 0,
        }];
        assert_eq!(
            unsafe { system(&invalid_group).validate("plugin") }
                .err()
                .unwrap(),
            ManifestError::InvalidPointerCount("entity group components")
        );
        let duplicate_names = [c"component".as_ptr(), c"component".as_ptr()];
        let duplicate_group = [WXRSystemEntityGroupDescriptor {
            components: duplicate_names.as_ptr(),
            component_count: duplicate_names.len(),
        }];
        assert!(matches!(
            unsafe { system(&duplicate_group).validate("plugin") },
            Err(ManifestError::DuplicateName {
                kind: "entity group component",
                ..
            })
        ));
    }

    #[test]
    fn definitions_share_one_namespace_within_a_manifest() {
        let component = WXRComponentDescriptor {
            name: c"same".as_ptr(),
            ..component(&[], &[])
        };
        let asset = WXRAssetDescriptor {
            name: c"same".as_ptr(),
            ..asset(&[])
        };
        let descriptor = WXRPluginDescriptor {
            version: Version::CURRENT,
            name: c"plugin".as_ptr(),
            components: &component,
            component_count: 1,
            assets: &asset,
            asset_count: 1,
            systems: std::ptr::null(),
            system_count: 0,
        };
        assert_eq!(
            unsafe { ValidatedManifest::from_descriptor(&descriptor) }
                .err()
                .unwrap(),
            ManifestError::DuplicateDefinition("same".to_owned())
        );
    }
}
