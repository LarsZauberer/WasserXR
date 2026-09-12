use std::error::Error;

use wasserxr::definitions::error::{
    AssetDefinitionError, AssetFieldDefinitionError, ComponentDefinitionError,
    ComponentFieldDefinitionError, PluginDefinitionError,
};

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
