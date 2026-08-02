fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        // Scene plugin discovery uses dlsym, so expose the linked fuzz fixtures.
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}
