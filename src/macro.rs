#[doc(hidden)]
pub mod private {
    pub use uuid::Uuid;
}

#[macro_export]
macro_rules! system {
    (
        id = $id:literal,
        group_symbol = $group_symbol:literal,
        entities = [
            $( [ $( $component:literal ),* $(,)? ] ),+ $(,)?
        ],
        $vis:vis fn $name:ident(
            $scene_arg:ident : &mut $scene_ty:ty,
            $entities_arg:ident : Vec<Vec<$entity_ty:ty>>,
            $groups_arg:ident : Vec<usize> $(,)?
        ) $body:block
    ) => {
        $vis fn $name(
            $scene_arg: &mut $scene_ty,
            $entities_arg: Vec<Vec<$entity_ty>>,
            $groups_arg: Vec<usize>,
        ) $body

        const _: () = {
            #[used]
            #[unsafe(export_name = $group_symbol)]
            static GROUPS: usize = $crate::system!(@count_groups $( [ $( $component ),* ] ),+);

            #[used]
            static KEEP_SELECTOR: unsafe extern "C" fn(*const $crate::scene::Scene, *const u8) -> i32 =
                selector;

            #[unsafe(export_name = concat!("wxr_select_", $id))]
            unsafe extern "C" fn selector(scene: *const $crate::scene::Scene, entity: *const u8) -> i32 {
                let scene = unsafe { &*scene };
                let entity_id = $crate::system!(@uuid_from_ptr entity);
                let selectors: &[&[&str]] = &[
                    $( &[ $( $component ),* ] ),+
                ];

                for (group, components) in selectors.iter().enumerate() {
                    if components
                        .iter()
                        .all(|component| scene.has_component(entity_id, component))
                    {
                        return group as i32;
                    }
                }

                -1
            }

            #[used]
            static KEEP_RUNNER: unsafe extern "C" fn(
                *mut $crate::scene::Scene,
                *const *const *const u8,
                *const usize,
            ) = runner;

            #[unsafe(export_name = concat!("wxr_system_", $id))]
            unsafe extern "C" fn runner(
                scene: *mut $crate::scene::Scene,
                entities: *const *const *const u8,
                groups: *const usize,
            ) {
                let groups = unsafe { std::slice::from_raw_parts(groups, GROUPS) }.to_vec();
                let raw_groups = unsafe { std::slice::from_raw_parts(entities, GROUPS) };
                let mut rust_entities: Vec<Vec<$entity_ty>> = Vec::with_capacity(GROUPS);

                for (group_index, group_entities) in raw_groups.iter().enumerate() {
                    let raw_entities =
                        unsafe { std::slice::from_raw_parts(*group_entities, groups[group_index]) };
                    let mut rust_group = Vec::with_capacity(groups[group_index]);

                    for entity in raw_entities {
                        rust_group.push($crate::system!(@uuid_from_ptr *entity));
                    }

                    rust_entities.push(rust_group);
                }

                let scene = unsafe { &mut *scene };
                $name(scene, rust_entities, groups);
            }
        };
    };

    (@count_groups $( $group:tt ),+) => {
        <[()]>::len(&[ $( $crate::system!(@unit $group) ),+ ])
    };

    (@unit $group:tt) => {
        ()
    };

    (@uuid_from_ptr $ptr:expr) => {{
        let bytes = unsafe { std::slice::from_raw_parts($ptr, 16) };
        $crate::r#macro::private::Uuid::from_slice(bytes)
            .expect("WasserXR entity pointers must point to 16 UUID bytes")
    }};
}

#[cfg(test)]
mod tests {
    use crate::scene::Scene;
    use std::{
        ffi::c_void,
        sync::{LazyLock, Mutex},
    };
    use uuid::Uuid;

    #[repr(C)]
    struct MacroCounter {
        value: i64,
    }

    static MACRO_SYSTEM_ENTITIES: LazyLock<Mutex<Vec<Vec<Uuid>>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    static MACRO_SYSTEM_GROUPS: LazyLock<Mutex<Vec<usize>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_macro_counter() -> *mut c_void {
        Box::into_raw(Box::new(MacroCounter { value: 0 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_macro_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut MacroCounter));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_create_macro_marker() -> *mut c_void {
        Box::into_raw(Box::new(MacroCounter { value: 0 })) as *mut c_void
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn wxr_destroy_macro_marker(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut MacroCounter));
        }
    }

    crate::system! {
        id = "macro_group_counter",
        group_symbol = "WXR_GROUPS_MACRO_GROUP_COUNTER",
        entities = [
            ["macro_counter"],
            ["macro_marker"],
        ],
        pub fn macro_group_counter(
            _scene: &mut Scene,
            entities: Vec<Vec<Uuid>>,
            groups: Vec<usize>,
        ) {
            *MACRO_SYSTEM_ENTITIES.lock().unwrap() = entities;
            *MACRO_SYSTEM_GROUPS.lock().unwrap() = groups;
        }
    }

    #[test]
    fn system_macro_registers_and_runs_static_system() {
        MACRO_SYSTEM_ENTITIES.lock().unwrap().clear();
        MACRO_SYSTEM_GROUPS.lock().unwrap().clear();

        let mut scene = Scene::new();
        let counter = scene.add_entity();
        scene
            .add_component(counter, "macro_counter".to_owned())
            .unwrap();

        let marker = scene.add_entity();
        scene
            .add_component(marker, "macro_marker".to_owned())
            .unwrap();

        let both = scene.add_entity();
        scene
            .add_component(both, "macro_counter".to_owned())
            .unwrap();
        scene
            .add_component(both, "macro_marker".to_owned())
            .unwrap();

        scene
            .add_system("macro_group_counter".to_owned(), 1)
            .unwrap();

        scene.tick();

        let groups = MACRO_SYSTEM_GROUPS.lock().unwrap();
        assert_eq!(*groups, vec![2, 1]);

        let entities = MACRO_SYSTEM_ENTITIES.lock().unwrap();
        assert_eq!(entities.len(), 2);
        assert!(entities[0].contains(&counter));
        assert!(entities[0].contains(&both));
        assert_eq!(entities[1], vec![marker]);
    }

    #[test]
    fn scene_has_component_reports_component_presence() {
        let mut scene = Scene::new();
        let entity = scene.add_entity();

        assert!(!scene.has_component(entity, "macro_counter"));

        scene
            .add_component(entity, "macro_counter".to_owned())
            .unwrap();

        assert!(scene.has_component(entity, "macro_counter"));
        assert!(!scene.has_component(entity, "macro_marker"));
    }
}
