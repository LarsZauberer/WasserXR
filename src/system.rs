use uuid::Uuid;

use crate::{
    error::{PluginError, SystemError},
    plugin::{Plugin, create_symbol},
    scene::Scene,
};

pub type Selector = unsafe extern "C" fn(*const Scene, *const u8) -> i32;
pub type Runner = unsafe extern "C" fn(*mut Scene, *const *const *const u8, *const usize);
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
        let Some(attacher) = self.attacher else {
            return;
        };

        unsafe {
            attacher(scene as *mut Scene);
        }
    }

    pub fn detach(&self, scene: &mut Scene) {
        let Some(detacher) = self.detacher else {
            return;
        };

        unsafe {
            detacher(scene as *mut Scene);
        }
    }

    pub fn run(&self, scene: &mut Scene) {
        let mut entities: Vec<Vec<Uuid>> = vec![Vec::new(); self.groups];

        if let Some(selector) = self.selector {
            let full_entities = scene.get_entities();
            for (entity, group) in full_entities.iter().map(|id| {
                (id, unsafe {
                    selector(scene as *const Scene, id.as_bytes() as *const u8)
                })
            }) {
                if group < 0 {
                    continue;
                }
                entities[group as usize].push(**entity);
            }
        }

        let sizes: Vec<usize> = entities.iter().map(|vec| vec.len()).collect();

        let entities_ptr: Vec<Vec<*const u8>> = entities
            .iter()
            .map(|vec| vec.iter().map(|id| id.as_bytes() as *const u8).collect())
            .collect();
        let group_ptr: Vec<*const *const u8> =
            entities_ptr.iter().map(|vec| vec.as_ptr()).collect();
        let sizes_ptr = sizes.as_ptr();

        unsafe {
            (self.runner)(scene as *mut Scene, group_ptr.as_ptr(), sizes_ptr);
        }
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
        let plugin_id = plugin.get_id().to_owned();
        let functions = SystemFunctions::new(&id, plugin)?;

        Ok(Self {
            id,
            plugin_id,
            priority,
            functions,
        })
    }

    pub fn get_functions(&self) -> SystemFunctions {
        self.functions
    }

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn get_id(&self) -> &str {
        &self.id
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
