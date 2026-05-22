use std::{env, fs, path::PathBuf};

fn main() {
    // Cargo sets this env var to the directory that contains Cargo.toml.
    // Build scripts often use it as the stable "project root" path.
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header_path = crate_dir.join("include").join("wasserxr.h");

    // These lines tell Cargo when to rerun this build script.
    // Without them, Cargo may rerun too often or miss changes that affect the generated header.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/scene.rs");
    println!("cargo:rerun-if-changed=src/bindings");

    // Linux needs exported Rust symbols so hot-reloaded modules can find engine functions.
    // The string after `cargo:` is an instruction that Cargo forwards to rustc/linker.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }

    // cbindgen writes one C header for the Rust API. Make sure include/ exists first.
    fs::create_dir_all(header_path.parent().unwrap()).expect("failed to create include directory");

    // Read cbindgen.toml plus Rust source, generate declarations, then write include/wasserxr.h.
    cbindgen::generate(&crate_dir)
        .expect("failed to generate C header")
        .write_to_file(header_path);
}
