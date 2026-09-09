use std::{sync::Arc, thread};

use wasserxr::scene::Scene;

#[test]
fn scene_can_be_shared_across_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Scene>();

    let scene = Arc::new(Scene::new());
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let scene = Arc::clone(&scene);
            thread::spawn(move || scene.add_entity())
        })
        .collect();
    let entities: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().expect("entity thread panicked"))
        .collect();

    assert_eq!(scene.get_entities().len(), entities.len());
}
