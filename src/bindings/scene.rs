use std::{
    ffi::{c_char, c_void},
    ptr, slice,
};

use uuid::Uuid;

use crate::{
    bindings::{
        WXRSceneError, clear_error, result_code, set_error, set_scene_error, str_from_ptr,
        string_to_ptr,
    },
    scene::{
        Scene,
        component::methods::{Method, WXRArgument, WXRMethodResult, WXRMethodStatus},
        plugin::WXRPluginDescriptor,
    },
};

/// C callback that destroys a raw resource pointer.
pub type WXRResourceDestroyer = unsafe extern "C" fn(*mut c_void);

/// Opaque C handle for `wasserxr::scene::Scene`.
pub struct WXRScene {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 16-byte entity UUID used by the C ABI.
pub struct WXREntity {
    /// UUID bytes.
    pub bytes: [u8; 16],
}

#[repr(C)]
/// Owned byte buffer returned by WasserXR bindings.
pub struct WXRBytes {
    /// Pointer to the first byte.
    pub ptr: *mut u8,
    /// Number of initialized bytes.
    pub len: usize,
    /// Allocation capacity.
    pub cap: usize,
}

#[repr(C)]
/// Owned array of entity ids returned by WasserXR bindings.
pub struct WXREntityArray {
    /// Pointer to the first entity id.
    pub ptr: *mut WXREntity,
    /// Number of entity ids.
    pub len: usize,
}

#[repr(C)]
/// Owned array of C strings returned by WasserXR bindings.
pub struct WXRStringArray {
    /// Pointer to the first C string pointer.
    pub ptr: *mut *mut c_char,
    /// Number of strings.
    pub len: usize,
}

impl WXREntity {
    pub(crate) fn from_uuid(id: Uuid) -> Self {
        Self {
            bytes: *id.as_bytes(),
        }
    }

    pub(crate) fn into_uuid(self) -> Uuid {
        Uuid::from_bytes(self.bytes)
    }
}

impl WXRBytes {
    fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    fn from_vec(mut data: Vec<u8>) -> Self {
        let bytes = Self {
            ptr: data.as_mut_ptr(),
            len: data.len(),
            cap: data.capacity(),
        };
        std::mem::forget(data);
        bytes
    }
}

fn scene_ref<'a>(scene: *const WXRScene) -> Result<&'a Scene, WXRSceneError> {
    if scene.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &*(scene as *const Scene) })
}

fn scene_mut<'a>(scene: *mut WXRScene) -> Result<&'a mut Scene, WXRSceneError> {
    if scene.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &mut *(scene as *mut Scene) })
}

fn string_array(values: Vec<String>) -> WXRStringArray {
    let mut values: Vec<*mut c_char> = values.into_iter().map(string_to_ptr).collect();
    let array = WXRStringArray {
        ptr: values.as_mut_ptr(),
        len: values.len(),
    };
    std::mem::forget(values);
    array
}

fn entity_array(values: Vec<Uuid>) -> WXREntityArray {
    let mut values: Vec<WXREntity> = values.into_iter().map(WXREntity::from_uuid).collect();
    let array = WXREntityArray {
        ptr: values.as_mut_ptr(),
        len: values.len(),
    };
    std::mem::forget(values);
    array
}

/// Creates a new scene.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_create_scene() -> *mut WXRScene {
    clear_error();
    Box::into_raw(Box::new(Scene::new())).cast()
}

/// Destroys a scene created by `wxr_create_scene`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_destroy_scene(scene: *mut WXRScene) {
    if !scene.is_null() {
        unsafe {
            drop(Box::from_raw(scene as *mut Scene));
        }
    }
}

/// Resets a scene.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_reset(scene: *mut WXRScene) -> i32 {
    match scene_mut(scene) {
        Ok(scene) => result_code(scene.reset()),
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Serializes a scene into an owned byte buffer.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_serialize(scene: *const WXRScene) -> WXRBytes {
    match scene_ref(scene).and_then(|scene| Ok(scene.serialize()?)) {
        Ok(data) => {
            clear_error();
            WXRBytes::from_vec(data)
        }
        Err(error) => {
            set_error(error);
            WXRBytes::empty()
        }
    }
}

/// Frees a byte buffer returned by WasserXR bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_free_bytes(bytes: WXRBytes) {
    if !bytes.ptr.is_null() && bytes.cap != 0 && bytes.len <= bytes.cap {
        unsafe {
            drop(Vec::from_raw_parts(bytes.ptr, bytes.len, bytes.cap));
        }
    }
}

