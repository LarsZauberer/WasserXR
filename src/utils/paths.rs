use std::{
    env,
    path::{Path, PathBuf},
};

/// Resolves an asset path to an existing absolute path.
///
/// Absolute inputs must exist. Relative inputs are searched next to the current
/// executable first and then in the current working directory.
///
/// # Examples
///
/// ```no_run
/// let cargo_toml = wasserxr::utils::paths::get_asset_path("Cargo.toml");
/// assert!(cargo_toml.is_some());
/// ```
pub fn get_asset_path(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);

    if path.is_absolute() {
        return path.exists().then(|| path.to_path_buf());
    }

    if let Ok(exe) = env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let exe_path = exe_dir.join(path);
        if exe_path.exists() {
            return exe_path.canonicalize().ok();
        }
    }

    let cwd_path = env::current_dir().ok()?.join(path);
    if cwd_path.exists() {
        return cwd_path.canonicalize().ok();
    }

    None
}
