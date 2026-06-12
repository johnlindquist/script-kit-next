//! Marker for the retired tap-to-dismiss habit (gesture grammar T8).

use std::path::PathBuf;

const TAP_DISMISS_HINT_MARKER: &str = "gesture-hint-tap-dismiss.json";

pub fn marker_path() -> PathBuf {
    crate::setup::get_kit_path().join(TAP_DISMISS_HINT_MARKER)
}

pub fn already_shown() -> bool {
    marker_path().exists()
}

pub fn mark_shown() {
    let path = marker_path();
    let state = serde_json::json!({
        "schemaVersion": 1,
        "shownAt": chrono::Utc::now().to_rfc3339(),
    });
    if let Ok(content) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use super::{already_shown, mark_shown, marker_path};
    use crate::setup::{get_kit_path, SK_PATH_ENV};

    fn sk_path_test_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn tap_dismiss_hint_marker_is_once_only() {
        let _lock = sk_path_test_lock();
        let temp = tempfile::tempdir().expect("tempdir");
        let kit = temp.path().join("kit");
        std::fs::create_dir_all(&kit).expect("kit dir");
        std::env::set_var(SK_PATH_ENV, kit.to_string_lossy().as_ref());

        assert!(!already_shown());
        mark_shown();
        assert!(already_shown());
        assert_eq!(
            marker_path(),
            PathBuf::from(get_kit_path()).join("gesture-hint-tap-dismiss.json")
        );

        std::env::remove_var(SK_PATH_ENV);
    }
}
