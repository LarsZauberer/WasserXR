#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginError {
    LinkingError(String),
    MissingSymbol(String),
    NotFound,
    InvalidSymbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemError {
    NoSystemFunction(PluginError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentError {
    FieldNotFound,
    FieldNoGetter,
    FieldNotMutable,
    FieldNoSerializer,
    FieldNoDeserializer,
    NoCreator(PluginError),
    NoDestroyer(PluginError),
    FieldParsing,
    FieldValueParsing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetError {
    FieldNotFound,
    FieldNoGetter,
    FieldParsing,
    InvalidAsset,
    NoCreator(PluginError),
    NoSchema(PluginError),
    NoDestroyer(PluginError),
    AssetTypeNotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    EntityNotFound,
    ComponentAlreadyExists,
    ResourceAlreadyExists,
    SystemAlreadyExists,
    PluginAlreadyLoaded,
    SystemNotFound,
    ResourceNotFound,
    PluginNotFound,
    StaticPluginUnload,
    ComponentNotFound,
    ComponentFieldError(ComponentError),
    AssetError(AssetError),
    PluginLoading(PluginError),
    SystemCreation,
    ComponentCreation,
    Serialization(String),
    Deserialization(String),
    FileIo(String),
}
