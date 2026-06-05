// use crate::scene::{Entity, Scene};
//
// #[unsafe(no_mangle)]
// pub extern "C" fn wxr_create_scene() -> *mut Scene {
//     Box::into_raw(Box::new(Scene::new()))
// }
//
// /// # Safety
// ///
// /// `scene` must be a pointer returned by `wxr_create_scene` that has not already been destroyed.
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn wxr_destroy_scene(scene: *mut Scene) {
//     if scene.is_null() {
//         return;
//     }
//
//     unsafe {
//         drop(Box::from_raw(scene));
//     }
// }
//
// /// # Safety
// ///
// /// `scene` must be null or a valid, uniquely borrowed `WXRScene` pointer.
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn wxr_add_entity(scene: *mut Scene) -> Entity {
//     if scene.is_null() {
//         return 0;
//     }
//
//     let scene = unsafe { &mut *scene };
//     scene.add_entity()
// }
//
// /// # Safety
// ///
// /// `scene` must be null or a valid `WXRScene` pointer.
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn wxr_get_entities_count(scene: *const Scene) -> usize {
//     if scene.is_null() {
//         return 0;
//     }
//
//     let scene = unsafe { &*scene };
//     scene.get_entities_count()
// }
//
// /// # Safety
// ///
// /// `scene` must be null or a valid `WXRScene` pointer. `out` must be null or point to enough
// /// writable memory for `capacity` entities.
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn wxr_get_entities(scene: *const Scene, out: *mut Entity, capacity: usize) {
//     if scene.is_null() || out.is_null() {
//         return;
//     }
//
//     let scene = unsafe { &*scene };
//
//     for (i, entity) in scene
//         .get_entities()
//         .iter()
//         .copied()
//         .take(capacity)
//         .enumerate()
//     {
//         unsafe {
//             out.add(i).write(entity);
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn scene_roundtrip() {
//         let scene = wxr_create_scene();
//         let entity = unsafe { wxr_add_entity(scene) };
//
//         let size = unsafe { wxr_get_entities_count(scene) };
//         let mut entities: Vec<Entity> = vec![0; size];
//         unsafe { wxr_get_entities(scene, entities.as_mut_ptr(), entities.len()) };
//
//         assert_eq!(size, 1);
//         assert_eq!(entities[0], entity);
//
//         unsafe { wxr_destroy_scene(scene) };
//     }
//
//     #[test]
//     fn null_scene_is_accepted() {
//         assert_eq!(unsafe { wxr_add_entity(std::ptr::null_mut()) }, 0);
//         assert_eq!(unsafe { wxr_get_entities_count(std::ptr::null()) }, 0);
//
//         unsafe {
//             wxr_get_entities(std::ptr::null(), std::ptr::null_mut(), 0);
//             wxr_destroy_scene(std::ptr::null_mut());
//         }
//     }
//
//     #[test]
//     fn get_entities_respects_capacity() {
//         let scene = wxr_create_scene();
//         unsafe {
//             wxr_add_entity(scene);
//             wxr_add_entity(scene);
//             wxr_add_entity(scene);
//         }
//
//         let mut entities: Vec<Entity> = vec![usize::MAX; 2];
//         unsafe { wxr_get_entities(scene, entities.as_mut_ptr(), entities.len()) };
//
//         assert_eq!(entities, &[0, 1]);
//
//         unsafe { wxr_destroy_scene(scene) };
//     }
// }
