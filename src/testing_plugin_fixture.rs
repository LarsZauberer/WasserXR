//! Shared testing plugin fixture.
//!
//! Instead of hand-writing `no_mangle` C stubs, this file declares the reusable
//! components, systems, and asset types with the real WasserXR macros. They
//! become the statically-linked "static plugin" symbols that `Scene` discovers
//! via `dlsym`, so every test can add/query/serialize them by name.
//!
//! It is compiled into each test binary two ways: as a `#[cfg(test)]` module of
//! the library (unit tests) and via `#[path = "../src/testing_plugin_fixture.rs"]`
//! from each integration test. It therefore uses `wasserxr::` paths only and
//! never touches crate-private items. It never ships in the production staticlib.
#![allow(dead_code)]

use std::sync::{
    LazyLock, Mutex,
    atomic::{AtomicBool, AtomicUsize},
};

use rstest::fixture;
use serde::{Deserialize, Serialize};
use wasserxr::scene::{
    Scene,
    component::{FieldType, Schema, SerializedBytes},
};
use wasserxr::{
    Uuid, asset_type, asset_type_creator, attacher, component, component_creator, detacher, system,
};

// ---------------------------------------------------------------------------
// Raw field hooks — for the field/schema metadata-layer unit tests that build
// `Field`/`Schema` values directly, below the macro layer.
// ---------------------------------------------------------------------------

/// A getter returning null; used only where a `Getter` value is compared by
/// identity, never dereferenced.
pub unsafe extern "C" fn null_getter(_data: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

/// A serializer returning fixed bytes; exercises serializer wiring.
pub unsafe extern "C" fn sample_serializer(_data: *const std::ffi::c_void) -> SerializedBytes {
    SerializedBytes::from_vec(vec![1, 2, 3])
}

/// A deserializer that ignores its input; exercises deserializer wiring.
pub unsafe extern "C" fn noop_deserializer(_data: *mut std::ffi::c_void, _value: SerializedBytes) {}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Canonical single-field component: a mutable, serializable `i64` `value`,
/// starting at 1.
#[component]
#[derive(Default)]
pub struct Counter {
    #[mutable]
    value: i64,
}

#[component_creator(Counter)]
fn create_counter(_scene: &mut Scene) -> Option<Counter> {
    Some(Counter { value: 1 })
}

/// Two `i64` fields: `a` mutable, `b` read-only. Covers multi-field queries and
/// per-field mutability / string-parsability.
#[component]
#[derive(Default)]
pub struct Pair {
    #[mutable]
    a: i64,
    #[getter]
    b: i64,
}

#[component_creator(Pair)]
fn create_pair(_scene: &mut Scene) -> Option<Pair> {
    Some(Pair { a: 1, b: 2 })
}

/// A non-string-parsable owned value, rendered as a `Blob` pointer. `readonly`
/// is immutable; `writable` is mutable (for in-place mutation of a non-`Copy`
/// field).
#[derive(Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct OwnedValue {
    pub value: String,
}

#[component]
#[derive(Default)]
pub struct Owner {
    #[getter]
    readonly: OwnedValue,
    #[mutable]
    writable: OwnedValue,
}

#[component_creator(Owner)]
fn create_owner(_scene: &mut Scene) -> Option<Owner> {
    Some(Owner::default())
}

/// Eight `i64` fields, to exercise the maximum query-tuple arity.
#[component]
#[derive(Default)]
pub struct Wide {
    #[getter]
    a: i64,
    #[getter]
    b: i64,
    #[getter]
    c: i64,
    #[getter]
    d: i64,
    #[getter]
    e: i64,
    #[getter]
    f: i64,
    #[getter]
    g: i64,
    #[getter]
    h: i64,
}

#[component_creator(Wide)]
fn create_wide(_scene: &mut Scene) -> Option<Wide> {
    Some(Wide {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: 5,
        f: 6,
        g: 7,
        h: 8,
    })
}

