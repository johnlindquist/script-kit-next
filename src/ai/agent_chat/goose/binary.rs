//! Locates the goose ACP sidecar that runs Script Kit's AI sessions.
//!
//! Resolution order, first match wins:
//! 1. `SCRIPT_KIT_GOOSE_BINARY` — an explicit override is authoritative even
//!    when it points at a broken file, so a bad override surfaces instead of
//!    silently falling back to another engine.
//! 2. `Contents/MacOS/goose` beside the app executable (release bundles).
//! 3. Debug builds only: the repo-managed sidecar at `target/goose-sidecar/goose`
//!    produced by `scripts/prepare-goose-sidecar.sh`.

use std::path::{Path, PathBuf};

pub const SCRIPT_KIT_GOOSE_BINARY_ENV: &str = "SCRIPT_KIT_GOOSE_BINARY";
pub const BUNDLED_GOOSE_BINARY_NAME: &str = "goose";

pub fn default_goose_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(SCRIPT_KIT_GOOSE_BINARY_ENV) {
        let path = path.to_string_lossy();
        let path = path.trim();
        if !path.is_empty() {
            return Some(expand_tilde_path(path));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(path) = existing_bundled_goose_binary_for_exe(&exe) {
            return Some(path);
        }
    }

    dev_sidecar_goose_binary()
}

pub fn bundled_goose_binary_candidate_for_exe(exe: &Path) -> Option<PathBuf> {
    is_macos_app_executable(exe)
        .then(|| exe.parent().map(|dir| dir.join(BUNDLED_GOOSE_BINARY_NAME)))
        .flatten()
}

pub fn existing_bundled_goose_binary_for_exe(exe: &Path) -> Option<PathBuf> {
    let candidate = bundled_goose_binary_candidate_for_exe(exe)?;
    is_executable_file(&candidate).then_some(candidate)
}

#[cfg(debug_assertions)]
pub fn dev_sidecar_goose_binary() -> Option<PathBuf> {
    dev_sidecar_goose_binary_in_repo(Path::new(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(not(debug_assertions))]
pub fn dev_sidecar_goose_binary() -> Option<PathBuf> {
    None
}

#[cfg(debug_assertions)]
pub fn dev_sidecar_goose_binary_in_repo(repo_root: &Path) -> Option<PathBuf> {
    let candidate = repo_root.join("target").join("goose-sidecar").join("goose");
    is_executable_file(&candidate).then_some(candidate)
}

pub fn expand_tilde_path(value: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(value).as_ref())
}

fn is_macos_app_executable(exe: &Path) -> bool {
    exe.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        == Some("MacOS")
        && exe
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            == Some("Contents")
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_candidate_sits_beside_the_app_executable() {
        let exe = Path::new("/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui");
        assert_eq!(
            bundled_goose_binary_candidate_for_exe(exe),
            Some(PathBuf::from(
                "/Applications/Script Kit.app/Contents/MacOS/goose"
            ))
        );
    }

    #[test]
    fn non_bundle_executables_have_no_bundled_candidate() {
        let exe = Path::new("/tmp/target/debug/script-kit-gpui");
        assert_eq!(bundled_goose_binary_candidate_for_exe(exe), None);
    }

    #[cfg(unix)]
    #[test]
    fn bundled_binary_must_exist_and_be_executable() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().expect("tempdir");
        let macos = temp
            .path()
            .join("Script Kit.app")
            .join("Contents")
            .join("MacOS");
        std::fs::create_dir_all(&macos).expect("macos dir");
        let exe = macos.join("script-kit-gpui");
        let goose = macos.join("goose");
        std::fs::write(&exe, b"app").expect("app binary");

        assert_eq!(existing_bundled_goose_binary_for_exe(&exe), None);

        std::fs::write(&goose, b"goose").expect("goose binary");
        assert_eq!(existing_bundled_goose_binary_for_exe(&exe), None);

        let mut permissions = std::fs::metadata(&goose).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&goose, permissions).unwrap();
        assert_eq!(existing_bundled_goose_binary_for_exe(&exe), Some(goose));
    }
}
