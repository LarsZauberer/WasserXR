use crate::{entity::Entity, error::WXRError, plugin::Plugin, scene::Scene};

pub type Selector = unsafe extern "C" fn(*const Scene, *const Entity) -> i32;
pub type Runner = unsafe extern "C" fn(*mut Scene, *const *const *mut Entity, *const usize);
pub type Attacher = unsafe extern "C" fn(*mut Scene);
pub type Detacher = unsafe extern "C" fn(*mut Scene);

#[derive(Copy, Clone)]
pub struct SystemFunctions {
    runner: Runner,
    selector: Option<Selector>,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
}

impl SystemFunctions {
    pub fn new(id: &str, plugin: &Plugin) -> Result<Self, WXRError> {
        todo!()
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
}

pub struct System {
    id: String,
    plugin_id: String,
    priority: usize,
    functions: SystemFunctions,
}

impl System {
    pub fn new(id: String, plugin: &Plugin, priority: usize) -> Result<Self, WXRError> {
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
