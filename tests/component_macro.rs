use wasserxr::{
    Component, Scene,
    error::{ComponentError, SceneError},
};

#[Component]
#[derive(Default)]
pub struct MacroComponent {
    #[getter]
    my_int: i32,

    #[getter]
    #[setter]
    my_string: String,

    #[allow(dead_code)]
    hidden: i32,
}

#[test]
fn component_macro_registers_and_accesses_static_component() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    scene
        .add_component(entity, "MacroComponent".to_owned())
        .unwrap();

    assert_eq!(
        *scene
            .get::<i32>(entity, "MacroComponent", "my_int")
            .unwrap(),
        0
    );
    assert_eq!(
        scene
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        ""
    );

    let updated = "updated through setter".to_owned();
    scene
        .set(entity, "MacroComponent", "my_string", &updated)
        .unwrap();

    assert_eq!(
        scene
            .get::<String>(entity, "MacroComponent", "my_string")
            .unwrap(),
        "updated through setter"
    );
    assert_eq!(
        scene.get::<i32>(entity, "MacroComponent", "hidden"),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoGetter
        ))
    );
    assert_eq!(
        scene.set(entity, "MacroComponent", "hidden", &7_i32),
        Err(SceneError::ComponentFieldError(
            ComponentError::FieldNoSetter
        ))
    );
}
