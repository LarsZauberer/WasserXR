#![no_main]

mod common;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wasserxr::scene::{Scene, SceneError, component::ComponentError};

#[derive(Arbitrary, Debug)]
struct Input {
    field: u8,
    text: String,
}

const FIELDS: [&str; 7] = [
    "integer",
    "float",
    "vector",
    "character",
    "text",
    "boolean",
    "blob",
];

// Run: cargo fuzz run field_text
// Reproduce: cargo fuzz run field_text fuzz/artifacts/field_text/<artifact>
fuzz_target!(|input: Input| {
    let mut scene = Scene::new();
    let entity = scene.add_entity();
    scene
        .add_component(entity, "FuzzFields".to_owned())
        .expect("the statically linked fuzz component must be available");

    let field = FIELDS[input.field as usize % FIELDS.len()];
    let before = scene
        .render_field(entity, "FuzzFields", field)
        .expect("the fuzz component field must render");

    match scene.parse_field(entity, "FuzzFields", field, &input.text) {
        Ok(()) => {
            scene
                .render_field(entity, "FuzzFields", field)
                .expect("a parsed fuzz component field must render");
        }
        Err(SceneError::Component(
            ComponentError::FieldValueParsing | ComponentError::FieldNotMutable,
        )) => {
            assert_eq!(
                scene
                    .render_field(entity, "FuzzFields", field)
                    .expect("a rejected fuzz component field must still render"),
                before,
                "rejected field text must not mutate the field",
            );
        }
        Err(error) => panic!("unexpected field parsing failure: {error}"),
    }
});
