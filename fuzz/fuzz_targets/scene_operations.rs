#![no_main]

mod common;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasserxr::{Uuid, scene::Scene};

#[derive(Arbitrary, Debug)]
enum Operation {
    AddEntity(String),
    RenameEntity { index: usize, name: String },
    RemoveEntity(usize),
    AddComponent(usize),
    RemoveComponent(usize),
    AddSystem(usize),
    RemoveSystem,
    SetResource(u64),
    SerializeRoundTrip,
    Reset,
    Tick,
}

#[derive(Arbitrary, Debug)]
struct Input {
    operations: Vec<Operation>,
}

struct EntityModel {
    id: Uuid,
    name: String,
    has_component: bool,
}

/// Verifies that the scene matches the fuzzer's expected state.
fn verify(scene: &Scene, entities: &[EntityModel], has_system: bool, resource: Option<u64>) {
    let mut expected_entities: Vec<_> = entities.iter().map(|entity| entity.id).collect();
    expected_entities.sort();
    assert_eq!(scene.get_entities(), expected_entities);

    for entity in entities {
        assert_eq!(
            scene
                .get_entity_name(entity.id)
                .expect("modeled entity must exist"),
            entity.name,
        );
        assert_eq!(
            scene.has_component(entity.id, "FuzzFields"),
            entity.has_component,
        );
    }

    assert_eq!(
        scene.get_systems(),
        has_system
            .then(|| "fuzz_system".to_owned())
            .into_iter()
            .collect::<Vec<_>>()
    );
    match resource {
        Some(value) => assert_eq!(
            *scene
                .get_resource::<u64>("fuzz")
                .expect("modeled resource must exist"),
            value,
        ),
        None => assert!(scene.get_resource::<u64>("fuzz").is_err()),
    }
}

// Run: cargo fuzz run scene_operations
// Reproduce: cargo fuzz run scene_operations fuzz/artifacts/scene_operations/<artifact>
fuzz_target!(|input: Input| {
    let mut scene = common::fixture_scene();
    let mut entities = Vec::<EntityModel>::new();
    let mut has_system = false;
    let mut resource = None;

    for operation in input.operations.into_iter().take(64) {
        match operation {
            Operation::AddEntity(name) => {
                let id = scene.add_entity();
                scene
                    .set_entity_name(id, name.clone())
                    .expect("newly added entity must be nameable");
                entities.push(EntityModel {
                    id,
                    name,
                    has_component: false,
                });
            }
            Operation::RenameEntity { index, name } if !entities.is_empty() => {
                let index = index % entities.len();
                let entity = &mut entities[index];
                scene
                    .set_entity_name(entity.id, name.clone())
                    .expect("modeled entity must be nameable");
                entity.name = name;
            }
            Operation::RemoveEntity(index) if !entities.is_empty() => {
                let entity = entities.swap_remove(index % entities.len());
                scene
                    .remove_entity(entity.id)
                    .expect("modeled entity must be removable");
            }
            Operation::AddComponent(index) if !entities.is_empty() => {
                let index = index % entities.len();
                let entity = &mut entities[index];
                if !entity.has_component {
                    scene
                        .add_component(entity.id, "FuzzFields".to_owned())
                        .expect("modeled entity must accept the fuzz component");
                    entity.has_component = true;
                }
            }
            Operation::RemoveComponent(index) if !entities.is_empty() => {
                let index = index % entities.len();
                let entity = &mut entities[index];
                if entity.has_component {
                    scene
                        .remove_component(entity.id, "FuzzFields")
                        .expect("modeled component must be removable");
                    entity.has_component = false;
                }
            }
            Operation::AddSystem(priority) if !has_system => {
                scene
                    .add_system("fuzz_system".to_owned(), priority)
                    .expect("the statically linked fuzz system must be available");
                has_system = true;
            }
            Operation::RemoveSystem if has_system => {
                scene
                    .remove_system("fuzz_system")
                    .expect("modeled system must be removable");
                has_system = false;
            }
            Operation::SetResource(value) => {
                if let Some(resource) = resource.as_mut() {
                    *scene
                        .get_mut_resource::<u64>("fuzz")
                        .expect("modeled resource must be mutable") = value;
                    *resource = value;
                } else {
                    scene
                        .add_resource("fuzz".to_owned(), value)
                        .expect("new fuzz resource must be insertable");
                    resource = Some(value);
                }
            }
            Operation::SerializeRoundTrip => {
                let bytes = scene
                    .serialize()
                    .expect("a valid modeled scene must serialize");
                scene
                    .deserialize(&bytes)
                    .expect("a serialized valid modeled scene must deserialize");
            }
            Operation::Reset => {
                scene.reset().expect("a valid modeled scene must reset");
                entities.clear();
                has_system = false;
            }
            Operation::Tick => {
                assert!(scene.tick());
            }
            _ => {}
        }

        verify(&scene, &entities, has_system, resource);
    }
});
