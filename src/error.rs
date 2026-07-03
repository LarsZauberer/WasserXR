//! Error types returned by the WasserXR runtime APIs.

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors raised while loading a dynamic or static plugin symbol.
pub enum PluginError {
    /// The plugin file could not be linked by the platform loader.
    LinkingError(String),
    /// A required exported symbol was missing.
    MissingSymbol(String),
    /// The requested plugin file was not found.
    NotFound,
    /// A symbol name could not be represented for lookup.
    InvalidSymbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors raised while creating a system from plugin bindings.
pub enum SystemError {
    /// The system runner symbol could not be resolved.
    NoSystemFunction(PluginError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors raised while creating, querying, serializing, or editing components.
pub enum ComponentError {
    /// A schema field id was not registered.
    FieldNotFound,
    /// A schema field has no getter function.
    FieldNoGetter,
    /// A mutable query requested a field that was not marked mutable.
    FieldNotMutable,
    /// A schema field has no serializer function.
    FieldNoSerializer,
    /// A schema field has no deserializer function.
    FieldNoDeserializer,
    /// The component creator symbol could not be resolved.
    NoCreator(PluginError),
    /// The component destroyer symbol could not be resolved.
    NoDestroyer(PluginError),
    /// The query field list does not match the requested typed query.
    FieldParsing,
    /// A field value could not be rendered or parsed as text.
    FieldValueParsing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors raised while registering or querying asset types and asset fields.
pub enum AssetError {
    /// A schema field id was not registered.
    FieldNotFound,
    /// A schema field has no getter function.
    FieldNoGetter,
    /// The query field list does not match the requested typed query.
    FieldParsing,
    /// An asset creator returned null or an asset lookup failed.
    InvalidAsset,
    /// The asset creator symbol could not be resolved.
    NoCreator(PluginError),
    /// The asset schema symbol could not be resolved.
    NoSchema(PluginError),
    /// The asset destroyer symbol could not be resolved.
    NoDestroyer(PluginError),
    /// No loaded plugin exposes this asset type.
    AssetTypeNotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Errors returned by high-level `Scene` operations.
pub enum SceneError {
    /// The entity id is not present in the scene.
    EntityNotFound,
    /// The entity already has a component with this id.
    ComponentAlreadyExists,
    /// A resource with this name is already registered.
    ResourceAlreadyExists,
    /// A system with this id is already registered.
    SystemAlreadyExists,
    /// A plugin with this path is already loaded.
    PluginAlreadyLoaded,
    /// The system id is not present in the scene.
    SystemNotFound,
    /// The resource name is not present in the scene.
    ResourceNotFound,
    /// The plugin id or path is not present in the scene.
    PluginNotFound,
    /// The built-in static plugin cannot be unloaded.
    StaticPluginUnload,
    /// The component id is not present on the requested entity.
    ComponentNotFound,
    /// A component field operation failed.
    ComponentFieldError(ComponentError),
    /// An asset operation failed.
    AssetError(AssetError),
    /// Loading a plugin failed.
    PluginLoading(PluginError),
    /// A system could not be created from any loaded plugin.
    SystemCreation,
    /// Component symbols could not be resolved from any loaded plugin.
    ComponentCreation,
    /// The component creator returned a null pointer.
    ComponentCreatorFailed,
    /// Scene serialization failed.
    Serialization(String),
    /// Scene deserialization failed.
    Deserialization(String),
    /// Reading or writing a scene file failed.
    FileIo(String),
}
