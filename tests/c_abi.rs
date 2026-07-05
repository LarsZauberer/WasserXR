use std::{
    env,
    ffi::CString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use wasserxr::bindings::{
    WXRSceneError,
    scene::{
        wxr_add_component, wxr_add_entity, wxr_add_system, wxr_create_scene, wxr_destroy_scene,
        wxr_load_plugin, wxr_query, wxr_tick,
    },
    wxr_error,
};

#[test]
fn c_staticlib_loads_c_shared_plugin() {
    if !cfg!(target_os = "linux") {
        eprintln!("skipping C ABI integration test outside Linux");
        return;
    }

    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping C ABI integration test because `gcc` is not available");
        return;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let work_dir = env::temp_dir()
        .join("wasserxr-c-abi-test")
        .join(std::process::id().to_string());
    let cargo_target_dir = work_dir.join("cargo-target");
    let build_dir = work_dir.join("build");
    let include_dir = build_dir.join("include");
    let header = include_dir.join("wasserxr.h");

    fs::create_dir_all(&include_dir).expect("failed to create C ABI test build directory");

    run(
        Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .current_dir(&work_dir)
            .arg("build")
            .arg("--manifest-path")
            .arg(manifest_dir.join("Cargo.toml"))
            .arg("--lib")
            .arg("--target-dir")
            .arg(&cargo_target_dir),
        "cargo build --lib",
    );

    let _staticlib = find_staticlib(&cargo_target_dir);

    run(
        Command::new("cbindgen")
            .current_dir(&work_dir)
            .arg(&manifest_dir)
            .arg("--config")
            .arg(manifest_dir.join("cbindgen.toml"))
            .arg("--output")
            .arg(&header),
        "cbindgen",
    );

    let plugin_source = build_dir.join("plugin.c");
    let plugin = build_dir.join("libwxr_c_abi_plugin.so");
    fs::copy(
        manifest_dir
            .join("tests")
            .join("fixtures")
            .join("c_abi_plugin.c"),
        &plugin_source,
    )
    .expect("failed to copy C plugin source");

    run(
        Command::new("gcc")
            .current_dir(&work_dir)
            .arg("-std=c11")
            .arg("-fPIC")
            .arg("-shared")
            .arg("-I")
            .arg(&include_dir)
            .arg(&plugin_source)
            .arg("-o")
            .arg(&plugin),
        "gcc plugin",
    );

    run_rust_harness(&plugin);
}

fn run(command: &mut Command, label: &str) {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("failed to run {label}: {error}"));

    assert!(
        output.status.success(),
        "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn find_staticlib(target_dir: &Path) -> PathBuf {
    let mut pending = vec![target_dir.to_path_buf()];

    while let Some(dir) = pending.pop() {
        let entries = fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));

        for entry in entries {
            let path = entry.expect("failed to read target directory entry").path();
            if path.file_name().is_some_and(|name| name == "libwasserxr.a") {
                return path;
            }
            if path.is_dir() {
                pending.push(path);
            }
        }
    }

    panic!("expected static library under {}", target_dir.display());
}

fn run_rust_harness(plugin: &PathBuf) {
    let component = CString::new("abi_counter").unwrap();
    let system = CString::new("abi_counter_system").unwrap();
    let field = CString::new("value").unwrap();
    let plugin = CString::new(plugin.to_string_lossy().as_bytes()).unwrap();

    let scene = wxr_create_scene();
    assert!(!scene.is_null());

    unsafe {
        assert_eq!(wxr_load_plugin(scene, plugin.as_ptr()), 0);

        let entity = wxr_add_entity(scene);
        assert_eq!(wxr_error(), WXRSceneError::NoError);
        assert_eq!(wxr_add_component(scene, entity, component.as_ptr()), 0);

        let value = wxr_query(scene, entity, component.as_ptr(), field.as_ptr()).cast::<i32>();
        assert!(!value.is_null());
        assert_eq!(*value, 0);

        assert_eq!(wxr_add_system(scene, system.as_ptr(), 0), 0);
        assert_eq!(wxr_tick(scene), 1);

        let value = wxr_query(scene, entity, component.as_ptr(), field.as_ptr()).cast::<i32>();
        assert!(!value.is_null());
        assert_eq!(*value, 1);

        wxr_destroy_scene(scene);
    }
}
