pub enum PluginError {
    LinkingError(String),
    MissingSymbol(String),
    InvalidSymbol,
}

pub enum EntityError {
    ComponentAlreadyExists,
    ComponentNotFound,
}

pub enum SystemError {
    NoSystemFunction,
    MissingSymbol(String),
    FunctionError,
}

pub enum ComponentError {
    FieldNotFound,
    FieldNoGetter,
    FieldNoSetter,
    NoCreator,
    NoDestroyer,
    FieldParsing,
    FunctionError,
}

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
