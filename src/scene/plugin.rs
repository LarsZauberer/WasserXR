use core::{ffi::c_void, ptr::null_mut};
use std::ffi::CString;

use crate::scene::{Scene, system::System};

pub struct Plugin {
    path: Option<String>,
    fd: *mut c_void,

    systems: Vec<System>,
}

impl Plugin {
    pub fn new(path: &str) -> Option<Self> {
        // Create the symbol
        let Ok(convert_path) = CString::new(path.to_owned()) else {
            return None;
        };

        // Open the library
        let fd: *mut c_void = unsafe { libc::dlopen(convert_path.as_ptr(), libc::RTLD_NOW) };
        if fd.is_null() {
            return None;
        }

        Some(Plugin {
            path: Some(path.to_owned()),
            fd: fd,
            systems: Vec::new(),
        })
    }

    pub fn new_static() -> Self {
        Plugin {
            path: None,
            fd: libc::RTLD_DEFAULT,
            systems: Vec::new(),
        }
    }

    pub fn get_abi_symbol<T>(&self, symbol: &str) -> Option<T> {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());

        let Ok(symbol_cstring) = CString::new(symbol.to_owned()) else {
            return None;
        };

        // Safety: Will return either null or will return the function pointer
        let ptr = unsafe { libc::dlsym(self.fd, symbol_cstring.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        let func: T = unsafe { std::mem::transmute_copy(&ptr) };
        Some(func)
    }

    pub fn get_systems(&self) -> Vec<&System> {
        self.systems.iter().map(|x| x).collect()
    }

    pub fn get_systems_mut(&mut self) -> Vec<&mut System> {
        self.systems.iter_mut().collect()
    }

    pub fn system_exists(&self, id: &str) -> bool {
        let res = self.systems.iter().find(|x| x.get_id() == id);
        res.is_some()
    }

    pub fn add_system(&mut self, scene: &mut Scene, id: &str, priority: usize) -> bool {
        if self.system_exists(id) {
            return false;
        }

        let system = System::new(scene, self, id, priority);

        match system {
            Some(system) => {
                self.systems.push(system);
                true
            }
            None => false,
        }
    }

    pub fn remove_system(&mut self, scene: &mut Scene, id: &str) -> bool {
        let Some(index) = self.systems.iter().position(|system| system.get_id() == id) else {
            return false;
        };

        let system = self.systems.remove(index);
        system.destroy(scene);
        true
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        if self.fd.is_null() {
            return;
        }

        unsafe {
            libc::dlclose(self.fd);
        }
        self.fd = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Entity;

    #[unsafe(no_mangle)]
    unsafe extern "C" fn my_func(my_num: usize) -> usize {
        my_num * 5
    }

    type TestFunction = fn(usize) -> usize;

    #[test]
    fn load_static_plugin() {
        let plugin = Plugin::new_static();
        let func: Option<TestFunction> = plugin.get_abi_symbol("my_func");
        assert!(func.is_some());

        let func: unsafe extern "C" fn(usize) -> usize = unsafe { std::mem::transmute(func) };
        let res = unsafe { func(5) };

        assert_eq!(res, 25);
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_system_plugin_test_system(
        _scene: *mut Scene,
        _entities: *const *const *mut Entity,
        _groups: *const usize,
    ) {
    }

    #[test]
    fn add_and_remove_system() {
        let mut scene = Scene::new();
        let mut plugin = Plugin::new_static();

        assert!(plugin.add_system(&mut scene, "plugin_test_system", 100));
        assert!(plugin.system_exists("plugin_test_system"));

        assert!(plugin.remove_system(&mut scene, "plugin_test_system"));
        assert!(!plugin.system_exists("plugin_test_system"));
    }

    #[test]
    fn add_same_system_twice() {
        let mut scene = Scene::new();
        let mut plugin = Plugin::new_static();

        assert!(plugin.add_system(&mut scene, "plugin_test_system", 100));
        assert!(!plugin.add_system(&mut scene, "plugin_test_system", 100));
        assert_eq!(plugin.get_systems().len(), 1);
    }

    #[test]
    fn add_non_existent_system() {
        let mut scene = Scene::new();
        let mut plugin = Plugin::new_static();

        assert!(!plugin.add_system(&mut scene, "does_not_exist", 100));
        assert!(plugin.get_systems().is_empty());
    }
}
