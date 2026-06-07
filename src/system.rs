use crate::{
    entity::Entity,
    error::{PluginError, SystemError},
    plugin::{Plugin, create_symbol},
    scene::Scene,
};

pub type Selector = unsafe extern "C" fn(*const Scene, *const Entity) -> i32;
pub type Runner = unsafe extern "C" fn(*mut Scene, *const *const *mut Entity, *const usize);
pub type Attacher = unsafe extern "C" fn(*mut Scene);
pub type Detacher = unsafe extern "C" fn(*mut Scene);

#[derive(Copy, Clone)]
pub struct SystemFunctions {
    runner: Runner,
    groups: usize,
    selector: Option<Selector>,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
}

impl SystemFunctions {
    pub fn new(id: &str, plugin: &Plugin) -> Result<Self, SystemError> {
        let runner_symbol = create_symbol("wxr_system_", id);
        let groups_symbol = create_symbol("WXRGroups", id);
        let selector_symbol = create_symbol("wxr_select_", id);
        let attacher_symbol = create_symbol("wxr_attach_", id);
        let detacher_symbol = create_symbol("wxr_detach_", id);

        let runner: Runner = plugin
            .get_symbol(&runner_symbol)
            .map_err(|error| match error {
                PluginError::MissingSymbol(sym) => SystemError::MissingSymbol(sym),
                _ => SystemError::FunctionError,
            })?;

        let groups: Option<*const usize> =
            SystemFunctions::map_symbol_option(plugin.get_symbol(&groups_symbol))?;
        let groups = match groups {
            Some(ptr) => {
                if ptr.is_null() {
                    0
                } else {
                    unsafe { ptr.read() }
                }
            }
            None => 0,
        };

        let selector: Option<Selector> =
            SystemFunctions::map_symbol_option(plugin.get_symbol(&selector_symbol))?;
        let attacher: Option<Attacher> =
            SystemFunctions::map_symbol_option(plugin.get_symbol(&attacher_symbol))?;
        let detacher: Option<Detacher> =
            SystemFunctions::map_symbol_option(plugin.get_symbol(&detacher_symbol))?;

        Ok(Self {
            runner,
            groups,
            selector,
            attacher,
            detacher,
        })
    }

    pub fn attach(&self, scene: &mut Scene) {
        todo!()
    }

    pub fn detach(&self, scene: &mut Scene) {
        todo!()
    }

    pub fn run(&self, scene: &mut Scene) {
        todo!()
    }

    fn map_symbol_option<T>(res: Result<T, PluginError>) -> Result<Option<T>, SystemError> {
        match res {
            Ok(obj) => Ok(Some(obj)),
            Err(PluginError::MissingSymbol(msg)) => {
                log::warn!("Symbol: {} not defined", msg);
                Ok(None)
            }
            Err(_) => Err(SystemError::FunctionError),
        }
    }
}

pub struct System {
    id: String,
    plugin_id: String,
    priority: usize,
    functions: SystemFunctions,
}

impl System {
    pub fn new(id: String, plugin: &Plugin, priority: usize) -> Result<Self, SystemError> {
        todo!()
    }

    pub fn get_functions(&self) -> SystemFunctions {
        self.functions
    }
}

impl PartialOrd for System {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Ord for System {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl Eq for System {}
