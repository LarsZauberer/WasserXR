#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginError {
    LinkingError(String),
    MissingSymbol(String),
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
    FieldNoSetter,
    NoCreator(PluginError),
    NoDestroyer(PluginError),
    FieldParsing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    EntityNotFound,
    ComponentAlreadyExists,
    SystemAlreadyExists,
    PluginAlreadyLoaded,
    SystemNotFound,
    PluginNotFound,
    StaticPluginUnload,
    ComponentNotFound,
    ComponentFieldError(ComponentError),
    PluginLoading(PluginError),
    SystemCreation,
    ComponentCreation,
}
