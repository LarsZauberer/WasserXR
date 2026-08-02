#![no_main]

use libfuzzer_sys::fuzz_target;
use wasserxr::scene::Scene;

// Run: cargo fuzz run scene_decode
// Reproduce: cargo fuzz run scene_decode fuzz/artifacts/scene_decode/<artifact>
fuzz_target!(|data: &[u8]| {
    let mut scene = Scene::new();
    if scene.deserialize(data).is_ok() {
        let encoded = scene
            .serialize()
            .expect("a successfully decoded scene must serialize");
        Scene::new()
            .deserialize(&encoded)
            .expect("bytes emitted by Scene::serialize must deserialize");
    }
});
