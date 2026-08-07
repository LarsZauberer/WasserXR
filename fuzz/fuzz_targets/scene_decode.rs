#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

// Run: cargo fuzz run scene_decode
// Reproduce: cargo fuzz run scene_decode fuzz/artifacts/scene_decode/<artifact>
fuzz_target!(|data: &[u8]| {
    let mut scene = common::fixture_scene();
    if scene.deserialize(data).is_ok() {
        let encoded = scene
            .serialize()
            .expect("a successfully decoded scene must serialize");
        common::fixture_scene()
            .deserialize(&encoded)
            .expect("bytes emitted by Scene::serialize must deserialize");
    }
});
