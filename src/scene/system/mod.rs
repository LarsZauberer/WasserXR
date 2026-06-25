use crate::{
    error::SystemError,
    scene::{Scene, plugin::Plugin, serialization::SystemData},
};

pub(crate) type Selector = unsafe extern "C" fn(*const Scene, *const u8) -> i32;
pub(crate) type Runner = unsafe extern "C" fn(*mut Scene, *const *const *const u8, *const usize);
pub(crate) type Attacher = unsafe extern "C" fn(*mut Scene);
pub(crate) type Detacher = unsafe extern "C" fn(*mut Scene);

// Default Selector

unsafe extern "C" fn default_selector(_scene: *const Scene, _entity: *const u8) -> i32 {
    0
}

unsafe extern "C" fn noop_attacher_detacher(_scene: *mut Scene) {}

pub(crate) struct System {
    // Metadata
    id: String,
    plugin_id: String,
    priority: usize,

    // Functions
    runner: Runner,
    groups: usize,
    selector: Selector,
    attacher: Attacher,
    detacher: Detacher,
}

impl System {
    pub(crate) fn new(
        id: String,
        plugin: &Plugin,
        priority: usize,
        scene: &Scene,
    ) -> Result<Self, SystemError> {
        let plugin_id = plugin.get_id().to_owned();

        let runner_symbol = "wxr_system_".to_owned() + &id;
        let groups_symbol = "WXR_GROUPS_".to_owned() + &id.to_uppercase();
        let selector_symbol = "wxr_select_".to_owned() + &id;
        let attacher_symbol = "wxr_attach_".to_owned() + &id;
        let detacher_symbol = "wxr_detach_".to_owned() + &id;

        let runner: Runner = plugin
            .get_symbol::<Runner>(&runner_symbol)
            .map_err(|error| {
                crate::debug!(scene, "System `{}` has no runner function", id);
                SystemError::NoSystemFunction(error)
            })?;

        let groups: usize = if let Ok(ptr) = plugin.get_symbol::<*const usize>(&groups_symbol) {
            unsafe { ptr.read() }
        } else {
            crate::debug!(scene, "System `{}` has no group amount", id);
            0
        };

        let selector: Selector = plugin
            .get_symbol::<Selector>(&selector_symbol)
            .unwrap_or_else(|_| {
                crate::debug!(scene, "System `{}` has no selector function", id);
                default_selector
            });
        let attacher: Attacher = plugin
            .get_symbol::<Attacher>(&attacher_symbol)
            .unwrap_or_else(|_| {
                crate::debug!(scene, "System `{}` has no attacher function", id);
                noop_attacher_detacher
            });
        let detacher: Detacher = plugin
            .get_symbol::<Detacher>(&detacher_symbol)
            .unwrap_or_else(|_| {
                crate::debug!(scene, "System `{}` has no detacher function", id);
                noop_attacher_detacher
            });

        crate::info!(scene, "System `{}` created", id);
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

    pub(crate) fn serialize(&self) -> SystemData {
        SystemData {
            id: self.id.clone(),
            priority: self.priority,
        }
    }

    pub(crate) fn deserialize(
        data: SystemData,
        plugin: &Plugin,
        scene: &Scene,
    ) -> Result<Self, SystemError> {
        Self::new(data.id, plugin, data.priority, scene)
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_priority(&self) -> usize {
        self.priority
    }

    pub(crate) fn get_attacher(&self) -> Attacher {
        self.attacher
    }

    pub(crate) fn get_detacher(&self) -> Detacher {
        self.detacher
    }

    pub(crate) fn get_selector(&self) -> Selector {
        self.selector
    }

    pub(crate) fn get_runner(&self) -> Runner {
        self.runner
    }

    pub(crate) fn get_groups(&self) -> usize {
        self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PluginError;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SYSTEM_ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SYSTEM_DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[unsafe(no_mangle)]
    static WXR_GROUPS_SYSTEM_WITH_GROUPS: usize = 3;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_system_with_groups(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_select_system_with_groups(
        _scene: *const Scene,
        _entity: *const u8,
    ) -> i32 {
        2
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_attach_system_with_groups(_scene: *mut Scene) {
        SYSTEM_ATTACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_detach_system_with_groups(_scene: *mut Scene) {
        SYSTEM_DETACH_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_system_with_defaults(
        _scene: *mut Scene,
        _entities: *const *const *const u8,
        _sizes: *const usize,
    ) {
    }

    #[test]
    fn system_new_with_all_symbols() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new("system_with_groups".to_owned(), &plugin, 7, &scene).unwrap();

        assert_eq!(system.get_id(), "system_with_groups");
        assert_eq!(system.get_plugin_id(), "");
        assert_eq!(system.get_priority(), 7);
        assert_eq!(system.get_groups(), 3);
        assert_eq!(
            system.get_selector() as usize,
            wxr_select_system_with_groups as *const () as usize
        );
        assert_eq!(
            system.get_attacher() as usize,
            wxr_attach_system_with_groups as *const () as usize
        );
        assert_eq!(
            system.get_detacher() as usize,
            wxr_detach_system_with_groups as *const () as usize
        );
    }

    #[test]
    fn system_new_without_optional_symbols() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new("system_with_defaults".to_owned(), &plugin, 1, &scene).unwrap();

        assert_eq!(system.get_groups(), 0);
        assert_eq!(
            unsafe { (system.get_selector())(std::ptr::null(), std::ptr::null()) },
            0
        );
    }

    #[test]
    fn system_new_without_runner() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();

        assert!(matches!(
            System::new("missing_runner".to_owned(), &plugin, 1, &scene),
            Err(SystemError::NoSystemFunction(PluginError::MissingSymbol(symbol)))
                if symbol == "wxr_system_missing_runner"
        ));
    }

    #[test]
    fn system_serialize_round_trip() {
        let scene = Scene::new();
        let plugin = Plugin::new_static();
        let system = System::new("system_with_defaults".to_owned(), &plugin, 7, &scene).unwrap();

        let deserialized = System::deserialize(system.serialize(), &plugin, &scene).unwrap();

        assert_eq!(deserialized.get_id(), "system_with_defaults");
        assert_eq!(deserialized.get_priority(), 7);
    }

    #[test]
    fn system_get_attacher_when_called() {
        let scene = Scene::new();
        SYSTEM_ATTACH_COUNT.store(0, Ordering::SeqCst);
        let plugin = Plugin::new_static();
        let system = System::new("system_with_groups".to_owned(), &plugin, 1, &scene).unwrap();

        unsafe {
            (system.get_attacher())(std::ptr::null_mut());
        }

        assert_eq!(SYSTEM_ATTACH_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn system_get_detacher_when_called() {
        let scene = Scene::new();
        SYSTEM_DETACH_COUNT.store(0, Ordering::SeqCst);
        let plugin = Plugin::new_static();
        let system = System::new("system_with_groups".to_owned(), &plugin, 1, &scene).unwrap();

        unsafe {
            (system.get_detacher())(std::ptr::null_mut());
        }

        assert_eq!(SYSTEM_DETACH_COUNT.load(Ordering::SeqCst), 1);
    }
}
