//! One-shot disclosure for the silent clipboard-sediment auto-keep.
//!
//! Sediment deliberately never opens UI at copy time (ADR 0004). That makes
//! the behavior invisible: kept URLs and promoted re-copies just appear on
//! the Day Page with no explanation. This module lets the keep paths record
//! (from any thread, no UI context) that the behavior has actually fired, so
//! the next main-window show can disclose it once via a toast.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

const DISCLOSURE_MARKER: &str = "sediment-disclosure.json";
const ACTIVITY_MARKER: &str = "sediment-activity.json";

/// In-process fast path so the clipboard monitor stats the filesystem at most
/// once per run.
static ACTIVITY_RECORDED: AtomicBool = AtomicBool::new(false);

pub fn disclosure_marker_path() -> PathBuf {
    crate::setup::get_kit_path().join(DISCLOSURE_MARKER)
}

pub fn activity_marker_path() -> PathBuf {
    crate::setup::get_kit_path().join(ACTIVITY_MARKER)
}

pub fn already_shown() -> bool {
    disclosure_marker_path().exists()
}

pub fn mark_shown() {
    write_marker(disclosure_marker_path());
}

/// True once any silent auto-keep (URL keep or re-copy promotion) has fired.
pub fn activity_recorded() -> bool {
    ACTIVITY_RECORDED.load(Ordering::Relaxed) || activity_marker_path().exists()
}

/// Called from the sediment keep paths (clipboard monitor thread, no UI).
pub fn record_activity() {
    if ACTIVITY_RECORDED.swap(true, Ordering::Relaxed) {
        return;
    }
    let path = activity_marker_path();
    if path.exists() {
        return;
    }
    write_marker(path);
}

#[cfg(test)]
pub fn reset_in_process_activity_flag() {
    ACTIVITY_RECORDED.store(false, Ordering::Relaxed);
}

fn write_marker(path: PathBuf) {
    let state = serde_json::json!({
        "schemaVersion": 1,
        "at": chrono::Utc::now().to_rfc3339(),
    });
    if let Ok(content) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::setup::SK_PATH_ENV;

    fn sk_path_test_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::test_utils::SK_PATH_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn sediment_disclosure_markers_are_once_only() {
        let _lock = sk_path_test_lock();
        let temp = tempfile::tempdir().expect("tempdir");
        let kit = temp.path().join("kit");
        std::fs::create_dir_all(&kit).expect("kit dir");
        std::env::set_var(SK_PATH_ENV, kit.to_string_lossy().as_ref());
        reset_in_process_activity_flag();

        assert!(!activity_recorded());
        assert!(!already_shown());

        record_activity();
        assert!(activity_recorded());
        assert!(activity_marker_path().exists());

        mark_shown();
        assert!(already_shown());
        assert!(disclosure_marker_path().exists());

        reset_in_process_activity_flag();
        std::env::remove_var(SK_PATH_ENV);
    }
}
