//! Helpers for discovering locally installed kit directories.

use std::fs;
use std::path::{Path, PathBuf};

/// Discover installed kits under `~/.scriptkit/kits/`.
///
/// Each direct subdirectory is treated as an installed kit.
pub fn discover_installed_kits() -> Vec<PathBuf> {
    let Some(home_dir) = dirs::home_dir() else {
        return Vec::new();
    };

    let kits_root = home_dir.join(".scriptkit").join("kits");
    discover_installed_kits_at(&kits_root)
}

/// Return the scripts directory for a specific kit.
pub fn kit_scripts_dir(kit_path: &Path) -> PathBuf {
    kit_path.join("scripts")
}

/// Return the scriptlets directory for a specific kit.
pub fn kit_scriptlets_dir(kit_path: &Path) -> PathBuf {
    kit_path.join("scriptlets")
}

fn discover_installed_kits_at(kits_root: &Path) -> Vec<PathBuf> {
    let entries = match fs::read_dir(kits_root) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut kits = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    kits.sort();
    kits
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{discover_installed_kits_at, kit_scriptlets_dir, kit_scripts_dir};

    #[test]
    fn test_discover_installed_kits_at_returns_empty_when_root_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let missing_root = temp_dir.path().join("kits");

        let kits = discover_installed_kits_at(&missing_root);
        assert!(kits.is_empty());
    }

    #[test]
    fn test_discover_installed_kits_at_returns_only_subdirectories() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let kits_root = temp_dir.path().join("kits");
        fs::create_dir_all(kits_root.join("alpha")).expect("alpha dir should create");
        fs::create_dir_all(kits_root.join("zeta")).expect("zeta dir should create");
        fs::write(kits_root.join("README.md"), "not a kit").expect("file should write");

        let kits = discover_installed_kits_at(&kits_root);

        assert_eq!(kits, vec![kits_root.join("alpha"), kits_root.join("zeta")]);
    }

    #[test]
    fn test_kit_scripts_dir_appends_scripts_suffix() {
        let kit_path = PathBuf::from("/tmp/example-kit");
        assert_eq!(
            kit_scripts_dir(&kit_path),
            PathBuf::from("/tmp/example-kit/scripts")
        );
    }

    #[test]
    fn test_kit_scriptlets_dir_appends_scriptlets_suffix() {
        let kit_path = PathBuf::from("/tmp/example-kit");
        assert_eq!(
            kit_scriptlets_dir(&kit_path),
            PathBuf::from("/tmp/example-kit/scriptlets")
        );
    }
}
