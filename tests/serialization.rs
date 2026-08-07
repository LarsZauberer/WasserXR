use serde::{Deserialize, Serialize};
use wasserxr::{
    component, component_creator,
    scene::{
        Scene,
        component::{FieldType, WXRComponentDescriptor, WXRComponentFieldDescriptor},
        plugin::{Version, WXRPluginDescriptor},
    },
};

#[component]
struct MyStruct {
    #[mutable]
    data: Data,
}

#[derive(Deserialize, Serialize)]
struct Data {
    a: Vec<usize>,
    b: [f32; 3],
}

#[component_creator(MyStruct)]
fn create_my_struct(_scene: &mut Scene) -> Option<MyStruct> {
    Some(MyStruct {
        data: Data {
            a: vec![],
            b: [1.0; 3],
        },
    })
}

static MY_STRUCT_FIELDS: [WXRComponentFieldDescriptor; 1] = [WXRComponentFieldDescriptor {
    name: c"data".as_ptr(),
    field_type: FieldType::Blob as u32,
    getter: Some(wxr_get_MyStruct_data),
    mutable: 1,
    serializer: Some(wxr_serialize_MyStruct_data),
    deserializer: Some(wxr_deserialize_MyStruct_data),
}];

static COMPONENTS: [WXRComponentDescriptor; 1] = [WXRComponentDescriptor {
    name: c"MyStruct".as_ptr(),
    creator: Some(wxr_create_MyStruct),
    destroyer: Some(wxr_destroy_MyStruct),
    fields: MY_STRUCT_FIELDS.as_ptr(),
    field_count: MY_STRUCT_FIELDS.len(),
    methods: std::ptr::null(),
    method_count: 0,
}];

static PLUGIN: WXRPluginDescriptor = WXRPluginDescriptor {
    version: Version::CURRENT,
    name: c"serialization-tests".as_ptr(),
    components: COMPONENTS.as_ptr(),
    component_count: COMPONENTS.len(),
    assets: std::ptr::null(),
    asset_count: 0,
    systems: std::ptr::null(),
    system_count: 0,
};

#[test]
fn test_nested_serialization() {
    let mut scene = Scene::new();
    unsafe { scene.load_static_plugin(&PLUGIN) }.unwrap();

    let entity = scene.add_entity();
    scene
        .add_component(entity, "MyStruct".to_owned())
        .expect("Failed to create MyStruct");

    let (data,) = scene
        .query_mut::<(&mut Data,)>(entity, "MyStruct", &["data"])
        .expect("Failed to get data field");

    data.a.push(5);
    data.b[1] = 0.5;
    data.b[2] = -1.0;

    let _ = data;

    unsafe { scene.reload() }.unwrap();

    let (data,) = scene
        .query::<(&Data,)>(entity, "MyStruct", &["data"])
        .expect("Failed to get data field");

    assert_eq!(data.a.len(), 1);
    assert_eq!(data.a[0], 5);

    assert_eq!(data.b[0], 1.0);
    assert_eq!(data.b[1], 0.5);
    assert_eq!(data.b[2], -1.0);
}
