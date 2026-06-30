use wasserxr::{component, component_creator, scene::Scene};

#[component]
struct MyStruct {
    #[mutable]
    data: Data,
}

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

#[test]
fn test_nested_serialization() {
    let mut scene = Scene::new();

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

    let _ = scene.reload();

    let (data,) = scene
        .query::<(&Data,)>(entity, "MyStruct", &["data"])
        .expect("Failed to get data field");

    assert_eq!(data.a.len(), 1);
    assert_eq!(data.a[0], 5);

    assert_eq!(data.b[0], 1.0);
    assert_eq!(data.b[1], 0.5);
    assert_eq!(data.b[2], -1.0);
}
