use std::{error::Error, ffi::c_void};

use rstest::{fixture, rstest};
use wasserxr::{
    definitions::{
        Definition,
        components::ComponentDefinition,
        error::{
            AssetFieldDefinitionError, ComponentDefinitionError, ComponentFieldDefinitionError,
            PluginDefinitionError,
        },
        fields::{AssetFieldDefinition, ComponentFieldDefinition},
        plugins::PluginDefinition,
    },
    utils::version::Version,
};

unsafe extern "C" fn getter(_: *const c_void) -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn serializer(_: *const c_void) {}

unsafe extern "C" fn deserializer(_: *const c_void) {}

unsafe extern "C" fn creator() -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn destroyer(_: *mut c_void) {}

mod component_fields {
    use super::*;

    #[fixture]
    fn field() -> ComponentFieldDefinition {
        static NAME: &[u8] = b"position\0";

        ComponentFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
            mutable: 0,
            serializer: Some(serializer),
            deserializer: Some(deserializer),
        }
    }

    #[rstest]
    fn validates_field(field: ComponentFieldDefinition) {
        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_mutable_field_without_getter(mut field: ComponentFieldDefinition) {
        field.getter = None;
        field.mutable = 1;

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::MutableButNoGetter(
                "position".to_owned()
            ))
        );
    }

    #[rstest]
    fn field_without_serializer_is_not_serializable(mut field: ComponentFieldDefinition) {
        field.serializer = None;

        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn field_without_deserializer_is_not_deserializable(mut field: ComponentFieldDefinition) {
        field.deserializer = None;

        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_null_name(mut field: ComponentFieldDefinition) {
        field.name = std::ptr::null();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsNull)
        );
    }

    #[rstest]
    fn rejects_invalid_utf8_name(mut field: ComponentFieldDefinition) {
        let name = [0xff_u8, 0];
        field.name = name.as_ptr().cast();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsNotUtf8)
        );
    }

    #[rstest]
    fn rejects_empty_name(mut field: ComponentFieldDefinition) {
        let name = [0_u8];
        field.name = name.as_ptr().cast();

        assert_eq!(
            unsafe { field.validate() },
            Err(ComponentFieldDefinitionError::NameIsEmpty)
        );
    }
}

mod asset_fields {
    use super::*;

    #[fixture]
    fn field() -> AssetFieldDefinition {
        static NAME: &[u8] = b"material\0";

        AssetFieldDefinition {
            name: NAME.as_ptr().cast(),
            getter: Some(getter),
        }
    }

    #[rstest]
    fn validates_field(field: AssetFieldDefinition) {
        assert!(unsafe { field.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_missing_getter(mut field: AssetFieldDefinition) {
        field.getter = None;

        assert_eq!(
            unsafe { field.validate() },
            Err(AssetFieldDefinitionError::GetterIsNull(
                "material".to_owned()
            ))
        );
    }
}

mod components {
    use super::*;

    #[fixture]
    fn component() -> ComponentDefinition {
        static NAME: &[u8] = b"Transform\0";

        ComponentDefinition {
            name: NAME.as_ptr().cast(),
            creator: Some(creator),
            destroyer: Some(destroyer),
            fields: std::ptr::null(),
            field_count: 0,
        }
    }

    #[rstest]
    fn validates_component(component: ComponentDefinition) {
        assert!(unsafe { component.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_null_name(mut component: ComponentDefinition) {
        component.name = std::ptr::null();
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::NameIsNull)
        );
    }

    #[rstest]
    fn rejects_invalid_utf8_name(mut component: ComponentDefinition) {
        let name = [0xff_u8, 0];
        component.name = name.as_ptr().cast();
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::NameIsNotUtf8)
        );
    }

    #[rstest]
    fn rejects_empty_name(mut component: ComponentDefinition) {
        let name = [0_u8];
        component.name = name.as_ptr().cast();
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::NameIsEmpty)
        );
    }

    #[rstest]
    fn rejects_missing_creator(mut component: ComponentDefinition) {
        component.creator = None;
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::CreatorIsNull(
                "Transform".to_owned()
            ))
        );
    }

    #[rstest]
    fn rejects_missing_destroyer(mut component: ComponentDefinition) {
        component.destroyer = None;
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::DestroyerIsNull(
                "Transform".to_owned()
            ))
        );
    }

    #[rstest]
    fn rejects_missing_fields(mut component: ComponentDefinition) {
        component.field_count = 1;
        assert_eq!(
            unsafe { component.validate() },
            Err(ComponentDefinitionError::FieldsIsNull(
                "Transform".to_owned()
            ))
        );
    }
}

mod plugins {
    use super::*;

    fn compatible_version() -> Version {
        Version {
            major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        }
    }

    #[fixture]
    fn plugin() -> PluginDefinition {
        static NAME: &[u8] = b"example\0";

        PluginDefinition {
            name: NAME.as_ptr().cast(),
            engine_version: compatible_version(),
            components: std::ptr::null(),
            component_count: 0,
        }
    }

    #[rstest]
    fn validates_plugin(plugin: PluginDefinition) {
        assert!(unsafe { plugin.validate() }.is_ok());
    }

    #[rstest]
    fn rejects_incompatible_engine_version(mut plugin: PluginDefinition) {
        let incompatible_version = if plugin.engine_version.major == 0 {
            Version {
                minor: plugin.engine_version.minor + 1,
                ..plugin.engine_version
            }
        } else {
            Version {
                major: plugin.engine_version.major + 1,
                ..plugin.engine_version
            }
        };
        plugin.engine_version = incompatible_version;

        assert_eq!(
            unsafe { plugin.validate() },
            Err(PluginDefinitionError::EngineVersionMismatch {
                name: "example".to_owned(),
                expected: compatible_version(),
                actual: incompatible_version,
            })
        );
    }

    #[rstest]
    fn rejects_missing_components(mut plugin: PluginDefinition) {
        plugin.component_count = 1;

        assert_eq!(
            unsafe { plugin.validate() },
            Err(PluginDefinitionError::ComponentsIsNull(
                "example".to_owned()
            ))
        );
    }
}

mod errors {
    use super::*;

    #[test]
    fn formats_nested_definition_errors() {
        let error = PluginDefinitionError::ComponentInvalid(
            "example".to_owned(),
            ComponentDefinitionError::FieldInvalid(
                "Transform".to_owned(),
                ComponentFieldDefinitionError::MutableButNoGetter("position".to_owned()),
            ),
        );

        assert_eq!(
            error.to_string(),
            "plugin 'example' has an invalid component: component 'Transform' has an invalid field: mutable component field 'position' has no getter"
        );
        assert!(error.source().is_some());
    }

    #[test]
    fn converts_nested_errors_with_context() {
        let component_error: ComponentDefinitionError =
            ("Transform", ComponentFieldDefinitionError::NameIsNull).into();
        let plugin_error: PluginDefinitionError = ("example", component_error).into();

        assert_eq!(
            plugin_error.to_string(),
            "plugin 'example' has an invalid component: component 'Transform' has an invalid field: component field name is null"
        );
    }

    #[test]
    fn converts_component_errors_with_explicit_plugin_context() {
        let plugin_error: PluginDefinitionError = (
            "example",
            ComponentDefinitionError::FieldsIsNull("Transform".to_owned()),
        )
            .into();
        assert_eq!(
            plugin_error,
            PluginDefinitionError::ComponentInvalid(
                "example".to_owned(),
                ComponentDefinitionError::FieldsIsNull("Transform".to_owned()),
            )
        );
    }
}
