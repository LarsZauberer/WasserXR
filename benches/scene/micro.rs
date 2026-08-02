use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion};
use wasserxr::scene::Scene;

use super::fixtures::{COMPONENT, SCALES, asset_fixture, entity_fixture, resource_fixture};

/// Times one entity operation; Scene population and removal fixtures are excluded.
pub(crate) fn entities(c: &mut Criterion) {
    let mut group = c.benchmark_group("microbenchmarks/entities");

    group.bench_function("insert/individual", |b| {
        b.iter_batched(
            Scene::new,
            |mut scene| black_box(scene.add_entity()),
            BatchSize::SmallInput,
        );
    });

    // Build a scene with the entity scale and lookup a single entity
    for scale in SCALES {
        let (scene, entities) = entity_fixture(scale.entities, false);
        let target = entities[scale.entities / 2];
        group.bench_function(BenchmarkId::new("lookup", scale.entity_id()), |b| {
            b.iter(|| black_box(scene.get_entity_name(black_box(target)).unwrap()));
        });
    }

    group.bench_function("remove/individual", |b| {
        b.iter_batched(
            || {
                let mut scene = Scene::new();
                let entity = scene.add_entity();
                (scene, entity)
            },
            |(mut scene, entity)| black_box(scene.remove_entity(entity).unwrap()),
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// Times one component operation; entity/component construction is excluded.
pub(crate) fn components(c: &mut Criterion) {
    let mut group = c.benchmark_group("microbenchmarks/components");

    group.bench_function("insert/individual", |b| {
        b.iter_batched(
            || {
                let mut scene = Scene::new();
                let entity = scene.add_entity();
                (scene, entity)
            },
            |(mut scene, entity)| {
                black_box(scene.add_component(entity, COMPONENT.to_owned()).unwrap())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("remove/individual", |b| {
        b.iter_batched(
            || {
                let (scene, mut entities) = entity_fixture(1, true);
                (scene, entities.pop().unwrap())
            },
            |(mut scene, entity)| black_box(scene.remove_component(entity, COMPONENT).unwrap()),
            BatchSize::SmallInput,
        );
    });

    for scale in SCALES {
        let (mut scene, entities) = entity_fixture(scale.entities, true);
        let target = entities[scale.entities / 2];
        group.bench_function(BenchmarkId::new("query_shared", scale.entity_id()), |b| {
            b.iter(|| {
                let (value,) = scene
                    .query::<(&i64,)>(black_box(target), COMPONENT, &["value"])
                    .unwrap();
                black_box(*value)
            });
        });
        group.bench_function(BenchmarkId::new("query_mut", scale.entity_id()), |b| {
            b.iter(|| {
                let (value,) = scene
                    .query_mut::<(&mut i64,)>(black_box(target), COMPONENT, &["value"])
                    .unwrap();
                *value = black_box(*value + 1);
            });
        });
    }
    group.finish();
}

/// Times resource lookup only; collection construction is excluded.
pub(crate) fn resources(c: &mut Criterion) {
    let mut group = c.benchmark_group("microbenchmarks/resources");
    for scale in SCALES {
        let mut scene = resource_fixture(scale.collections);
        let key = format!("resource-{}", scale.collections - 1);
        group.bench_function(
            BenchmarkId::new("access_shared", scale.collection_id()),
            |b| {
                b.iter(|| black_box(*scene.get_resource::<usize>(black_box(&key)).unwrap()));
            },
        );
        group.bench_function(BenchmarkId::new("access_mut", scale.collection_id()), |b| {
            b.iter(|| {
                let value = scene.get_mut_resource::<usize>(black_box(&key)).unwrap();
                *value = black_box(*value + 1);
            });
        });
    }
    group.finish();
}

/// Times cached asset lookup and field query; asset creation is excluded.
pub(crate) fn assets(c: &mut Criterion) {
    let mut group = c.benchmark_group("microbenchmarks/assets");
    for scale in SCALES {
        let mut scene = asset_fixture(scale.collections);
        let key = format!("asset-{}", scale.collections - 1);
        group.bench_function(
            BenchmarkId::new("cached_lookup", scale.collection_id()),
            |b| {
                b.iter(|| {
                    black_box(
                        scene
                            .ensure_asset_loaded("BenchAsset", black_box(&key))
                            .unwrap(),
                    )
                });
            },
        );
        group.bench_function(BenchmarkId::new("query", scale.collection_id()), |b| {
            b.iter(|| {
                let (value,) = scene
                    .asset_query_loaded::<(&usize,)>("BenchAsset", black_box(&key), &["value"])
                    .unwrap();
                black_box(*value)
            });
        });
    }
    group.finish();
}