/// Replaces a scene with serialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_deserialize(scene: *mut WXRScene, data: *const u8, len: usize) -> i32 {
    if data.is_null() && len != 0 {
        set_error(WXRSceneError::NullPointer);
        return -1;
    }

    match scene_mut(scene) {
        Ok(scene) => {
            let data = unsafe { slice::from_raw_parts(data, len) };
            result_code(scene.deserialize(data))
        }
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Saves a scene to a filesystem path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_save(scene: *const WXRScene, path: *const c_char) -> i32 {
    match (scene_ref(scene), unsafe { str_from_ptr(path) }) {
        (Ok(scene), Ok(path)) => result_code(scene.save(path)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Loads a scene from a filesystem path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_load(scene: *mut WXRScene, path: *const c_char) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(path) }) {
        (Ok(scene), Ok(path)) => result_code(scene.load(path)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Loads a dynamic plugin by path.
///
/// # Safety
/// `scene` must be a live scene and `path` a valid NUL-terminated UTF-8 string.
/// The loaded library's descriptor graph and callbacks must satisfy the full
/// contract documented by [`Scene::load_plugin`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_load_plugin(scene: *mut WXRScene, path: *const c_char) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(path) }) {
        (Ok(scene), Ok(path)) => result_code(unsafe { scene.load_plugin(path.to_owned()) }),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Loads a statically linked plugin descriptor.
///
/// # Safety
/// `scene` must be a live scene. `descriptor` and its complete pointer graph
/// must remain immutable and readable for the process lifetime, and every
/// callback must have its declared ABI, uphold its pointer and ownership
/// contract, return valid ABI values, and never unwind across the C boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_load_static_plugin(
    scene: *mut WXRScene,
    descriptor: *const WXRPluginDescriptor,
) -> i32 {
    let (Ok(scene), Some(descriptor)) = (scene_mut(scene), unsafe { descriptor.as_ref() }) else {
        set_error(WXRSceneError::NullPointer);
        return -1;
    };
    result_code(unsafe { scene.load_static_plugin(descriptor) })
}

/// Returns the sorted manifest names of all loaded plugins.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_get_plugins(scene: *const WXRScene) -> WXRStringArray {
    match scene_ref(scene) {
        Ok(scene) => {
            clear_error();
            string_array(scene.get_plugins())
        }
        Err(error) => {
            set_error(error);
            WXRStringArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Unloads a plugin by manifest name.
///
/// # Safety
/// `scene` must be a live scene and `name` a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_unload_plugin(scene: *mut WXRScene, name: *const c_char) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(name) }) {
        (Ok(scene), Ok(name)) => result_code(scene.unload_plugin(name)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Adds a new entity and returns its 16-byte id.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_add_entity(scene: *mut WXRScene) -> WXREntity {
    match scene_mut(scene) {
        Ok(scene) => {
            clear_error();
            WXREntity::from_uuid(scene.add_entity())
        }
        Err(error) => {
            set_error(error);
            WXREntity { bytes: [0; 16] }
        }
    }
}

/// Returns all entity ids in sorted order.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_get_entities(scene: *const WXRScene) -> WXREntityArray {
    match scene_ref(scene) {
        Ok(scene) => {
            clear_error();
            entity_array(scene.get_entities())
        }
        Err(error) => {
            set_error(error);
            WXREntityArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Frees an entity array returned by WasserXR bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_free_entities(array: WXREntityArray) {
    if !array.ptr.is_null() {
        unsafe {
            drop(Vec::from_raw_parts(array.ptr, array.len, array.len));
        }
    }
}

/// Frees a string array returned by WasserXR bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_free_strings(array: WXRStringArray) {
    if array.ptr.is_null() {
        return;
    }

    let values = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
    for value in values {
        unsafe {
            crate::bindings::wxr_free_string(value);
        }
    }
}

/// Removes an entity and all components attached to it.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_remove_entity(scene: *mut WXRScene, entity: WXREntity) -> i32 {
    match scene_mut(scene) {
        Ok(scene) => result_code(scene.remove_entity(entity.into_uuid())),
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Returns an entity display name as an owned C string.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_get_entity_name(scene: *const WXRScene, entity: WXREntity) -> *mut c_char {
    match scene_ref(scene)
        .and_then(|scene| Ok(scene.get_entity_name(entity.into_uuid())?.to_owned()))
    {
        Ok(name) => {
            clear_error();
            string_to_ptr(name)
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Sets an entity display name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_set_entity_name(
    scene: *mut WXRScene,
    entity: WXREntity,
    name: *const c_char,
) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(name) }) {
        (Ok(scene), Ok(name)) => {
            result_code(scene.set_entity_name(entity.into_uuid(), name.to_owned()))
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Adds a system by id and priority.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_add_system(
    scene: *mut WXRScene,
    id: *const c_char,
    priority: usize,
) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => result_code(scene.add_system(id.to_owned(), priority)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Removes a system by id.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_remove_system(scene: *mut WXRScene, id: *const c_char) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => result_code(scene.remove_system(id)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Returns all system ids in sorted order.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_get_systems(scene: *const WXRScene) -> WXRStringArray {
    match scene_ref(scene) {
        Ok(scene) => {
            clear_error();
            string_array(scene.get_systems())
        }
        Err(error) => {
            set_error(error);
            WXRStringArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Returns a system priority, or zero on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_system_priority(
    scene: *const WXRScene,
    id: *const c_char,
) -> usize {
    match (scene_ref(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => match scene.get_system_priority(id) {
            Ok(priority) => {
                clear_error();
                priority
            }
            Err(error) => {
                set_scene_error(error);
                0
            }
        },
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            0
        }
    }
}

/// Returns the plugin id that provided a system.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_system_plugin_id(
    scene: *const WXRScene,
    id: *const c_char,
) -> *mut c_char {
    match (scene_ref(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => match scene.get_system_plugin_id(id) {
            Ok(plugin) => {
                clear_error();
                string_to_ptr(plugin.to_owned())
            }
            Err(error) => {
                set_scene_error(error);
                ptr::null_mut()
            }
        },
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Adds a component to an entity by component id.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_add_component(
    scene: *mut WXRScene,
    entity: WXREntity,
    id: *const c_char,
) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => result_code(scene.add_component(entity.into_uuid(), id.to_owned())),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Removes a component from an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_remove_component(
    scene: *mut WXRScene,
    entity: WXREntity,
    id: *const c_char,
) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(id) }) {
        (Ok(scene), Ok(id)) => result_code(scene.remove_component(entity.into_uuid(), id)),
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Returns component ids attached to an entity.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_get_entity_components(
    scene: *const WXRScene,
    entity: WXREntity,
) -> WXRStringArray {
    match scene_ref(scene).and_then(|scene| Ok(scene.get_entity_components(entity.into_uuid())?)) {
        Ok(components) => {
            clear_error();
            string_array(components)
        }
        Err(error) => {
            set_error(error);
            WXRStringArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Returns the first entity with the component id, or a zero UUID if none exists.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_entity_with_component(
    scene: *const WXRScene,
    component_id: *const c_char,
) -> WXREntity {
    match (scene_ref(scene), unsafe { str_from_ptr(component_id) }) {
        (Ok(scene), Ok(component_id)) => {
            clear_error();
            scene
                .get_entity_with_component(component_id)
                .map(WXREntity::from_uuid)
                .unwrap_or(WXREntity { bytes: [0; 16] })
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            WXREntity { bytes: [0; 16] }
        }
    }
}

/// Returns 1 when an entity has the component id, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_has_component(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
) -> i32 {
    match (scene_ref(scene), unsafe { str_from_ptr(component_id) }) {
        (Ok(scene), Ok(component_id)) => {
            clear_error();
            i32::from(scene.has_component(entity.into_uuid(), component_id))
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            0
        }
    }
}

/// Hot-reloads all dynamic plugins while preserving serializable scene state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_reload(scene: *mut WXRScene) -> i32 {
    match scene_mut(scene) {
        Ok(scene) => result_code(unsafe { scene.reload() }),
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Runs all systems once. Returns 1 to continue and 0 to stop.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_tick(scene: *mut WXRScene) -> i32 {
    match scene_mut(scene) {
        Ok(scene) => {
            clear_error();
            i32::from(scene.tick())
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

/// Ensures an asset is loaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_ensure_asset_loaded(
    scene: *mut WXRScene,
    asset_type: *const c_char,
    data_string: *const c_char,
) -> i32 {
    match (
        scene_mut(scene),
        unsafe { str_from_ptr(asset_type) },
        unsafe { str_from_ptr(data_string) },
    ) {
        (Ok(scene), Ok(asset_type), Ok(data_string)) => {
            result_code(scene.ensure_asset_loaded(asset_type, data_string))
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Returns loaded data strings for an asset type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_loaded_asset_data_strings(
    scene: *const WXRScene,
    asset_type: *const c_char,
) -> WXRStringArray {
    match (scene_ref(scene), unsafe { str_from_ptr(asset_type) }) {
        (Ok(scene), Ok(asset_type)) => {
            clear_error();
            string_array(scene.get_loaded_asset_data_strings(asset_type))
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            WXRStringArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Returns a raw pointer to one component field.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_query(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
) -> *mut c_void {
    match (
        scene_ref(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id)) => {
            let fields = [field_id];
            match unsafe { scene.get_field_ptrs(entity.into_uuid(), component_id, &fields) } {
                Ok(mut fields) => {
                    clear_error();
                    fields.pop().unwrap_or(ptr::null_mut())
                }
                Err(error) => {
                    set_scene_error(error);
                    ptr::null_mut()
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Loads an asset if needed and returns a raw pointer to one asset field.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_asset_query(
    scene: *mut WXRScene,
    asset_type: *const c_char,
    data_string: *const c_char,
    field_id: *const c_char,
) -> *mut c_void {
    match (
        scene_mut(scene),
        unsafe { str_from_ptr(asset_type) },
        unsafe { str_from_ptr(data_string) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(asset_type), Ok(data_string), Ok(field_id)) => {
            if let Err(error) = scene.ensure_asset_loaded(asset_type, data_string) {
                set_scene_error(error);
                return ptr::null_mut();
            }
            let fields = [field_id];
            match unsafe { scene.get_asset_field_ptrs(asset_type, data_string, &fields) } {
                Ok(mut fields) => {
                    clear_error();
                    fields.pop().unwrap_or(ptr::null_mut())
                }
                Err(error) => {
                    set_scene_error(error);
                    ptr::null_mut()
                }
            }
        }
        (Err(error), _, _, _)
        | (_, Err(error), _, _)
        | (_, _, Err(error), _)
        | (_, _, _, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Returns component field ids for a component on an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_component_fields(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
) -> WXRStringArray {
    match (scene_ref(scene), unsafe { str_from_ptr(component_id) }) {
        (Ok(scene), Ok(component_id)) => {
            match scene.get_component_fields(entity.into_uuid(), component_id) {
                Ok(fields) => {
                    clear_error();
                    string_array(fields)
                }
                Err(error) => {
                    set_scene_error(error);
                    WXRStringArray {
                        ptr: ptr::null_mut(),
                        len: 0,
                    }
                }
            }
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            WXRStringArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Returns the plugin id that provided a component on an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_entity_component_plugin_id(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
) -> *mut c_char {
    match (scene_ref(scene), unsafe { str_from_ptr(component_id) }) {
        (Ok(scene), Ok(component_id)) => {
            match scene.get_entity_component_plugin_id(entity.into_uuid(), component_id) {
                Ok(plugin) => {
                    clear_error();
                    string_to_ptr(plugin.to_owned())
                }
                Err(error) => {
                    set_scene_error(error);
                    ptr::null_mut()
                }
            }
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Returns a component field type hint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_component_field_type(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
) -> i32 {
    match (
        scene_ref(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id)) => {
            match scene.get_component_field_type(entity.into_uuid(), component_id, field_id) {
                Ok(field_type) => {
                    clear_error();
                    field_type as i32
                }
                Err(error) => {
                    set_scene_error(error);
                    -1
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Returns 1 when a component field is mutable, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_is_component_field_mutable(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
) -> i32 {
    match (
        scene_ref(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id)) => {
            match scene.is_component_field_mutable(entity.into_uuid(), component_id, field_id) {
                Ok(value) => {
                    clear_error();
                    i32::from(value)
                }
                Err(error) => {
                    set_scene_error(error);
                    0
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            0
        }
    }
}

/// Returns 1 when a component field can be parsed from a string, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_is_component_field_string_parsable(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
) -> i32 {
    match (
        scene_ref(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id)) => {
            match scene.is_component_field_string_parsable(
                entity.into_uuid(),
                component_id,
                field_id,
            ) {
                Ok(value) => {
                    clear_error();
                    i32::from(value)
                }
                Err(error) => {
                    set_scene_error(error);
                    0
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            0
        }
    }
}

/// Renders a component field as an owned C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_render_field(
    scene: *const WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
) -> *mut c_char {
    match (
        scene_ref(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id)) => {
            match scene.render_field(entity.into_uuid(), component_id, field_id) {
                Ok(value) => {
                    clear_error();
                    string_to_ptr(value)
                }
                Err(error) => {
                    set_scene_error(error);
                    ptr::null_mut()
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Parses a string into a component field.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_parse_field(
    scene: *mut WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    field_id: *const c_char,
    input: *const c_char,
) -> i32 {
    match (
        scene_mut(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(field_id) },
        unsafe { str_from_ptr(input) },
    ) {
        (Ok(scene), Ok(component_id), Ok(field_id), Ok(input)) => {
            result_code(scene.parse_field(entity.into_uuid(), component_id, field_id, input))
        }
        (Err(error), _, _, _)
        | (_, Err(error), _, _)
        | (_, _, Err(error), _)
        | (_, _, _, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Adds a raw C resource to the scene.
#[deprecated(note = "resources are deprecated and will be removed in a future release")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_add_resource(
    scene: *mut WXRScene,
    name: *const c_char,
    data: *mut c_void,
    destroyer: WXRResourceDestroyer,
) -> i32 {
    if data.is_null() {
        set_error(WXRSceneError::NullPointer);
        return -1;
    }

    match scene_mut(scene) {
        Ok(scene) => {
            crate::warn!(scene, "Resource bindings are deprecated");
            match unsafe { str_from_ptr(name) } {
                Ok(name) => result_code(scene.add_raw_resource(name.to_owned(), data, destroyer)),
                Err(error) => {
                    set_error(error);
                    -1
                }
            }
        }
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Returns a raw C resource pointer by name.
#[deprecated(note = "resources are deprecated and will be removed in a future release")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_resource(
    scene: *mut WXRScene,
    name: *const c_char,
) -> *mut c_void {
    match scene_mut(scene) {
        Ok(scene) => {
            crate::warn!(scene, "Resource bindings are deprecated");
            match unsafe { str_from_ptr(name) } {
                Ok(name) => match scene.get_raw_resource(name) {
                    Ok(data) => {
                        clear_error();
                        data
                    }
                    Err(error) => {
                        set_scene_error(error);
                        ptr::null_mut()
                    }
                },
                Err(error) => {
                    set_error(error);
                    ptr::null_mut()
                }
            }
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Requests that the next `wxr_tick` return 0.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_should_exit(scene: *mut WXRScene) -> i32 {
    match scene_mut(scene) {
        Ok(scene) => {
            scene.should_exit();
            clear_error();
            0
        }
        Err(error) => {
            set_error(error);
            -1
        }
    }
}

/// Opaque heap-allocated handle for a component method invocation.
///
/// The target is resolved again when the handle is called, so removing the
/// component or unloading its plugin turns the call into an error instead of
/// retaining a dangling component pointer.
pub struct WXRMethod {
    scene: *mut WXRScene,
    entity: Uuid,
    component_id: String,
    method_name: String,
    arguments: Vec<WXRArgument>,
}

/// Resolves a component method handle, or null on failure.
///
/// On failure the error is recorded for `wxr_error`. The returned handle must
/// be consumed by `wxr_method_call` or abandoned with `wxr_method_destroy`.
/// The scene must remain alive until either operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_get_method(
    scene: *mut WXRScene,
    entity: WXREntity,
    component_id: *const c_char,
    method_name: *const c_char,
) -> *mut WXRMethod {
    match (
        scene_mut(scene),
        unsafe { str_from_ptr(component_id) },
        unsafe { str_from_ptr(method_name) },
    ) {
        (Ok(scene_ref), Ok(component_id), Ok(method_name)) => {
            match scene_ref.resolve_method(entity.into_uuid(), component_id, method_name) {
                Ok(_) => {
                    clear_error();
                    Box::into_raw(Box::new(WXRMethod {
                        scene,
                        entity: entity.into_uuid(),
                        component_id: component_id.to_owned(),
                        method_name: method_name.to_owned(),
                        arguments: Vec::new(),
                    }))
                }
                Err(error) => {
                    set_scene_error(error);
                    ptr::null_mut()
                }
            }
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Appends a borrowed named argument to a method handle.
///
/// The name and data stay owned by the caller and must remain valid until the
/// consuming `wxr_method_call`.
///
/// # Safety
/// `method` must be a live method handle, `name` must be a valid
/// NUL-terminated UTF-8 string, and non-null `data` must match `field_type`.
/// Null `data` is allowed and is checked against the manifest declaration when
/// the method is called.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_method_argument(
    method: *mut WXRMethod,
    name: *const c_char,
    field_type: u32,
    data: *mut c_void,
) {
    if let Some(method) = unsafe { method.as_mut() } {
        method.arguments.push(WXRArgument {
            name,
            field_type,
            data,
        });
    }
}

/// Invokes a method handle synchronously and frees it.
///
/// The handle is consumed and freed on every outcome and is invalid afterward.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_method_call(method: *mut WXRMethod) -> WXRMethodResult {
    if method.is_null() {
        return WXRMethodResult {
            status: WXRMethodStatus::ActionError,
            action_error: -1,
            value: ptr::null_mut(),
        };
    }

    let method = unsafe { Box::from_raw(method) };
    let result = match scene_mut(method.scene) {
        Ok(scene) => {
            match scene.resolve_method(method.entity, &method.component_id, &method.method_name) {
                Ok((definition, component)) => {
                    let mut resolved = Method::new(method.scene, definition, component);
                    for argument in method.arguments {
                        resolved.push_argument(argument.name, argument.field_type, argument.data);
                    }
                    resolved.call()
                }
                Err(error) => {
                    set_scene_error(error);
                    WXRMethodResult {
                        status: WXRMethodStatus::ActionError,
                        action_error: -1,
                        value: ptr::null_mut(),
                    }
                }
            }
        }
        Err(error) => {
            set_error(error);
            WXRMethodResult {
                status: WXRMethodStatus::ActionError,
                action_error: -1,
                value: ptr::null_mut(),
            }
        }
    };
    result
}

/// Frees a method handle without calling it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_method_destroy(method: *mut WXRMethod) {
    if !method.is_null() {
        unsafe {
            drop(Box::from_raw(method));
        }
    }
}
#[cfg(test)]
mod tests {
    #![allow(deprecated)]

    use super::*;
    use crate::{
        bindings::wxr_error,
        scene::{
            component::{
                FieldType, SerializedBytes, WXRComponentDescriptor, WXRComponentFieldDescriptor,
            },
            plugin::{Version, WXRPluginDescriptor},
        },
    };
    use std::{
        ffi::{CStr, CString},
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[repr(C)]
    struct FfiCounter {
        value: i64,
    }

    unsafe extern "C" fn ffi_counter_getter(data: *mut c_void) -> *mut c_void {
        unsafe { &mut (*(data as *mut FfiCounter)).value as *mut i64 as *mut c_void }
    }

    unsafe extern "C" fn ffi_counter_serializer(data: *const c_void) -> SerializedBytes {
        unsafe {
            SerializedBytes::from_vec((*(data as *const FfiCounter)).value.to_le_bytes().to_vec())
        }
    }

    unsafe extern "C" fn ffi_counter_deserializer(data: *mut c_void, value: SerializedBytes) {
        unsafe {
            let bytes = value.into_vec();
            if let Ok(bytes) = <[u8; 8]>::try_from(bytes.as_slice()) {
                (*(data as *mut FfiCounter)).value = i64::from_le_bytes(bytes);
            }
        }
    }

    unsafe extern "C" fn create_ffi_counter(_scene: *mut Scene) -> *mut c_void {
        Box::into_raw(Box::new(FfiCounter { value: 5 })) as *mut c_void
    }

    unsafe extern "C" fn destroy_ffi_counter(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut FfiCounter));
        }
    }

    static FFI_COUNTER_FIELDS: [WXRComponentFieldDescriptor; 1] = [WXRComponentFieldDescriptor {
        name: c"value".as_ptr(),
        field_type: FieldType::I64 as u32,
        getter: Some(ffi_counter_getter),
        mutable: 1,
        serializer: Some(ffi_counter_serializer),
        deserializer: Some(ffi_counter_deserializer),
    }];
    static FFI_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
        name: c"ffi_counter".as_ptr(),
        creator: Some(create_ffi_counter),
        destroyer: Some(destroy_ffi_counter),
        fields: FFI_COUNTER_FIELDS.as_ptr(),
        field_count: FFI_COUNTER_FIELDS.len(),
        methods: ptr::null(),
        method_count: 0,
    }];
    static FFI_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
        version: Version::CURRENT,
        name: c"bindings-tests".as_ptr(),
        components: FFI_COMPONENTS.as_ptr(),
        component_count: FFI_COMPONENTS.len(),
        assets: ptr::null(),
        asset_count: 0,
        systems: ptr::null(),
        system_count: 0,
    };

    static RESOURCE_DROPS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn drop_usize(data: *mut c_void) {
        unsafe {
            drop(Box::from_raw(data as *mut usize));
        }
        RESOURCE_DROPS.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn scene_entity_name_round_trip() {
        let scene = wxr_create_scene();
        let entity = wxr_add_entity(scene);
        let name = CString::new("Player").unwrap();

        assert_eq!(
            unsafe { wxr_set_entity_name(scene, entity, name.as_ptr()) },
            0
        );

        let actual = wxr_get_entity_name(scene, entity);
        assert_eq!(
            unsafe { CStr::from_ptr(actual) }.to_str().unwrap(),
            "Player"
        );
        unsafe {
            crate::bindings::wxr_free_string(actual);
        }

        let entities = wxr_get_entities(scene);
        assert_eq!(entities.len, 1);
        unsafe {
            wxr_free_entities(entities);
            wxr_destroy_scene(scene);
        }
    }

    #[test]
    fn scene_error_tracks_failure() {
        let scene = wxr_create_scene();
        let missing = WXREntity { bytes: [7; 16] };

        assert_eq!(wxr_remove_entity(scene, missing), -1);
        assert_eq!(wxr_error(), WXRSceneError::EntityNotFound);

        unsafe {
            wxr_destroy_scene(scene);
        }
    }

    #[test]
    fn scene_query_returns_field_pointer() {
        let scene = wxr_create_scene();
        unsafe {
            (&mut *(scene as *mut Scene))
                .load_static_plugin(&FFI_PLUGIN)
                .unwrap();
        }
        let entity = wxr_add_entity(scene);
        let component = CString::new("ffi_counter").unwrap();
        let field = CString::new("value").unwrap();

        assert_eq!(
            unsafe { wxr_add_component(scene, entity, component.as_ptr()) },
            0
        );
        let value = unsafe { wxr_query(scene, entity, component.as_ptr(), field.as_ptr()) };
        assert!(!value.is_null());
        assert_eq!(unsafe { *(value as *const i64) }, 5);

        unsafe {
            *(value as *mut i64) = 9;
        }
        let rendered =
            unsafe { wxr_render_field(scene, entity, component.as_ptr(), field.as_ptr()) };
        assert_eq!(unsafe { CStr::from_ptr(rendered) }.to_str().unwrap(), "9");
        unsafe {
            crate::bindings::wxr_free_string(rendered);
            wxr_destroy_scene(scene);
        }
    }

    #[test]
    fn raw_resource_round_trip_and_drop() {
        RESOURCE_DROPS.store(0, Ordering::SeqCst);
        let scene = wxr_create_scene();
        let name = CString::new("answer").unwrap();
        let data = Box::into_raw(Box::new(42usize)) as *mut c_void;

        assert_eq!(
            unsafe { wxr_add_resource(scene, name.as_ptr(), data, drop_usize) },
            0
        );
        let value = unsafe { wxr_get_resource(scene, name.as_ptr()) };
        assert_eq!(unsafe { *(value as *const usize) }, 42);

        let logs = unsafe { &*(scene as *const Scene) }
            .iter_logs()
            .collect::<Vec<_>>();
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().all(|entry| {
            entry.get_level() == crate::scene::logging::LogLevel::WARN
                && entry.get_message() == "Resource bindings are deprecated"
        }));

        unsafe {
            wxr_destroy_scene(scene);
        }
        assert_eq!(RESOURCE_DROPS.load(Ordering::SeqCst), 1);
    }
}
