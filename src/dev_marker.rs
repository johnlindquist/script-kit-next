//! Dev-only marker capture used by `./dev.sh`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use gpui::App;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const EXPLANATION_HEADING: &str = "## Explanation";
const EXPLANATION_PLACEHOLDER: &str =
    "Write the short context here. Autosave will append it to the structured logs.";
const EXPLANATION_LOG_IDLE_MS: u64 = 2_000;

#[derive(Debug, Clone)]
struct PendingExplanationLog {
    note_id: String,
    explanation: String,
    explanation_hash: String,
    generation: u64,
}

#[derive(Debug, Default)]
struct ExplanationLogState {
    pending_by_marker_id: HashMap<String, PendingExplanationLog>,
    last_logged_hash_by_marker_id: HashMap<String, String>,
    next_generation: u64,
}

static EXPLANATION_LOG_STATE: OnceLock<Mutex<ExplanationLogState>> = OnceLock::new();

fn explanation_log_state() -> &'static Mutex<ExplanationLogState> {
    EXPLANATION_LOG_STATE.get_or_init(|| Mutex::new(ExplanationLogState::default()))
}

fn marker_dir() -> PathBuf {
    crate::logging::session_log_path()
        .parent()
        .map(|path| path.join("dev-markers"))
        .unwrap_or_else(|| std::env::temp_dir().join("script-kit-dev-markers"))
}

fn hash_text(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{digest:x}")
}

