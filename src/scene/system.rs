use crate::scene::{Entity, Scene, plugin::Plugin};

pub type Selector = unsafe extern "C" fn(*const Scene, *const Entity) -> i32;
pub type Runner = unsafe extern "C" fn(*mut Scene, *const *const *mut Entity, *const usize);
pub type Attacher = unsafe extern "C" fn(*mut Scene);
pub type Detacher = unsafe extern "C" fn(*mut Scene);

pub struct System {
    id: String,
    priority: usize,
    selector: Option<Selector>,
    runner: Runner,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
    groups: usize,
}

impl System {
    pub fn new(scene: &mut Scene, plugin: &Plugin, id: &str, priority: usize) -> Option<Self> {
        let selector_symbol = "wxr_select_".to_owned() + id;
        let runner_symbol = "wxr_system_".to_owned() + id;
        let attacher_symbol = "wxr_attach_".to_owned() + id;
        let detacher_symbol = "wxr_detach_".to_owned() + id;
        let groups_symbol = "WXR_GROUPS_".to_owned() + &id.to_uppercase();

        let selector: Option<Selector> = plugin.get_abi_symbol(&selector_symbol);
        let runner: Option<Runner> = plugin.get_abi_symbol(&runner_symbol);
        let attacher: Option<Attacher> = plugin.get_abi_symbol(&attacher_symbol);
        let detacher: Option<Detacher> = plugin.get_abi_symbol(&detacher_symbol);
        let groups: Option<*const usize> = plugin.get_abi_symbol(&groups_symbol);

        // A system cannot exist without a system function
        if runner.is_none() {
            return None;
        }
        let runner = runner.unwrap();

        let groups = match groups {
            Some(ptr) => unsafe { ptr.read() },
            None => 0,
        };

        // Run Attacher
        if attacher.is_some() {
            let attacher = attacher.unwrap();
            unsafe { attacher(scene as *mut Scene) };
        }

        Some(Self {
            id: id.to_owned(),
            priority,
            groups,
            selector,
            runner,
            attacher,
            detacher,
        })
    }

    pub fn run(&mut self, scene: &mut Scene) {
        let scene_ptr = scene as *mut Scene;
        let mut groups = vec![Vec::<*mut Entity>::new(); self.groups];

        if self.selector.is_some() {
            let selector = self.selector.unwrap();
            for entity in scene.entities.iter_mut() {
                let group = unsafe { selector(scene_ptr as *const Scene, entity as *const Entity) };
                if group >= 0 {
                    groups[group as usize].push(entity as *mut Entity);
                }
            }
        }

        // Convert everything to a pointer
        let entities: Vec<*const *mut Entity> = groups.iter().map(|group| group.as_ptr()).collect();
        let sizes: Vec<usize> = groups.iter().map(|group| group.len()).collect();

        unsafe { (self.runner)(scene, entities.as_ptr(), sizes.as_ptr()) }
    }

    pub fn destroy(&self, scene: &mut Scene) {
        let scene_ptr = scene as *mut Scene;

        if self.detacher.is_some() {
            unsafe {
                let detacher = self.detacher.unwrap();
                detacher(scene_ptr);
            }
        }
    }

    pub fn get_id(&self) -> String {
        self.id.to_owned()
    }
}

impl PartialOrd for System {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_system_non_existent() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new(&mut scene, &plugin, "asdf", 100);
        assert!(system.is_none());
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_basic_system(
        _scene: *mut Scene,
        _entities: *const *const *mut Entity,
        _groups: *const usize,
    ) {
    }

    #[test]
    fn create_system() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new(&mut scene, &plugin, "basic_system", 100);
        assert!(system.is_some());
        let system = system.unwrap();
        system.destroy(&mut scene);
    }

    #[unsafe(no_mangle)]
    pub static WXR_GROUPS_SELECTING_SYSTEM: usize = 1;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_select_selecting_system(
        _scene: *const Scene,
        _entity: *const Entity,
    ) -> i32 {
        return 0;
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_selecting_system(
        _scene: *mut Scene,
        entities: *const *const *mut Entity,
        groups: *const usize,
    ) {
        assert!(!entities.is_null());
        assert!(!groups.is_null());

        unsafe {
            assert_eq!(*groups, 1);

            let first_group = *entities;
            assert!(!first_group.is_null());

            let first_entity = *first_group;
            assert!(!first_entity.is_null());
            assert_eq!(*first_entity, 0);
        }
    }

    #[test]
    fn run_system() {
        let mut scene = Scene::new();
        let plugin = Plugin::new_static();
        let _entity = scene.add_entity();
        let system = System::new(&mut scene, &plugin, "selecting_system", 100);
        assert!(system.is_some());
        let mut system = system.unwrap();
        system.run(&mut scene);
        system.destroy(&mut scene);
    }
}
