use std::ffi::CString;

use libc::{RTLD_LAZY, dlopen, dlsym};

use crate::{
    definitions::plugins::PluginDefinition,
    private::{
        io::{FileIO, PluginIO},
        plugins::error::PluginError,
    },
};

/// The local disk file system that will become an IO provider for different operations
pub(crate) struct LocalFileSystem {}

impl FileIO for LocalFileSystem {
    type Error = std::io::Error;

    fn copy(src: &std::path::Path, dst: &std::path::Path) -> Result<(), Self::Error> {
        std::fs::copy(src, dst)?;
        Ok(())
    }
}

impl PluginIO for LocalFileSystem {
    type Error = PluginError;

    unsafe fn get_plugin_definition(
        src: &std::path::Path,
    ) -> Result<crate::definitions::plugins::PluginDefinition, Self::Error> {
        let c_string = CString::new(src.to_str().unwrap()).unwrap();

        let handle = unsafe { dlopen(c_string.as_ptr(), RTLD_LAZY) };

        if handle.is_null() {
            return Err(PluginError::FailedToOpenPlugin);
        }

        let plugin_symbol_name = CString::new("wxr_plugin").unwrap();
        let plugin = unsafe { dlsym(handle, plugin_symbol_name.as_ptr()) };

        if plugin.is_null() {
            return Err(PluginError::FailedToFindPluginDefinition);
        }

        let plugin = unsafe { plugin.cast::<PluginDefinition>().read() };

        Ok(plugin)
    }
}