fn capture_marker_screenshot_bytes(
    marker_id: &str,
) -> anyhow::Result<(Vec<u8>, u32, u32, &'static str)> {
    let target = crate::protocol::AutomationWindowTarget::Focused;
    match crate::platform::capture_targeted_screenshot(Some(&target), false) {
        Ok((png, width, height)) => return Ok((png, width, height, "focused_automation_window")),
        Err(error) => {
            tracing::warn!(
                event_type = "dev_marker_focused_screenshot_failed",
                marker_id = %marker_id,
                error = %error,
                "Dev marker focused-window screenshot failed; falling back to active display"
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        let (png, width, height) = crate::platform::capture_active_display_screenshot_sck()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        Ok((png, width, height, "active_display"))
    }

    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("active-display dev marker screenshots are only supported on macOS")
    }
}

fn capture_marker_screenshot(marker_id: &str) -> anyhow::Result<PathBuf> {
    let dir = marker_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{marker_id}.png"));
    let (png, width, height, capture_method) = capture_marker_screenshot_bytes(marker_id)?;
    fs::write(&path, png)?;
    tracing::info!(
        event_type = "dev_marker_screenshot",
        marker_id = %marker_id,
        screenshot_path = %path.display(),
        width = width,
        height = height,
        capture_method = capture_method,
        "Dev marker screenshot captured"
    );
    Ok(path)
}

fn marker_note_content(
    marker_id: &str,
    created_at: &str,
    screenshot_path: Option<&PathBuf>,
) -> String {
    let screenshot = screenshot_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    format!(
        concat!(
            "---\n",
            "tags: [dev-marker]\n",
            "dev_marker_id: {}\n",
            "dev_marker_created_at: {}\n",
            "dev_marker_screenshot: \"{}\"\n",
            "---\n",
            "# Dev Marker {}\n\n",
            "- created_at: {}\n",
            "- screenshot: {}\n",
            "- session_log: {}\n\n",
            "{}\n\n",
            "{}\n",
        ),
        marker_id,
        created_at,
        screenshot,
        marker_id,
        created_at,
        screenshot,
        crate::logging::session_log_path().display(),
        EXPLANATION_HEADING,
        EXPLANATION_PLACEHOLDER
    )
}

/// Handle the dev-marker hotkey on the GPUI main thread.
pub fn handle_hotkey(cx: &mut App) {
    let marker_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let screenshot_path = match capture_marker_screenshot(&marker_id) {
        Ok(path) => Some(path),
        Err(error) => {
            tracing::warn!(
                event_type = "dev_marker_screenshot_failed",
                marker_id = %marker_id,
                error = %error,
                "Dev marker screenshot capture failed"
            );
            None
        }
    };

    tracing::info!(
        event_type = "dev_marker",
        marker_id = %marker_id,
        created_at = %created_at,
        screenshot_path = %screenshot_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unavailable".to_string()),
        session_log = %crate::logging::session_log_path().display(),
        "Dev marker created"
    );

    let content = marker_note_content(&marker_id, &created_at, screenshot_path.as_ref());
    if let Err(error) = crate::notes::save_note_with_content(cx, content) {
        tracing::warn!(
            event_type = "dev_marker_note_failed",
            marker_id = %marker_id,
            error = %error,
            "Dev marker note open failed"
        );
    }
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        let Some((line_key, value)) = line.split_once(':') else {
            continue;
        };
        if line_key.trim() == key {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn explanation_from_marker_note(content: &str) -> Option<String> {
    let (_, tail) = content.split_once(EXPLANATION_HEADING)?;
    let mut explanation = tail.trim();
    if let Some(rest) = explanation.strip_prefix(EXPLANATION_PLACEHOLDER) {
        explanation = rest.trim();
    }
    if explanation.is_empty() {
        return None;
    }
    Some(explanation.to_string())
}

fn emit_pending_explanation_if_still_current(marker_id: &str, generation: u64) {
    let pending = {
        let Ok(mut state) = explanation_log_state().lock() else {
            return;
        };
        let Some(pending) = state.pending_by_marker_id.get(marker_id).cloned() else {
            return;
        };
        if pending.generation != generation {
            return;
        }
        if state
            .last_logged_hash_by_marker_id
            .get(marker_id)
            .is_some_and(|hash| hash == &pending.explanation_hash)
        {
            return;
        }
        state
            .last_logged_hash_by_marker_id
            .insert(marker_id.to_string(), pending.explanation_hash.clone());
        pending
    };

    tracing::info!(
        event_type = "dev_marker_explanation",
        marker_id = %marker_id,
        note_id = %pending.note_id,
        explanation = %pending.explanation,
        explanation_len = pending.explanation.chars().count(),
        explanation_hash = %pending.explanation_hash,
        idle_ms = EXPLANATION_LOG_IDLE_MS,
        "Dev marker explanation saved"
    );
}

/// Append a structured log when a dev-marker note receives a real explanation.
///
/// Notes autosave while the user types. To avoid one full-text log line per
/// autosaved prefix, this schedules an idle-edge log and only emits if the
/// marker explanation has not changed for `EXPLANATION_LOG_IDLE_MS`.
pub(crate) fn log_marker_note_explanation_if_ready(note_id: &crate::notes::NoteId, content: &str) {
    let Some(marker_id) = frontmatter_value(content, "dev_marker_id") else {
        return;
    };
    let Some(explanation) = explanation_from_marker_note(content) else {
        return;
    };
    let explanation_hash = hash_text(&explanation);
    let note_id = note_id.as_str();
    let generation = {
        let Ok(mut state) = explanation_log_state().lock() else {
            return;
        };
        if state
            .last_logged_hash_by_marker_id
            .get(&marker_id)
            .is_some_and(|hash| hash == &explanation_hash)
        {
            return;
        }
        state.next_generation = state.next_generation.wrapping_add(1);
        let generation = state.next_generation;
        state.pending_by_marker_id.insert(
            marker_id.clone(),
            PendingExplanationLog {
                note_id,
                explanation,
                explanation_hash,
                generation,
            },
        );
        generation
    };

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(EXPLANATION_LOG_IDLE_MS));
        emit_pending_explanation_if_still_current(&marker_id, generation);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_value_extracts_dev_marker_id() {
        let content = "---\ntags: [dev-marker]\ndev_marker_id: abc-123\n---\nbody";

        assert_eq!(
            frontmatter_value(content, "dev_marker_id"),
            Some("abc-123".to_string())
        );
    }

    #[test]
    fn explanation_from_marker_note_ignores_placeholder() {
        let content = format!("{EXPLANATION_HEADING}\n\n{EXPLANATION_PLACEHOLDER}\n");

        assert_eq!(explanation_from_marker_note(&content), None);
    }

    #[test]
    fn explanation_from_marker_note_extracts_real_context() {
        let content = format!("{EXPLANATION_HEADING}\n\nButton disappeared after filter reset\n");

        assert_eq!(
            explanation_from_marker_note(&content),
            Some("Button disappeared after filter reset".to_string())
        );
    }

    #[test]
    fn explanation_from_marker_note_strips_placeholder_prefix() {
        let content = format!(
            "{EXPLANATION_HEADING}\n\n{EXPLANATION_PLACEHOLDER} Button disappeared after filter reset\n"
        );

        assert_eq!(
            explanation_from_marker_note(&content),
            Some("Button disappeared after filter reset".to_string())
        );
    }
}
