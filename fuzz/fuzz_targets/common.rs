use wasserxr::{Uuid, component, component_creator, scene::Scene, system};

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
