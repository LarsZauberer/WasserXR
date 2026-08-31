use std::ffi::c_void;

use crate::{private::manifests::components::ComponentManifest, scene::PluginID};

#[derive(Debug)]
pub struct Component {
    plugin_id: PluginID,
    manifest: ComponentManifest,
    data: *mut c_void,
}
