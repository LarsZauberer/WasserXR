use std::ffi::{CString, c_void};

use wasserxr::{
    bindings::{
        WXRSceneError,
        scene::{
            wxr_add_component, wxr_add_entity, wxr_create_scene, wxr_destroy_scene, wxr_get_method,
            wxr_method_argument, wxr_method_call, wxr_method_destroy, wxr_query,
        },
        wxr_error,
    },
    component, component_creator,
    error::SceneError,
    method,
    scene::{Scene, component::methods::WXRMethodStatus},
};

#[component]
#[derive(Default)]
pub struct MethodComponent {
    #[getter]
    #[mutable]
    value: i32,
}

#[component_creator(MethodComponent)]
pub fn create_method_component(_scene: &mut Scene) -> Option<MethodComponent> {
    Some(MethodComponent::default())
}

#[method(MethodComponent)]
fn add(
    _scene: &mut Scene,
    component: &mut MethodComponent,
    amount: &mut i32,
) -> Result<*mut c_void, i32> {
    component.value += *amount;
    Ok(std::ptr::null_mut())
}

#[method(MethodComponent)]
fn combine(
    _scene: &mut Scene,
    component: &mut MethodComponent,
    a: &mut i32,
    b: &mut i32,
) -> Result<*mut c_void, i32> {
    component.value = *a + *b;
    Ok(std::ptr::null_mut())
}

#[method(MethodComponent)]
fn value_ptr(_scene: &mut Scene, component: &mut MethodComponent) -> Result<*mut c_void, i32> {
    Ok(&mut component.value as *mut i32 as *mut c_void)
}

#[method(MethodComponent)]
fn always_fail(_scene: &mut Scene, _component: &mut MethodComponent) -> Result<*mut c_void, i32> {
    Err(7)
}

#[test]
fn get_method_mutates_component_through_argument() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let mut amount = 5_i32;
    let result = scene
        .get_method(entity, "MethodComponent", "add")
        .unwrap()
        .argument(c"amount", &mut amount)
        .call();
    assert_eq!(result.status, WXRMethodStatus::Success);
    assert_eq!(result.action_error, 0);
    assert!(result.value.is_null());

    let (value,) = scene
        .query::<(&i32,)>(entity, "MethodComponent", &["value"])
        .unwrap();
    assert_eq!(*value, 5);
}

#[test]
fn get_method_resolves_arguments_by_name_out_of_order() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let mut a = 3_i32;
    let mut b = 4_i32;
    let result = scene
        .get_method(entity, "MethodComponent", "combine")
        .unwrap()
        .argument(c"b", &mut b)
        .argument(c"a", &mut a)
        .call();
    assert_eq!(result.status, WXRMethodStatus::Success);

    let (value,) = scene
        .query::<(&i32,)>(entity, "MethodComponent", &["value"])
        .unwrap();
    assert_eq!(*value, 7);
}

#[test]
fn get_method_returns_user_pointer() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let result = scene
        .get_method(entity, "MethodComponent", "value_ptr")
        .unwrap()
        .call();
    assert_eq!(result.status, WXRMethodStatus::Success);
    assert!(!result.value.is_null());
    assert_eq!(unsafe { *(result.value as *const i32) }, 0);
}

#[test]
fn get_method_reports_missing_argument() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let result = scene
        .get_method(entity, "MethodComponent", "add")
        .unwrap()
        .call();
    assert_eq!(result.status, WXRMethodStatus::MissingArgument);
}

#[test]
fn get_method_reports_duplicate_argument() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let mut first = 1_i32;
    let mut second = 2_i32;
    let result = scene
        .get_method(entity, "MethodComponent", "add")
        .unwrap()
        .argument(c"amount", &mut first)
        .argument(c"amount", &mut second)
        .call();
    assert_eq!(result.status, WXRMethodStatus::DuplicateArgument);
}

#[test]
fn get_method_reports_action_error() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    let result = scene
        .get_method(entity, "MethodComponent", "always_fail")
        .unwrap()
        .call();
    assert_eq!(result.status, WXRMethodStatus::ActionError);
    assert_eq!(result.action_error, 7);
    assert!(result.value.is_null());
}

#[test]
fn get_method_reports_unknown_method() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "MethodComponent".to_owned())
        .unwrap();

    assert_eq!(
        scene
            .get_method(entity, "MethodComponent", "does_not_exist")
            .map(|_| ())
            .unwrap_err(),
        SceneError::MethodNotFound
    );
}

#[test]
fn get_method_reports_missing_component() {
    let mut scene = Scene::new();
    let entity = scene.add_entity();

    assert_eq!(
        scene
            .get_method(entity, "MethodComponent", "add")
            .map(|_| ())
            .unwrap_err(),
        SceneError::ComponentNotFound
    );
}

#[test]
fn c_method_call_round_trip() {
    let scene = wxr_create_scene();
    let entity = wxr_add_entity(scene);
    let component = CString::new("MethodComponent").unwrap();
    assert_eq!(
        unsafe { wxr_add_component(scene, entity, component.as_ptr()) },
        0
    );

    let method_name = CString::new("add").unwrap();
    let method = unsafe { wxr_get_method(scene, entity, component.as_ptr(), method_name.as_ptr()) };
    assert!(!method.is_null());
    assert_eq!(wxr_error(), WXRSceneError::NoError);

    let argument_name = CString::new("amount").unwrap();
    let mut amount = 9_i32;
    unsafe {
        wxr_method_argument(
            method,
            argument_name.as_ptr(),
            &mut amount as *mut i32 as *mut c_void,
        );
    }

    let result = unsafe { wxr_method_call(method) };
    assert_eq!(result.status, WXRMethodStatus::Success);

    unsafe {
        wxr_destroy_scene(scene);
    }
}

#[test]
fn c_get_method_reports_unknown_method() {
    let scene = wxr_create_scene();
    let entity = wxr_add_entity(scene);
    let component = CString::new("MethodComponent").unwrap();
    unsafe {
        wxr_add_component(scene, entity, component.as_ptr());
    }

    let method_name = CString::new("does_not_exist").unwrap();
    let method = unsafe { wxr_get_method(scene, entity, component.as_ptr(), method_name.as_ptr()) };
    assert!(method.is_null());
    assert_eq!(wxr_error(), WXRSceneError::MethodNotFound);

    unsafe {
        wxr_destroy_scene(scene);
    }
}

#[test]
fn c_method_destroy_abandons_without_calling() {
    let scene = wxr_create_scene();
    let entity = wxr_add_entity(scene);
    let component = CString::new("MethodComponent").unwrap();
    unsafe {
        wxr_add_component(scene, entity, component.as_ptr());
    }

    let method_name = CString::new("add").unwrap();
    let method = unsafe { wxr_get_method(scene, entity, component.as_ptr(), method_name.as_ptr()) };
    assert!(!method.is_null());

    // Abandoning the handle must not call the method or leak it.
    unsafe {
        wxr_method_destroy(method);
    }

    let field = CString::new("value").unwrap();
    let value = unsafe { wxr_query(scene, entity, component.as_ptr(), field.as_ptr()) };
    assert_eq!(unsafe { *(value as *const i32) }, 0);

    unsafe {
        wxr_destroy_scene(scene);
    }
}
