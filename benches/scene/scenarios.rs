use std::hint::black_box;

#[cfg(target_os = "linux")]
use std::{fs, path::PathBuf, process::Command, sync::OnceLock, time::Duration};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use wasserxr::scene::Scene;

use super::fixtures::{
    COMPONENT, SCALES, Scale, add_collections, add_systems, entity_fixture, representative_scene,
};

/// Times complete representative Scene construction from an empty Scene.
pub(crate) fn construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/scene_construction");
    for scale in SCALES {
        group.throughput(Throughput::Elements(scale.entities as u64));
        group.bench_function(BenchmarkId::new("four_systems", scale.entity_id()), |b| {
            b.iter(|| black_box(representative_scene(scale, 4)));
        });
    }
    group.finish();
}

/// Times only `Scene::tick`; Scene construction and system registration are excluded.
pub(crate) fn tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/system_tick");
    for scale in SCALES {
        for system_count in [1, 4] {
            let mut scene = representative_scene(scale, system_count);
            group.throughput(Throughput::Elements(scale.entities as u64));
            group.bench_function(
                BenchmarkId::new(format!("{system_count}_systems"), scale.entity_id()),
                |b| b.iter(|| black_box(scene.tick())),
            );
        }
    }
    group.finish();
}

/// Times bulk creation, attachment, mutation/query, and removal as one lifecycle.
pub(crate) fn lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/entity_component_lifecycle");
    for scale in SCALES {
        group.throughput(Throughput::Elements(scale.entities as u64));
        group.bench_function(BenchmarkId::new("complete", scale.entity_id()), |b| {
            b.iter(|| {
                let (mut scene, entities) = entity_fixture(scale.entities, true);
                for entity in &entities {
                    let (value,) = scene
                        .query_mut::<(&mut i64,)>(*entity, COMPONENT, &["value"])
                        .unwrap();
                    *value += 1;
                    black_box(*value);
                }
                for entity in entities {
                    scene.remove_entity(entity).unwrap();
                }
                black_box(scene)
            });
        });
    }
    group.finish();
}

/// Times complete serialization or deserialization; input Scene/bytes setup is excluded.
pub(crate) fn serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/serialization");
    for scale in SCALES {
        let scene = representative_scene(scale, 4);
        let bytes = scene.serialize().unwrap();
        group.throughput(Throughput::Elements(scale.entities as u64));
        group.bench_function(BenchmarkId::new("serialize", scale.entity_id()), |b| {
            b.iter(|| black_box(scene.serialize().unwrap()));
        });
        group.bench_function(BenchmarkId::new("deserialize", scale.entity_id()), |b| {
            b.iter_batched(
                Scene::new,
                |mut scene| black_box(scene.deserialize(black_box(&bytes)).unwrap()),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Times the complete mixed lifecycle documented in the benchmark target.
pub(crate) fn mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/mixed_scene");
    for scale in SCALES {
        group.throughput(Throughput::Elements(scale.entities as u64));
        group.bench_function(BenchmarkId::new("complete", scale.entity_id()), |b| {
            b.iter(|| {
                let (mut scene, entities) = entity_fixture(scale.entities, true);
                add_collections(&mut scene, scale.collections);
                add_systems(&mut scene, 4);
                scene.tick();
                for entity in &entities {
                    let (value,) = scene
                        .query_mut::<(&mut i64,)>(*entity, COMPONENT, &["value"])
                        .unwrap();
                    *value += 1;
                    black_box(*value);
                }
                let bytes = scene.serialize().unwrap();
                let mut loaded = Scene::new();
                loaded.deserialize(&bytes).unwrap();
                for entity in entities.into_iter().take(scale.entities / 2) {
                    loaded.remove_entity(entity).unwrap();
                }
                black_box(loaded)
            });
        });
    }
    group.finish();
}

#[cfg(target_os = "linux")]
fn dynamic_plugin() -> &'static str {
    static PLUGIN: OnceLock<String> = OnceLock::new();
    PLUGIN.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let build = root.join("target/criterion-plugin");
        let include = build.join("include");
        let header = include.join("wasserxr.h");
        let plugin = build.join("libwasserxr_benchmark_plugin.so");
        fs::create_dir_all(&include).unwrap();

        let status = Command::new("cbindgen")
            .arg(&root)
            .arg("--config")
            .arg(root.join("cbindgen.toml"))
            .arg("--output")
            .arg(&header)
            .status()
            .expect("cbindgen is required for the hot-reload benchmark");
        assert!(
            status.success(),
            "cbindgen failed for the hot-reload benchmark"
        );

        let status = Command::new("gcc")
            .arg("-std=c11")
            .arg("-fPIC")
            .arg("-shared")
            .arg("-I")
            .arg(&include)
            .arg(root.join("tests/fixtures/c_abi_plugin.c"))
            .arg("-o")
            .arg(&plugin)
            .status()
            .expect("gcc is required for the hot-reload benchmark");
        assert!(status.success(), "gcc failed for the hot-reload benchmark");
        plugin.to_string_lossy().into_owned()
    })
}

#[cfg(target_os = "linux")]
fn dynamic_scene(scale: Scale) -> Scene {
    let mut scene = Scene::new();
    scene.load_plugin(dynamic_plugin().to_owned()).unwrap();
    for _ in 0..scale.entities {
        let entity = scene.add_entity();
        scene
            .add_component(entity, "abi_counter".to_owned())
            .unwrap();
    }
    scene
        .add_system("abi_counter_system".to_owned(), 1)
        .unwrap();
    scene
}

/// Linux only. Times `Scene::reload`; plugin compilation and Scene setup are excluded.
#[cfg(target_os = "linux")]
pub(crate) fn hotreload(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_benchmarks/hotreload");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    for scale in SCALES {
        group.throughput(Throughput::Elements(scale.entities as u64));
        group.bench_function(BenchmarkId::new("reload", scale.entity_id()), |b| {
            let mut scene = dynamic_scene(scale);
            b.iter(|| black_box(scene.reload().unwrap()));
        });
    }
    group.finish();
}
