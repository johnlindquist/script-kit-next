//! Where notes live. `notesFolder` in config.ts names the folder; inside it `days/` holds a
//! page per day and `notes/` the named notes, as plain Markdown that Obsidian, Foam and other
//! Markdown apps open as they are. The folder is read once when Script Kit starts.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static CONFIGURED_FOLDER: OnceLock<PathBuf> = OnceLock::new();
static ANNOUNCED_PENDING_FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Startup: use the folder config.ts's `notesFolder` names for the rest of the run. Only the
/// first call counts; with no setting, notes stay in the default folder.
pub fn use_configured_notes_folder(setting: Option<&str>) {
    if setting.is_some_and(|setting| !setting.trim().is_empty()) {
        let _ = CONFIGURED_FOLDER.set(resolve_notes_folder(setting));
    }
}

/// The folder notes are read from and written to while Script Kit runs.
pub fn notes_folder() -> PathBuf {
    CONFIGURED_FOLDER
        .get()
        .cloned()
        .unwrap_or_else(default_notes_folder)
}

/// `~/.scriptkit/brain`, where notes live when `notesFolder` is not set.
pub fn default_notes_folder() -> PathBuf {
    crate::setup::get_kit_path().join("brain")
}

/// The folder a `notesFolder` setting names: `~` expands to home, and a relative path is read
/// from `~/.scriptkit`. An empty or missing setting is the default folder.
pub fn resolve_notes_folder(setting: Option<&str>) -> PathBuf {
    let Some(setting) = setting.map(str::trim).filter(|setting| !setting.is_empty()) else {
        return default_notes_folder();
    };
    let expanded = PathBuf::from(shellexpand::tilde(setting).as_ref());
    let path = if expanded.is_absolute() {
        expanded
    } else {
        crate::setup::get_kit_path().join(expanded)
    };
    std::fs::canonicalize(&path).unwrap_or(path)
}

/// When config.ts now names a different folder than the one in use, the folder it names, once
/// per new value: the change takes hold the next time Script Kit starts.
pub fn newly_configured_folder(setting: Option<&str>) -> Option<PathBuf> {
    let configured = resolve_notes_folder(setting);
    let mut announced = ANNOUNCED_PENDING_FOLDER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if same_folder(&configured, &notes_folder()) {
        *announced = None;
        return None;
    }
    if announced.as_deref() == Some(configured.as_path()) {
        return None;
    }
    *announced = Some(configured.clone());
    Some(configured)
}
/// A folder as people write it: the home folder as `~`.
pub fn display_path(path: &Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf)) {
        Some(relative) if relative.as_os_str().is_empty() => "~".to_string(),
        Some(relative) => format!("~/{}", relative.display()),
        None => path.display().to_string(),
    }
}

fn same_folder(a: &Path, b: &Path) -> bool {
    a == b
        || std::fs::canonicalize(a)
            .ok()
            .zip(std::fs::canonicalize(b).ok())
            .is_some_and(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_setting_names_a_folder_from_home_or_from_the_kit() {
        let home = dirs::home_dir().expect("home");
        assert_eq!(resolve_notes_folder(None), default_notes_folder());
        assert_eq!(resolve_notes_folder(Some("  ")), default_notes_folder());
        assert_eq!(
            resolve_notes_folder(Some("~/script-kit-notes-folder-test-missing")),
            home.join("script-kit-notes-folder-test-missing")
        );
        assert_eq!(
            resolve_notes_folder(Some("vault-test-missing")),
            crate::setup::get_kit_path().join("vault-test-missing")
        );
    }

    #[test]
    fn a_new_folder_is_named_once_until_it_changes_again() {
        let elsewhere = std::env::temp_dir().join("script-kit-notes-folder-test-a");
        let elsewhere = elsewhere.to_string_lossy().to_string();
        let other = std::env::temp_dir().join("script-kit-notes-folder-test-b");
        let other = other.to_string_lossy().to_string();
        assert!(newly_configured_folder(Some(&elsewhere)).is_some());
        assert!(newly_configured_folder(Some(&elsewhere)).is_none());
        assert!(newly_configured_folder(Some(&other)).is_some());
        assert!(
            newly_configured_folder(None).is_none(),
            "back to the folder in use"
        );
        assert!(newly_configured_folder(Some(&other)).is_some());
    }
}
