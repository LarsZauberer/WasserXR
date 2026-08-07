use std::ffi::{c_char, c_void};

use wasserxr::scene::{
    Scene, SceneError,
    component::{WXRComponentDescriptor, WXRComponentFieldDescriptor},
    plugin::{ManifestError, PluginError, Version, WXRPluginDescriptor},
    system::{WXRSystemDescriptor, WXRSystemEntityGroupDescriptor},
};

unsafe extern "C" fn create_component(_scene: *mut Scene) -> *mut c_void {
    Box::into_raw(Box::new(0_u8)).cast()
}

unsafe extern "C" fn destroy_component(data: *mut c_void) {
    unsafe { drop(Box::from_raw(data.cast::<u8>())) };
}

unsafe extern "C" fn run_system(
    _scene: *mut Scene,
    _delta: f32,
    _entities: *const *const wasserxr::bindings::scene::WXREntity,
    _entity_counts: *const usize,
    _entity_group_count: usize,
) {
}

static TAKEN_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"taken".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
    methods: std::ptr::null(),
    method_count: 0,
}];
static FIRST_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"first".as_ptr(),
    components: TAKEN_COMPONENTS.as_ptr(),
    component_count: TAKEN_COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

static NEW_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"new_component".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
    methods: std::ptr::null(),
    method_count: 0,
}];
static COLLIDING_SYSTEMS: [WXRSystemDescriptor; 1] = [WXRSystemDescriptor {
    name: c"taken".as_ptr(),
    runner: Some(run_system),
    attach: None,
    detach: None,
    entity_groups: std::ptr::null(),
    entity_group_count: 0,
}];
static COLLIDING_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"second".as_ptr(),
    components: NEW_COMPONENTS.as_ptr(),
    component_count: NEW_COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: COLLIDING_SYSTEMS.as_ptr(),
    system_count: COLLIDING_SYSTEMS.len(),
};

static EMPTY_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"empty".as_ptr(),
    components: std::ptr::null(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

static INCOMPATIBLE_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version {
        major: Version::CURRENT.major,
        minor: Version::CURRENT.minor + 1,
        patch: 0,
    },
    name: c"incompatible".as_ptr(),
    components: std::ptr::null(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

static INVALID_POINTER_COUNT_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"bad-pointer-count".as_ptr(),
    components: TAKEN_COMPONENTS.as_ptr(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

static MISSING_CREATOR_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"missing_creator".as_ptr(),
    creator: None,
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
    methods: std::ptr::null(),
    method_count: 0,
}];
static MISSING_CALLBACK_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"missing-callback".as_ptr(),
    components: MISSING_CREATOR_COMPONENTS.as_ptr(),
    component_count: MISSING_CREATOR_COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

const GROUP_AB: [*const c_char; 2] = [c"A".as_ptr(), c"B".as_ptr()];
const GROUP_BA: [*const c_char; 2] = [c"B".as_ptr(), c"A".as_ptr()];
static DUPLICATE_GROUPS: [WXRSystemEntityGroupDescriptor; 2] = [
    WXRSystemEntityGroupDescriptor {
        components: GROUP_AB.as_ptr(),
        component_count: GROUP_AB.len(),
    },
    WXRSystemEntityGroupDescriptor {
        components: GROUP_BA.as_ptr(),
        component_count: GROUP_BA.len(),
    },
];
static DUPLICATE_GROUP_SYSTEMS: [WXRSystemDescriptor; 1] = [WXRSystemDescriptor {
    name: c"duplicate_groups".as_ptr(),
    runner: Some(run_system),
    attach: None,
    detach: None,
    entity_groups: DUPLICATE_GROUPS.as_ptr(),
    entity_group_count: DUPLICATE_GROUPS.len(),
}];
static DUPLICATE_GROUP_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"duplicate-group-plugin".as_ptr(),
    components: std::ptr::null(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: DUPLICATE_GROUP_SYSTEMS.as_ptr(),
    system_count: DUPLICATE_GROUP_SYSTEMS.len(),
};

static UNKNOWN_FIELD_TYPE_FIELDS: [WXRComponentFieldDescriptor; 1] =
    [WXRComponentFieldDescriptor {
        name: c"unknown".as_ptr(),
        field_type: u32::MAX,
        getter: None,
        mutable: 0,
        serializer: None,
        deserializer: None,
    }];
static UNKNOWN_FIELD_TYPE_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"unknown_field_type".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: UNKNOWN_FIELD_TYPE_FIELDS.as_ptr(),
    field_count: UNKNOWN_FIELD_TYPE_FIELDS.len(),
    methods: std::ptr::null(),
    method_count: 0,
}];
static UNKNOWN_FIELD_TYPE_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"unknown-field-type-plugin".as_ptr(),
    components: UNKNOWN_FIELD_TYPE_COMPONENTS.as_ptr(),
    component_count: UNKNOWN_FIELD_TYPE_COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

const LATE_GROUP_COMPONENTS: [*const c_char; 1] = [c"late_component".as_ptr()];
static LATE_GROUPS: [WXRSystemEntityGroupDescriptor; 1] = [WXRSystemEntityGroupDescriptor {
    components: LATE_GROUP_COMPONENTS.as_ptr(),
    component_count: LATE_GROUP_COMPONENTS.len(),
}];
static LATE_SYSTEMS: [WXRSystemDescriptor; 1] = [WXRSystemDescriptor {
    name: c"late_system".as_ptr(),
    runner: Some(run_system),
    attach: None,
    detach: None,
    entity_groups: LATE_GROUPS.as_ptr(),
    entity_group_count: LATE_GROUPS.len(),
}];
static LATE_SYSTEM_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"late-system-plugin".as_ptr(),
    components: std::ptr::null(),
    component_count: 0,
    assets: std::ptr::null(),
    asset_count: 0,
    systems: LATE_SYSTEMS.as_ptr(),
    system_count: LATE_SYSTEMS.len(),
};
static LATE_COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"late_component".as_ptr(),
    creator: Some(create_component),
    destroyer: Some(destroy_component),
    fields: std::ptr::null(),
    field_count: 0,
    methods: std::ptr::null(),
    method_count: 0,
}];
static LATE_COMPONENT_PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"late-component-plugin".as_ptr(),
    components: LATE_COMPONENTS.as_ptr(),
    component_count: LATE_COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

#[test]
fn empty_static_plugin_is_identified_and_unloaded_by_manifest_name() {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&EMPTY_PLUGIN) }.unwrap();
    assert_eq!(scene.get_plugins(), ["empty"]);
    scene.unload_plugin("empty").unwrap();
    assert!(scene.get_plugins().is_empty());
}

#[test]
fn plugin_install_is_atomic_across_global_definition_collisions() {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&FIRST_PLUGIN) }.unwrap();

    assert_eq!(
        unsafe { scene.load_static_plugin(&COLLIDING_PLUGIN) },
        Err(SceneError::Plugin(PluginError::DefinitionCollision(
            "taken".to_owned()
        )))
    );
    assert_eq!(scene.get_plugins(), ["first"]);

    let entity = scene.add_entity();
    assert!(matches!(
        scene.add_component(entity, "new_component".to_owned()),
        Err(SceneError::Component(_))
    ));
}

#[test]
fn static_plugin_definitions_are_removed_on_unload() {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&FIRST_PLUGIN) }.unwrap();
    let entity = scene.add_entity();
    scene.add_component(entity, "taken".to_owned()).unwrap();

    scene.unload_plugin("first").unwrap();

    assert!(!scene.has_component(entity, "taken"));
    assert!(scene.add_component(entity, "taken".to_owned()).is_err());
}

