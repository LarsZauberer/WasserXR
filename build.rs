use std::{env, path::PathBuf};

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    cbindgen::generate(&crate_dir)
        .expect("failed to generate C bindings")
        .write_to_file(crate_dir.join("include/wasserxr.h"));

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}
