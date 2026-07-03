use std::{env, fs, path::PathBuf};

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header_path = crate_dir.join("include").join("wasserxr.h");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/error.rs");
    println!("cargo:rerun-if-changed=src/scene");
    println!("cargo:rerun-if-changed=src/utils");
    println!("cargo:rerun-if-changed=src/bindings");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }

    fs::create_dir_all(header_path.parent().unwrap()).expect("failed to create include directory");

    cbindgen::generate(&crate_dir)
        .expect("failed to generate C header")
        .write_to_file(header_path);
}