/// Exercises the spread of field attributes the `#[component]` macro supports:
/// getter-only (immutable), mutable-only, explicit serializer/deserializer, a
/// getter+mutable boolean, and an excluded `#[none]` field.
#[component]
#[derive(Default)]
pub struct Featured {
    #[getter]
    reads_only: i32,
    #[mutable]
    reads_writes: i32,
    #[getter]
    #[mutable]
    #[serializer]
    #[deserializer]
    text: String,
    #[getter]
    #[mutable]
    enabled: bool,
    #[none]
    hidden: i32,
}

#[component_creator(Featured)]
fn create_featured(_scene: &mut Scene) -> Option<Featured> {
    Some(Featured::default())
}

/// A `usize` field wired to custom getter/serializer/deserializer functions.
/// The serializer always writes `99` and the deserializer adds `1`, so a
/// round-trip is observable.
#[component]
#[derive(Default)]
pub struct CustomHooks {
    #[getter(custom_value_getter)]
    #[mutable]
    #[serializer(custom_value_serializer)]
    #[deserializer(custom_value_deserializer)]
    value: usize,
}

#[component_creator(CustomHooks)]
fn create_custom_hooks(_scene: &mut Scene) -> Option<CustomHooks> {
    Some(CustomHooks::default())
}

unsafe extern "C" fn custom_value_getter(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    unsafe { &mut (*(ptr as *mut CustomHooks)).value as *mut usize as *mut std::ffi::c_void }
}

unsafe extern "C" fn custom_value_serializer(_ptr: *const std::ffi::c_void) -> SerializedBytes {
    SerializedBytes::from_vec(99usize.to_le_bytes().to_vec())
}

unsafe extern "C" fn custom_value_deserializer(ptr: *mut std::ffi::c_void, data: SerializedBytes) {
    let bytes = unsafe { data.into_vec() };
    if let Ok(bytes) = <[u8; std::mem::size_of::<usize>()]>::try_from(bytes.as_slice()) {
        unsafe {
            (*(ptr as *mut CustomHooks)).value = usize::from_le_bytes(bytes) + 1;
        }
    }
}

/// A `[f32; 3]` backing store exposed through two virtual fields `x` (mutable)
/// and `y` (read-only).
#[component]
#[virtual_field(x: f32, getter = position_x, mutable)]
#[virtual_field(y: f32, getter = position_y)]
#[derive(Default)]
pub struct Position {
    #[mutable]
    position: [f32; 3],
}

#[component_creator(Position)]
fn create_position(_scene: &mut Scene) -> Option<Position> {
    Some(Position::default())
}

unsafe extern "C" fn position_x(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    unsafe { &mut (*(ptr as *mut Position)).position[0] as *mut f32 as *mut std::ffi::c_void }
}

unsafe extern "C" fn position_y(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    unsafe { &mut (*(ptr as *mut Position)).position[1] as *mut f32 as *mut std::ffi::c_void }
}

/// A `no_schema` component whose schema is supplied by a hand-written
/// `wxr_schema_ManualSchema` function.
#[component(no_schema)]
#[derive(Default)]
pub struct ManualSchema {
    #[none]
    value: i32,
}

#[component_creator(ManualSchema)]
fn create_manual_schema(_scene: &mut Scene) -> Option<ManualSchema> {
    Some(ManualSchema::default())
}

unsafe extern "C" fn manual_schema_value_getter(
    ptr: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    unsafe { &mut (*(ptr as *mut ManualSchema)).value as *mut i32 as *mut std::ffi::c_void }
}

#[unsafe(export_name = "wxr_schema_ManualSchema")]
#[allow(non_snake_case)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn wxr_schema_ManualSchema(schema: *mut Schema) {
    unsafe {
        (*schema).add_field(
            "value".to_owned(),
            FieldType::I32,
            Some(manual_schema_value_getter),
            true,
            None,
            None,
        );
    }
}

