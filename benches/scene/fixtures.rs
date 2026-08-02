use std::hint::black_box;

use wasserxr::scene::Scene;
use wasserxr::{Uuid, asset_type, asset_type_creator, component, component_creator, system};

pub(crate) const COMPONENT: &str = "BenchCounter";
const SYSTEMS: [&str; 4] = [
    "bench_system_one",
    "bench_system_two",
    "bench_system_three",
    "bench_system_four",
];

#[derive(Clone, Copy)]
pub(crate) struct Scale {
    pub(crate) name: &'static str,
    pub(crate) entities: usize,
    // Collections are number of resources and assets
    pub(crate) collections: usize,
}

impl Scale {
    pub(crate) fn entity_id(self) -> String {
        format!("{}_{}", self.name, self.entities)
    }

    pub(crate) fn collection_id(self) -> String {
        format!("{}_{}", self.name, self.collections)
    }
}

pub(crate) const SCALES: [Scale; 3] = [
    Scale {
        name: "small",
        entities: 100,
        collections: 10,
    },
    Scale {
        name: "medium",
        entities: 1_000,
        collections: 100,
    },
    Scale {
        name: "large",
        entities: 10_000,
        collections: 1_000,
    },
];

#[component]
#[derive(Default)]
struct BenchCounter {
    #[mutable]
    value: i64,
}

#[component_creator(BenchCounter)]
fn create_bench_counter(_scene: &mut Scene) -> Option<BenchCounter> {
    Some(BenchCounter::default())
}

#[asset_type]
struct BenchAsset {
    value: usize,
}

#[asset_type_creator(BenchAsset)]
fn create_bench_asset(_scene: &mut Scene, key: &str) -> Option<BenchAsset> {
    Some(BenchAsset { value: key.len() })
}

fn increment(scene: &mut Scene, entities: &[Vec<Uuid>]) {
    for entity in &entities[0] {
        let (value,) = scene
            .query_mut::<(&mut i64,)>(*entity, COMPONENT, &["value"])
            .unwrap();
        *value += 1;
    }
}

#[system(entities = [["BenchCounter"]])]
fn bench_system_one(scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    increment(scene, &entities);
}

#[system(entities = [["BenchCounter"]])]
fn bench_system_two(scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    for entity in &entities[0] {
        let (value,) = scene
            .query::<(&i64,)>(*entity, COMPONENT, &["value"])
            .unwrap();
        black_box(*value);
    }
}

#[system(entities = [["BenchCounter"]])]
fn bench_system_three(scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    increment(scene, &entities);
}

#[system(entities = [["BenchCounter"]])]
fn bench_system_four(scene: &mut Scene, _delta: f32, entities: Vec<Vec<Uuid>>) {
    increment(scene, &entities);
}

pub(crate) fn entity_fixture(count: usize, with_components: bool) -> (Scene, Vec<Uuid>) {
    let mut scene = Scene::new();
    let mut entities = Vec::with_capacity(count);
    for _ in 0..count {
        let entity = scene.add_entity();
        if with_components {
            scene.add_component(entity, COMPONENT.to_owned()).unwrap();
        }
        entities.push(entity);
    }
    (scene, entities)
}

pub(crate) fn add_resource(scene: &mut Scene, index: usize) {
    scene
        .add_resource(format!("resource-{index}"), index)
        .unwrap();
}

pub(crate) fn resource_fixture(count: usize) -> Scene {
    let mut scene = Scene::new();
    for index in 0..count {
        add_resource(&mut scene, index);
    }
    scene
}

pub(crate) fn add_asset(scene: &mut Scene, index: usize) {
    scene
        .ensure_asset_loaded("BenchAsset", &format!("asset-{index}"))
        .unwrap();
}

pub(crate) fn asset_fixture(count: usize) -> Scene {
    let mut scene = Scene::new();
    for index in 0..count {
        add_asset(&mut scene, index);
    }
    scene
}

pub(crate) fn add_collections(scene: &mut Scene, count: usize) {
    for index in 0..count {
        add_resource(scene, index);
        add_asset(scene, index);
    }
}

pub(crate) fn add_systems(scene: &mut Scene, count: usize) {
    for (priority, system) in SYSTEMS[..count].iter().enumerate() {
        scene.add_system((*system).to_owned(), priority).unwrap();
    }
}

pub(crate) fn representative_scene(scale: Scale, system_count: usize) -> Scene {
    let (mut scene, _) = entity_fixture(scale.entities, true);
    add_collections(&mut scene, scale.collections);
    add_systems(&mut scene, system_count);
    scene
}
