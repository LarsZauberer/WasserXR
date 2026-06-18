use crate::{
    error::SystemError,
    scene::{Scene, plugin::Plugin},
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
    pub(crate) fn new(id: String, plugin: &Plugin, priority: usize) -> Result<Self, SystemError> {
        let plugin_id = plugin.get_id().to_owned();

        let runner_symbol = "wxr_system_".to_owned() + &id;
        let groups_symbol = "WXR_GROUPS_".to_owned() + &id.to_uppercase();
        let selector_symbol = "wxr_select_".to_owned() + &id;
        let attacher_symbol = "wxr_attach_".to_owned() + &id;
        let detacher_symbol = "wxr_detach_".to_owned() + &id;

        let runner = plugin
            .get_symbol(&runner_symbol)
            .map_err(SystemError::NoSystemFunction)?;

        let groups = if let Ok(ptr) = plugin.get_symbol::<*const usize>(&groups_symbol) {
            unsafe { ptr.read() }
        } else {
            log::debug!("No group amount was specified for system: {}", id);
            0
        };

        let selector = plugin.get_symbol(&selector_symbol).unwrap_or_else(|_| {
            log::debug!("No selector was specified for system: {}", id);
            default_selector
        });
        let attacher = plugin.get_symbol(&attacher_symbol).unwrap_or_else(|_| {
            log::debug!("No attacher was specified for system: {}", id);
            noop_attacher_detacher
        });
        let detacher = plugin.get_symbol(&detacher_symbol).unwrap_or_else(|_| {
            log::debug!("No detacher was specified for system: {}", id);
            noop_attacher_detacher
        });

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
mod tests {}