/// A `no_schema` component with no schema function at all: every field lookup
/// must fail.
#[component(no_schema)]
#[derive(Default)]
pub struct NoSchema {
    #[none]
    value: i32,
}

#[component_creator(NoSchema)]
fn create_no_schema(_scene: &mut Scene) -> Option<NoSchema> {
    Some(NoSchema::default())
}

/// A component whose creator always fails.
#[component]
#[derive(Default)]
pub struct FailingCreator {
    value: i32,
}

#[component_creator(FailingCreator)]
fn create_failing(_scene: &mut Scene) -> Option<FailingCreator> {
    None
}

/// A component with a nested, serde-serialized field, to exercise round-tripping
/// of compound data through reload.
#[derive(Default, Deserialize, Serialize)]
pub struct Nested {
    pub numbers: Vec<usize>,
    pub vector: [f32; 3],
}

#[component]
#[derive(Default)]
pub struct NestedComponent {
    #[mutable]
    data: Nested,
}

#[component_creator(NestedComponent)]
fn create_nested(_scene: &mut Scene) -> Option<NestedComponent> {
    Some(NestedComponent {
        data: Nested {
            numbers: vec![],
            vector: [1.0; 3],
        },
    })
}

/// A marker component that records its destruction, for asserting plugin-unload
/// teardown order.
#[component(no_schema)]
#[derive(Default)]
pub struct OrderComponent {
    #[none]
    marker: u8,
}

impl Drop for OrderComponent {
    fn drop(&mut self) {
        UNLOAD_ORDER.lock().unwrap().push("component");
    }
}

#[component_creator(OrderComponent)]
fn create_order_component(_scene: &mut Scene) -> Option<OrderComponent> {
    Some(OrderComponent::default())
}

// ---------------------------------------------------------------------------
// Systems + their observable state
// ---------------------------------------------------------------------------

/// Serializes tests that mutate the shared system state below (counters, tick
/// order, load path). Cargo runs test functions in parallel threads within one
/// binary, so any test touching these globals must hold this lock.
pub static SCENE_LOCK: Mutex<()> = Mutex::new(());

// Lib-binary statics, all reset + guarded together by the `scene_state` fixture.
pub static ATTACH_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static DETACH_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static DEFERRED_REMOVE_TARGET_PRESENT: AtomicBool = AtomicBool::new(false);
pub static DEFERRED_UNLOAD_PLUGIN_PRESENT: AtomicBool = AtomicBool::new(false);
pub static LOAD_PATH: LazyLock<Mutex<Option<std::path::PathBuf>>> =
    LazyLock::new(|| Mutex::new(None));
pub static TICK_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
pub static UNLOAD_ORDER: LazyLock<Mutex<Vec<&'static str>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

// Integration-binary statics. Each is written by exactly one test in
// `tests/macro.rs`, so no lock is needed. If a second test ever uses
// `group_counter`/`empty_system`/`lifecycle`, guard these like the ones above.
pub static GROUP_ENTITIES: LazyLock<Mutex<Vec<Vec<Uuid>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
pub static EMPTY_ENTITIES: LazyLock<Mutex<Option<Vec<Vec<Uuid>>>>> =
    LazyLock::new(|| Mutex::new(None));
pub static LIFECYCLE_ATTACH_ENTITY: LazyLock<Mutex<Option<Uuid>>> =
    LazyLock::new(|| Mutex::new(None));
pub static LIFECYCLE_DETACH_ENTITY: LazyLock<Mutex<Option<Uuid>>> =
    LazyLock::new(|| Mutex::new(None));

