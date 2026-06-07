pub enum WXRError {
    LinkingError,
    MissingSymbol,
    PluginAlreadyLoaded,
    SystemAlreadyLoaded,
    PluginNotFound,
    SystemNotFound,
    Other,
}

pub enum EntityError {
    ComponentExists,
}

pub enum ComponentError {
    FieldNotFound,
    FieldNoGetter,
    FieldNoSetter,
    NoCreator,
    NoDestroyer,
    Other,
}

pub enum SceneError<T> {
    EntityNotFound,
    ComponentAlreadyExists,
    Other(T),
}
