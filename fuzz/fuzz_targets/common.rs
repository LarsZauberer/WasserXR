use wasserxr::{
    Uuid, component, component_creator,
    scene::{
        Scene,
        component::{FieldType, WXRComponentDescriptor, WXRComponentFieldDescriptor},
        plugin::{Version, WXRPluginDescriptor},
        system::WXRSystemDescriptor,
    },
    system,
};

#[derive(Default)]
pub struct Blob;

#[component]
#[derive(Default)]
pub struct FuzzFields {
    #[mutable]
    integer: i64,
    #[mutable]
    float: f64,
    #[mutable]
    vector: [f32; 3],
    #[mutable]
    character: char,
    #[mutable]
    text: String,
    #[mutable]
    boolean: bool,
    #[getter]
    blob: Blob,
}

#[component_creator(FuzzFields)]
fn create_fuzz_fields(_scene: &mut Scene) -> Option<FuzzFields> {
    Some(FuzzFields::default())
}

#[system]
pub fn fuzz_system(_scene: &mut Scene, _delta: f32, _entities: Vec<Vec<Uuid>>) {}

macro_rules! field {
    ($name:literal, $type:ident, $getter:ident, $mutable:expr, $serializer:expr, $deserializer:expr) => {
        WXRComponentFieldDescriptor {
            name: concat!($name, "\0").as_ptr().cast(),
            field_type: FieldType::$type as u32,
            getter: Some($getter),
            mutable: $mutable,
            serializer: $serializer,
            deserializer: $deserializer,
        }
    };
}

static FIELDS: [WXRComponentFieldDescriptor; 7] = [
    field!("integer", I64, wxr_get_FuzzFields_integer, 1, Some(wxr_serialize_FuzzFields_integer), Some(wxr_deserialize_FuzzFields_integer)),
    field!("float", F64, wxr_get_FuzzFields_float, 1, Some(wxr_serialize_FuzzFields_float), Some(wxr_deserialize_FuzzFields_float)),
    field!("vector", F32Vec3, wxr_get_FuzzFields_vector, 1, Some(wxr_serialize_FuzzFields_vector), Some(wxr_deserialize_FuzzFields_vector)),
    field!("character", Char, wxr_get_FuzzFields_character, 1, Some(wxr_serialize_FuzzFields_character), Some(wxr_deserialize_FuzzFields_character)),
    field!("text", String, wxr_get_FuzzFields_text, 1, Some(wxr_serialize_FuzzFields_text), Some(wxr_deserialize_FuzzFields_text)),
    field!("boolean", Boolean, wxr_get_FuzzFields_boolean, 1, Some(wxr_serialize_FuzzFields_boolean), Some(wxr_deserialize_FuzzFields_boolean)),
    field!("blob", Blob, wxr_get_FuzzFields_blob, 0, None, None),
];
static COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"FuzzFields".as_ptr(),
    creator: Some(wxr_create_FuzzFields),
    destroyer: Some(wxr_destroy_FuzzFields),
    fields: FIELDS.as_ptr(),
    field_count: FIELDS.len(),
    methods: std::ptr::null(),
    method_count: 0,
}];
static SYSTEMS: [WXRSystemDescriptor; 1] = [WXRSystemDescriptor {
    name: c"fuzz_system".as_ptr(),
    runner: Some(wxr_system_fuzz_system),
    attach: None,
    detach: None,
    entity_groups: std::ptr::null(),
    entity_group_count: 0,
}];
static PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"fuzz-fixtures".as_ptr(),
    components: COMPONENTS.as_ptr(),
    component_count: COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: SYSTEMS.as_ptr(),
    system_count: SYSTEMS.len(),
};

pub fn fixture_scene() -> Scene {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&PLUGIN) }.unwrap();
    scene
}
