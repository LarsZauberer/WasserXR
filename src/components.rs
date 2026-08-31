use std::ffi::c_void;

use crate::{private::manifests::components::ComponentManifest, scene::PluginID};

#[derive(Debug)]
pub struct Component {
    plugin_id: PluginID,
    manifest: ComponentManifest,
    data: *mut c_void,
}

impl Component {
    pub fn new(plugin_id: PluginID, manifest: ComponentManifest) -> Self {
        let data = unsafe { (manifest.creator)() };
        Self {
            plugin_id,
            manifest,
            data,
        }
    }

    pub fn get_field(&self, name: &str) -> Option<*const c_void> {
        todo!()
    }

    pub fn get_mut_field(&self, name: &str) -> Option<*mut c_void> {
        todo!()
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { (self.manifest.destroyer)(self.data) }
    }
}
