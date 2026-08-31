//! Informational before/after benchmarks for the ECS rewrite.
//!
//! `microbenchmarks/*` times one public Scene operation. Fixture construction
//! is outside each timed section. `scenario_benchmarks/*` times the lifecycle
//! named by the group; the mixed workload intentionally includes its complete
//! lifecycle.
//!
//! Dataset sizes are `small_100`, `medium_1000`, and `large_10000` entities.
//! Asset and resource collections use 10, 100, and 1,000 entries respectively.
//! Entity IDs still come from `Scene::add_entity`, so UUID generation remains
//! part of insertion while every workload choice and value is fixed.
//!
//! Timed scenario boundaries:
//!
//! - `scene_construction`: starts empty and adds entities, components,
//!   resources, assets, and four systems.
//! - `system_tick`: times only one tick of a prebuilt Scene with one or four
//!   systems using shared and mutable component access.
//! - `entity_component_lifecycle`: creates entities, attaches components,
//!   mutates and reads every component, then removes every entity.
//! - `serialization`: times only serialization or deserialization.
//! - `mixed_scene`: creates every subsystem fixture, ticks, queries and
//!   mutates, serializes, deserializes into a fresh Scene, then removes half
//!   the entities.
//! - `hot_reload`: times only `Scene::reload` on a prebuilt dynamic-plugin
//!   Scene.
//!
//! Record the pre-rewrite baseline on a quiet machine with:
//!
//! ```text
//! cargo bench --bench scene -- --save-baseline pre-ecs-rewrite
//! ```
//!
//! Compare a later rewrite issue on the same machine and toolchain with:
//!
//! ```text
//! cargo bench --bench scene -- --baseline pre-ecs-rewrite
//! ```
//!
//! Results are informational and are not comparable across unrelated machines.
//! The Linux-only hot-reload group compiles the synthetic C plugin fixture
//! once, outside the timed section, and requires `cbindgen` and `gcc` on
//! `PATH`.

#[path = "scene/fixtures.rs"]
mod fixtures;
#[path = "scene/micro.rs"]
mod micro;
#[path = "scene/scenarios.rs"]
mod scenarios;

use criterion::{criterion_group, criterion_main};
use micro::{assets, components, entities, resources};
use scenarios::{construction, lifecycle, mixed, serialization, tick};

#[cfg(target_os = "linux")]
use scenarios::hotreload;

#[cfg(target_os = "linux")]
criterion_group!(
    scene_benchmarks,
    entities,
    components,
    resources,
    assets,
    construction,
    tick,
    lifecycle,
    serialization,
    mixed,
    hotreload,
);

#[cfg(not(target_os = "linux"))]
criterion_group!(
    scene_benchmarks,
    entities,
    components,
    resources,
    assets,
    construction,
    tick,
    lifecycle,
    serialization,
    mixed,
);
criterion_main!(scene_benchmarks);
