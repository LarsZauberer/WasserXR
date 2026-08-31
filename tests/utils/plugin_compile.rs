//! This module provides utility functions to easily build using `gcc` some test
//! plugins into shared object files in the OS's tmp directory (named with a
//! UUID) and return the filepath to them

use std::{path::Path, process::Command};

use uuid::Uuid;

pub fn compile_plugin(source: impl AsRef<Path>) -> &'static Path {
    let source = source.as_ref();
    let output = std::env::temp_dir().join(format!("wasserxr-test-plugin-{}.so", Uuid::new_v4()));
    let status = Command::new("gcc")
        .args(["-std=c11", "-fPIC", "-shared"])
        .arg(source)
        .arg("-o")
        .arg(&output)
        .status()
        .expect("gcc is required to compile test plugins");

    assert!(
        status.success(),
        "gcc failed to compile test plugin {}",
        source.display()
    );

    Box::leak(output.into_boxed_path())
}