/// No-op system, used wherever a test just needs a registered system.
#[system]
pub fn cleanup(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {}

/// System whose attach/detach hooks bump [`ATTACH_COUNT`] / [`DETACH_COUNT`].
/// Used both to observe add/remove and to prove a reload detaches then
/// re-attaches systems.
#[system]
pub fn attach_counter(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {}

#[attacher(attach_counter)]
pub fn attach_attach_counter(_scene: &mut Scene) {
    ATTACH_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

#[detacher(attach_counter)]
pub fn detach_attach_counter(_scene: &mut Scene) {
    DETACH_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Low-priority system that records its run order into [`TICK_ORDER`].
#[system]
pub fn low_priority(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    TICK_ORDER.lock().unwrap().push("low");
}

/// High-priority system that records its run order into [`TICK_ORDER`].
#[system]
pub fn high_priority(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    TICK_ORDER.lock().unwrap().push("high");
}

/// System that reloads the scene from within a tick, then records ordering.
#[system]
pub fn reload_request(scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    scene.reload().unwrap();
    TICK_ORDER.lock().unwrap().push("reload-request");
}

/// System that loads a scene (from [`LOAD_PATH`]) from within a tick.
#[system]
pub fn load_request(scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    let path = LOAD_PATH.lock().unwrap().clone().unwrap();
    scene.load(path).unwrap();
}

/// Marker system used to prove a loaded scene installed its systems.
#[system]
pub fn loaded_marker(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {}

/// System that removes [`remove_target`] mid-tick, records whether the removal
/// was deferred until the tick finished, and logs its run into [`TICK_ORDER`].
#[system]
pub fn remove_request(scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    scene.remove_system("remove_target").unwrap();
    DEFERRED_REMOVE_TARGET_PRESENT.store(
        scene.get_systems().contains(&"remove_target".to_owned()),
        std::sync::atomic::Ordering::SeqCst,
    );
    TICK_ORDER.lock().unwrap().push("remove-request");
}

/// The system targeted for deferred removal; logs its run into [`TICK_ORDER`].
#[system]
pub fn remove_target(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    TICK_ORDER.lock().unwrap().push("remove-target");
}

/// System that unloads the `dynamic_deferred` plugin mid-tick and records whether
/// the unload was deferred until the tick finished.
#[system]
pub fn unload_request(scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {
    scene.unload_plugin("dynamic_deferred").unwrap();
    DEFERRED_UNLOAD_PLUGIN_PRESENT.store(
        scene.get_plugins().contains(&"dynamic_deferred".to_owned()),
        std::sync::atomic::Ordering::SeqCst,
    );
}

/// System filtered to two entity groups, recording the selected entities.
#[system(entities = [["Counter"], ["Owner"]])]
pub fn group_counter(_scene: &mut Scene, entities: Vec<Vec<Uuid>>) {
    *GROUP_ENTITIES.lock().unwrap() = entities;
}

/// System with the default (empty) entity filter, recording what it received.
#[system]
pub fn empty_system(_scene: &mut Scene, entities: Vec<Vec<Uuid>>) {
    *EMPTY_ENTITIES.lock().unwrap() = Some(entities);
}

/// System with lifecycle hooks that add a named entity on attach and detach.
#[system]
pub fn lifecycle(_scene: &mut Scene, _entities: Vec<Vec<Uuid>>) {}

#[attacher(lifecycle)]
pub fn attach_lifecycle(scene: &mut Scene) {
    let entity = scene.add_entity();
    scene
        .set_entity_name(entity, "attached".to_owned())
        .unwrap();
    *LIFECYCLE_ATTACH_ENTITY.lock().unwrap() = Some(entity);
}

#[detacher(lifecycle)]
pub fn detach_lifecycle(scene: &mut Scene) {
    let entity = scene.add_entity();
    scene
        .set_entity_name(entity, "detached".to_owned())
        .unwrap();
    *LIFECYCLE_DETACH_ENTITY.lock().unwrap() = Some(entity);
}

// ---------------------------------------------------------------------------
// Asset types
// ---------------------------------------------------------------------------

pub static ASSET_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
pub static ASSET_CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// An asset built from a file's contents. Registers a mix of field types plus a
/// `#[none]` field, and counts creations so caching can be observed.
#[asset_type]
pub struct FileAsset {
    content: String,
    bytes: usize,
    position: [f64; 2],
    available: bool,

    #[none]
    hidden: String,
}

#[asset_type_creator(FileAsset)]
fn create_file_asset(_scene: &mut Scene, path: &str) -> Option<FileAsset> {
    ASSET_CREATE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let content = std::fs::read_to_string(path).ok()?;
    let bytes = content.len();
    Some(FileAsset {
        content,
        bytes,
        position: [1.0, 2.0],
        available: true,
        hidden: "hidden".to_owned(),
    })
}

/// A marker asset that records its destruction, for asserting plugin-unload
/// teardown order relative to [`OrderComponent`].
#[asset_type]
pub struct OrderAsset {
    #[none]
    marker: u8,
}

impl Drop for OrderAsset {
    fn drop(&mut self) {
        UNLOAD_ORDER.lock().unwrap().push("asset");
    }
}

#[asset_type_creator(OrderAsset)]
fn create_order_asset(_scene: &mut Scene, _path: &str) -> Option<OrderAsset> {
    Some(OrderAsset { marker: 0 })
}

// ---------------------------------------------------------------------------
// Helpers + rstest fixtures
// ---------------------------------------------------------------------------

/// Reads the `value` field of a `"Counter"` component on `entity`.
pub fn get_counter(scene: &Scene, entity: Uuid) -> i64 {
    let (value,) = scene
        .query::<(&i64,)>(entity, "Counter", &["value"])
        .unwrap();
    *value
}

/// Writes the `value` field of a `"Counter"` component on `entity`.
pub fn set_counter(scene: &mut Scene, entity: Uuid, value: i64) {
    let (field,) = scene
        .query_mut::<(&mut i64,)>(entity, "Counter", &["value"])
        .unwrap();
    *field = value;
}

/// A fresh scene with one entity carrying `component`. The single canonical
/// "scene with one component" builder.
pub fn scene_with(component: &str) -> (Scene, Uuid) {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene.add_component(entity, component.to_owned()).unwrap();
    (scene, entity)
}

/// A fresh, empty scene.
#[fixture]
pub fn scene() -> Scene {
    Scene::new()
}

/// A scene holding one entity that already carries a `"Counter"` component.
#[fixture]
pub fn counter_scene() -> (Scene, Uuid) {
    scene_with("Counter")
}

/// Guards and resets the shared lib-binary system state. Holding this (via the
/// [`scene_state`] fixture) is the *only* thing a test must do to safely touch
/// [`ATTACH_COUNT`], [`TICK_ORDER`], [`LOAD_PATH`], etc.: acquisition resets
/// them all, and the held lock serializes against every other such test.
pub struct SceneState(std::sync::MutexGuard<'static, ()>);

#[fixture]
pub fn scene_state() -> SceneState {
    let guard = SCENE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    ATTACH_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    DETACH_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    DEFERRED_REMOVE_TARGET_PRESENT.store(false, std::sync::atomic::Ordering::SeqCst);
    DEFERRED_UNLOAD_PLUGIN_PRESENT.store(false, std::sync::atomic::Ordering::SeqCst);
    TICK_ORDER.lock().unwrap().clear();
    UNLOAD_ORDER.lock().unwrap().clear();
    *LOAD_PATH.lock().unwrap() = None;
    SceneState(guard)
}

/// The asset-test counterpart of [`SceneState`]: serializes and resets the
/// asset-creation state (`FileAsset` caching is observed via [`ASSET_CREATE_COUNT`]).
pub struct AssetState(std::sync::MutexGuard<'static, ()>);

#[fixture]
pub fn asset_state() -> AssetState {
    let guard = ASSET_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    ASSET_CREATE_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    AssetState(guard)
}
