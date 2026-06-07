#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginError {
    LinkingError(String),
    MissingSymbol(String),
    InvalidSymbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityError {
    ComponentAlreadyExists,
    ComponentNotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemError {
    NoSystemFunction,
    MissingSymbol(String),
    FunctionError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentError {
    FieldNotFound,
    FieldNoGetter,
    FieldNoSetter,
    NoCreator,
    NoDestroyer,
    FieldParsing,
    FunctionError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    EntityNotFound,
    ComponentAlreadyExists,
    SystemAlreadyExists,
    PluginAlreadyLoaded,
    SystemNotFound,
    PluginNotFound,
    ComponentNotFound,
    ComponentCreation,
    SystemCreation,
    PluginLoading(PluginError),
    ComponentFieldError(ComponentError),
}
