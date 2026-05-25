use std::path::{Path, PathBuf};

pub const SCRIPT_KIT_PI_BINARY_ENV: &str = "SCRIPT_KIT_PI_BINARY";
pub const BUNDLED_PI_BINARY_NAME: &str = "pi";

pub fn default_pi_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(SCRIPT_KIT_PI_BINARY_ENV) {
        let path = path.to_string_lossy();
        if let Some(path) = clean_path(path.as_ref()) {
            return Some(expand_tilde_path(path));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(path) = existing_bundled_pi_binary_for_exe(&exe) {
            return Some(path);
        }
    }

    dev_pi_binary_for_home(dirs::home_dir().as_deref())
}

pub fn bundled_pi_binary_candidate_for_exe(exe: &Path) -> Option<PathBuf> {
    is_macos_app_executable(exe)
        .then(|| {
            exe.parent()
                .map(|parent| parent.join(BUNDLED_PI_BINARY_NAME))
        })
        .flatten()
}

pub fn existing_bundled_pi_binary_for_exe(exe: &Path) -> Option<PathBuf> {
    let candidate = bundled_pi_binary_candidate_for_exe(exe)?;
    is_executable_file(&candidate).then_some(candidate)
}

#[cfg(debug_assertions)]
pub fn dev_pi_binary_for_home(home: Option<&Path>) -> Option<PathBuf> {
    let home = home?;
    let pi_repo = home.join("dev").join("pi_agent_rust");
    let release = pi_repo.join("target").join("release").join("pi");
    if is_executable_file(&release) {
        return Some(release);
    }

    let debug = pi_repo.join("target").join("debug").join("pi");
    is_executable_file(&debug).then_some(debug)
}

#[cfg(not(debug_assertions))]
pub fn dev_pi_binary_for_home(_home: Option<&Path>) -> Option<PathBuf> {
    None
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
        return path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn clean_path(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_candidate_is_next_to_macos_executable() {
        let exe = Path::new("/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui");

        assert_eq!(
            bundled_pi_binary_candidate_for_exe(exe),
            Some(PathBuf::from(
                "/Applications/Script Kit.app/Contents/MacOS/pi"
            ))
        );
    }

    #[test]
    fn bundled_resolution_requires_executable_sidecar() {
        let temp = tempfile::tempdir().expect("temp dir");
        let macos = temp
            .path()
            .join("Script Kit.app")
            .join("Contents")
            .join("MacOS");
        std::fs::create_dir_all(&macos).expect("macos dir");
        let exe = macos.join("script-kit-gpui");
        let pi = macos.join("pi");
        std::fs::write(&exe, b"app").expect("app binary");
        std::fs::write(&pi, b"pi").expect("pi binary");

        assert_eq!(existing_bundled_pi_binary_for_exe(&exe), None);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&pi).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&pi, permissions).unwrap();
        }

        assert_eq!(existing_bundled_pi_binary_for_exe(&exe), Some(pi));
    }

    #[test]
    fn non_bundle_executable_does_not_claim_bundled_sidecar() {
        let exe = Path::new("/tmp/script-kit-gpui");

        assert_eq!(bundled_pi_binary_candidate_for_exe(exe), None);
    }

    #[test]
    fn dev_fallback_is_absent_when_local_pi_binary_is_not_executable() {
        let home = Path::new("/Users/test");

        assert_eq!(dev_pi_binary_for_home(Some(home)), None);
    }
}
