use crate::error::SceneError;
use crate::scene::Scene;

use std::collections::hash_map::Entry;
use std::ffi::c_void;

pub struct Resource {
    data: *mut c_void,
    dropper: unsafe fn(*mut c_void),
}

impl Resource {
    pub fn new<T>(value: T) -> Self {
        Self {
            data: Box::into_raw(Box::new(value)).cast(),
            dropper: drop_value::<T>,
        }
    }

    pub fn get<T>(&self) -> &T {
        unsafe { &*self.data.cast::<T>() }
    }

    pub fn get_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *self.data.cast::<T>() }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe { (self.dropper)(self.data) };
    }
}

unsafe fn drop_value<T>(data: *mut c_void) {
    unsafe {
        drop(Box::from_raw(data.cast::<T>()));
    }
}

impl Scene {
    pub fn add_resource<T>(&mut self, name: String, value: T) -> Result<(), SceneError> {
        match self.resources.entry(name) {
            Entry::Occupied(_) => Err(SceneError::ResourceAlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(Resource::new(value));
                Ok(())
            }
        }
    }

    pub fn get_resource<T>(&self, name: &str) -> Result<&T, SceneError> {
        self.resources
            .get(name)
            .map(Resource::get)
            .ok_or(SceneError::ResourceNotFound)
    }

    pub fn get_mut_resource<T>(&mut self, name: &str) -> Result<&mut T, SceneError> {
        self.resources
            .get_mut(name)
            .map(Resource::get_mut)
            .ok_or(SceneError::ResourceNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct CountDrop(Arc<AtomicUsize>);

    impl Drop for CountDrop {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn resource_returns_stored_value() {
        let mut resource = Resource::new(1usize);

        assert_eq!(*resource.get::<usize>(), 1);
        *resource.get_mut::<usize>() = 2;
        assert_eq!(*resource.get::<usize>(), 2);
    }

    #[test]
    fn resource_drops_stored_value() {
        let drops = Arc::new(AtomicUsize::new(0));

        {
            let _resource = Resource::new(CountDrop(Arc::clone(&drops)));
            assert_eq!(drops.load(Ordering::SeqCst), 0);
        }

        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scene_adds_and_gets_resource() {
        let mut scene = Scene::new();

        scene.add_resource("counter".to_owned(), 1usize).unwrap();
        *scene.get_mut_resource::<usize>("counter").unwrap() = 2;

        assert_eq!(*scene.get_resource::<usize>("counter").unwrap(), 2);
    }

    #[test]
    fn scene_rejects_duplicate_resource() {
        let mut scene = Scene::new();
        scene.add_resource("counter".to_owned(), 1usize).unwrap();

        assert_eq!(
            scene.add_resource("counter".to_owned(), 2usize),
            Err(SceneError::ResourceAlreadyExists)
        );
    }

    #[test]
    fn scene_rejects_missing_resource() {
        let scene = Scene::new();

        assert_eq!(
            scene.get_resource::<usize>("missing"),
            Err(SceneError::ResourceNotFound)
        );
    }

    #[test]
    fn scene_resource_survives_reload() {
        let mut scene = Scene::new();
        scene.add_resource("counter".to_owned(), 1usize).unwrap();

        scene.reload().unwrap();

        assert_eq!(*scene.get_resource::<usize>("counter").unwrap(), 1);
    }
}
