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

    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let exe_path = exe_dir.join(path);
            if exe_path.exists() {
                return exe_path.canonicalize().ok();
            }
        }
    }

    let cwd_path = env::current_dir().ok()?.join(path);
    if cwd_path.exists() {
        return cwd_path.canonicalize().ok();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(unique_name(name))
    }

    fn unique_name(name: &str) -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("wasserxr-{name}-{stamp}")
    }

    #[test]
    fn absolute_path_must_exist() {
        let dir = temp_path("absolute");
        fs::create_dir(&dir).unwrap();

        assert_eq!(get_asset_path(dir.to_str().unwrap()), Some(dir.clone()));
        assert_eq!(get_asset_path(dir.join("missing").to_str().unwrap()), None);

        fs::remove_dir(&dir).unwrap();
    }

    #[test]
    fn relative_path_falls_back_to_current_dir() {
        assert_eq!(
            get_asset_path("Cargo.toml"),
            Some(
                env::current_dir()
                    .unwrap()
                    .join("Cargo.toml")
                    .canonicalize()
                    .unwrap()
            )
        );
    }

    #[test]
    fn relative_path_prefers_binary_dir() {
        let name = unique_name("priority");
        let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let exe_file = exe_dir.join(&name);
        let cwd_file = env::current_dir().unwrap().join(&name);

        fs::write(&exe_file, "exe").unwrap();
        fs::write(&cwd_file, "cwd").unwrap();

        assert_eq!(
            get_asset_path(&name),
            Some(exe_file.canonicalize().unwrap())
        );

        fs::remove_file(exe_file).unwrap();
        fs::remove_file(cwd_file).unwrap();
    }
}
