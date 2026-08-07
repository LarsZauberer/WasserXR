use std::env;

fn main() {
    let header_config = include_str!("cbindgen.toml");
    for (name, value) in [
        ("MAJOR", env!("CARGO_PKG_VERSION_MAJOR")),
        ("MINOR", env!("CARGO_PKG_VERSION_MINOR")),
        ("PATCH", env!("CARGO_PKG_VERSION_PATCH")),
    ] {
        assert!(
            header_config.contains(&format!("#define WXR_VERSION_{name} {value}")),
            "cbindgen.toml WXR_VERSION_{name} must match Cargo.toml"
        );
    }

    // Dynamically loaded C callbacks resolve the host's `wxr_*` bindings.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}
