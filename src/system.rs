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

pub struct System {
    id: String,
    plugin_id: String,
    priority: usize,
    runner: Runner,
    groups: usize,
    selector: Option<Selector>,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
}

impl System {
    pub fn new(id: String, plugin: &Plugin, priority: usize) -> Result<Self, SystemError> {
        let plugin_id = plugin.get_id().to_owned();
        
        let runner_symbol = create_symbol("wxr_system_", &id);
        let groups_symbol = create_symbol("WXR_GROUPS_", &id.to_uppercase());
        let selector_symbol = create_symbol("wxr_select_", &id);
        let attacher_symbol = create_symbol("wxr_attach_", &id);
        let detacher_symbol = create_symbol("wxr_detach_", &id);

        let runner: Runner = plugin
            .get_symbol(&runner_symbol)
            .map_err(|error| match error {
                PluginError::MissingSymbol(sym) => SystemError::MissingSymbol(sym),
                _ => SystemError::FunctionError,
            })?;

        let groups: Option<*const usize> =
            System::map_symbol_option(plugin.get_symbol(&groups_symbol))?;
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
            System::map_symbol_option(plugin.get_symbol(&selector_symbol))?;
        let attacher: Option<Attacher> =
            System::map_symbol_option(plugin.get_symbol(&attacher_symbol))?;
        let detacher: Option<Detacher> =
            System::map_symbol_option(plugin.get_symbol(&detacher_symbol))?;

        Ok(Self {
            id,
            plugin_id,
            priority,
            runner,
            groups,
            selector,
            attacher,
            detacher,
        })
    }

    pub fn attach(&self, scene: &mut Scene) {
        if let Some(attacher) = self.attacher {
            unsafe {
                attacher(scene as *mut Scene);
            }
        }
    }

    pub fn detach(&self, scene: &mut Scene) {
        if let Some(detacher) = self.detacher {
            unsafe {
                detacher(scene as *mut Scene);
            }
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

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_priority(&self) -> usize {
        self.priority
    }
}

impl PartialOrd for System {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use super::*;

    #[test]
    fn test_nonexistent_runner_system() {
        let plugin = Plugin::new_static();
        let Err(err) = System::new("no_runner".to_owned(), &plugin, 0) else {
            panic!("System::new didn't throw an error");
        };
        assert_eq!(
            err,
            SystemError::MissingSymbol("wxr_system_no_runner".to_owned())
        );
    }

    static WITH_RUNNER_ATOMIC: AtomicUsize = AtomicUsize::new(0);

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_with_runner(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
        WITH_RUNNER_ATOMIC.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn test_with_runner_system_load() {
        let plugin = Plugin::new_static();
        let system = System::new("with_runner".to_owned(), &plugin, 0).unwrap();
        assert_eq!(system.groups, 0);
        assert!(system.selector.is_none());
        assert!(system.attacher.is_none());
        assert!(system.detacher.is_none());
    }

    #[test]
    fn test_with_runner_system_run() {
        WITH_RUNNER_ATOMIC.store(0, Ordering::SeqCst);
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new("with_runner".to_owned(), &plugin, 0).unwrap();
        system.run(&mut scene);
        assert_eq!(WITH_RUNNER_ATOMIC.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_nonexistent_system() {
        let plugin = Plugin::new_static();
        match System::new("nonexistent".to_owned(), &plugin, 100) {
            Ok(_) => {
                panic!("System should not exist");
            }
            Err(err) => match err {
                SystemError::MissingSymbol(symbol) => {
                    assert_eq!(symbol, "wxr_system_nonexistent");
                }
                _ => {
                    panic!("System creation threw an error which is not a MissingSymbol error");
                }
            },
        }
    }

    #[test]
    fn test_with_runner_system() {
        let plugin = Plugin::new_static();
        let system = System::new("with_runner".to_owned(), &plugin, 100)
            .expect("System should have been correctly created");

        assert_eq!(system.get_id(), "with_runner");
        assert_eq!(system.get_priority(), 100);
        assert_eq!(system.get_plugin_id(), plugin.get_id());
    }

    // Entity tests
    #[unsafe(no_mangle)]
    static WXR_GROUPS_ENTITY_SYSTEM: usize = 1;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_select_entity_system(scene: *const Scene, entity: *const u8) -> i32 {
        let scene = unsafe { &*scene };
        let entity: *const [u8; 16] = entity as *const [u8; 16];
        let uuid: Uuid = Uuid::from_bytes(unsafe { entity.read() });

        let entities = scene.get_entities();
        assert!(entities.contains(&&uuid));
        0
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_entity_system(
        scene: *mut Scene,
        entities: *const *const *const u8,
        groups: *const usize,
    ) {
        let scene = unsafe { &*scene };

        let group = unsafe { groups.read() };
        assert_eq!(group, 1);

        let entity: *const [u8; 16] = unsafe {
            let entity = entities.read().read();
            entity as *const [u8; 16]
        };
        let uuid: Uuid = Uuid::from_bytes(unsafe { entity.read() });

        let scene_entities = scene.get_entities();
        assert!(scene_entities.contains(&&uuid));
    }

    #[test]
    fn test_entity_system_system() {
        let mut scene = Scene::new();
        let _ = scene.add_entity(None);
        let plugin = Plugin::new_static();
        let system = System::new("entity_system".to_owned(), &plugin, 100)
            .expect("System should have been correctly created");
        system.run(&mut scene);
    }
}
