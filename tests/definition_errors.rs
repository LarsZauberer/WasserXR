use std::error::Error;

use wasserxr::definitions::error::{
    AssetDefinitionError, AssetFieldDefinitionError, ComponentDefinitionError,
    ComponentFieldDefinitionError, PluginDefinitionError,
};
use wasserxr::errors::PluginError;

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
fn formats_nested_asset_definition_errors() {
    let error = PluginDefinitionError::AssetInvalid(
        "example".to_owned(),
        AssetDefinitionError::FieldInvalid(
            "Mesh".to_owned(),
            AssetFieldDefinitionError::GetterIsNull("vertices".to_owned()),
        ),
    );

    assert_eq!(
        error.to_string(),
        "plugin 'example' has an invalid asset: asset 'Mesh' has an invalid field: asset field 'vertices' has no getter"
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

#[test]
fn formats_plugin_errors() {
    assert_eq!(
        PluginError::FailedToOpenPlugin.to_string(),
        "failed to open plugin"
    );
    assert_eq!(
        PluginError::FailedToFindPluginDefinition.to_string(),
        "failed to find plugin definition"
    );
    assert_eq!(
        PluginError::IOError(std::io::Error::other("permission denied")).to_string(),
        "I/O error: permission denied"
    );

    let error = PluginError::DefinitionValidationError(PluginDefinitionError::NameIsEmpty);
    assert_eq!(
        error.to_string(),
        "plugin definition validation error: plugin name is empty"
    );
    assert!(error.source().is_some());
}
