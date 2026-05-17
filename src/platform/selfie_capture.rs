/// File paths and capture metadata produced by the Script Kit Selfie command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptKitSelfieReceipt {
    pub schema_version: u8,
    pub command_id: String,
    pub receipt_id: String,
    pub created_at: String,
    pub state: String,
    pub shortcut: String,
    pub capture_method: String,
    pub png_path: String,
    pub receipt_path: String,
    pub window_bounds: ScriptKitSelfieBounds,
    pub monitor_bounds: ScriptKitSelfieBounds,
    pub crop_bounds: ScriptKitSelfieBounds,
    pub image_width: u32,
    pub image_height: u32,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptKitSelfieBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

const SCRIPT_KIT_SELFIE_COMMAND_ID: &str = "builtin/script-kit-selfie";
const SCRIPT_KIT_SELFIE_SHORTCUT: &str = "cmd+alt+1";
const SCRIPT_KIT_SELFIE_MARGIN: i32 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptKitSelfieWindowKind {
    Dictation,
    Notes,
    MainOrOther,
}

impl ScriptKitSelfieWindowKind {
    fn priority(self) -> i32 {
        match self {
            Self::Dictation => 3,
            Self::Notes => 2,
            Self::MainOrOther => 1,
        }
    }

    fn state_label(self, fallback: &str) -> String {
        match self {
            Self::Dictation => "Dictation".to_string(),
            Self::Notes => "Notes".to_string(),
            Self::MainOrOther => fallback.to_string(),
        }
    }

    fn capture_method_suffix(self) -> &'static str {
        match self {
            Self::Dictation => "dictation",
            Self::Notes => "notes",
            Self::MainOrOther => "main",
        }
    }
}

#[derive(Debug, Clone)]
struct ScriptKitSelfieCandidateSnapshot {
    title: String,
    app_name: String,
    focused: bool,
    width: i32,
    height: i32,
}

fn classify_script_kit_selfie_candidate(
    candidate: &ScriptKitSelfieCandidateSnapshot,
    dictation_open: bool,
) -> ScriptKitSelfieWindowKind {
    let title = candidate.title.to_lowercase();
    let app_name = candidate.app_name.to_lowercase();
    let title_or_app_mentions_dictation =
        title.contains("dictation") || app_name.contains("dictation");
    let looks_like_titleless_dictation_overlay =
        dictation_open && candidate.height <= 220 && candidate.width >= 240 && candidate.width <= 900;
    if title_or_app_mentions_dictation || looks_like_titleless_dictation_overlay {
        ScriptKitSelfieWindowKind::Dictation
    } else if title.contains("notes") || app_name.contains("notes") {
        ScriptKitSelfieWindowKind::Notes
    } else {
        ScriptKitSelfieWindowKind::MainOrOther
    }
}

fn select_script_kit_selfie_candidate_index(
    candidates: &[ScriptKitSelfieCandidateSnapshot],
    dictation_open: bool,
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .max_by_key(|(_, candidate)| {
            let kind = classify_script_kit_selfie_candidate(candidate, dictation_open);
            let area = candidate.width as i64 * candidate.height as i64;
            (kind.priority(), candidate.focused, area)
        })
        .map(|(index, _)| index)
}

pub fn script_kit_selfie_output_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".scriptkit")
        .join("screenshots")
        .join("selfies")
}

pub fn slugify_script_kit_selfie_state(state: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in state.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "unknown-state".to_string()
    } else {
        slug.to_string()
    }
}

pub fn capture_script_kit_selfie(state: &str) -> anyhow::Result<ScriptKitSelfieReceipt> {
    #[cfg(target_os = "macos")]
    {
        capture_script_kit_selfie_macos(state)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = state;
        anyhow::bail!("Script Kit Selfie is only supported on macOS");
    }
}

#[cfg(target_os = "macos")]
fn capture_script_kit_selfie_macos(state: &str) -> anyhow::Result<ScriptKitSelfieReceipt> {
    use anyhow::Context as _;
    use xcap::Monitor;

    let candidates = list_script_kit_candidates().map_err(|error| {
        anyhow::anyhow!("failed to enumerate Script Kit windows for selfie capture: {error}")
    })?;
    let candidate_snapshots = candidates
        .iter()
        .map(|candidate| ScriptKitSelfieCandidateSnapshot {
            title: candidate.title.clone(),
            app_name: candidate.app_name.clone(),
            focused: candidate.focused,
            width: candidate.width,
            height: candidate.height,
        })
        .collect::<Vec<_>>();
    let dictation_open = crate::dictation::is_dictation_overlay_open();
    let candidate_index =
        select_script_kit_selfie_candidate_index(&candidate_snapshots, dictation_open)
        .context("no visible Script Kit window found for selfie capture")?;
    let candidate = &candidates[candidate_index];
    let captured_kind =
        classify_script_kit_selfie_candidate(&candidate_snapshots[candidate_index], dictation_open);
    let captured_state = captured_kind.state_label(state);

    let window_x = candidate.window.x().context("failed to read window x")?;
    let window_y = candidate.window.y().context("failed to read window y")?;
    let window_w = candidate
        .window
        .width()
        .context("failed to read window width")?;
    let window_h = candidate
        .window
        .height()
        .context("failed to read window height")?;

    let center_x = window_x + (window_w as i32 / 2);
    let center_y = window_y + (window_h as i32 / 2);
    let monitor = Monitor::from_point(center_x, center_y)
        .context("failed to resolve monitor containing Script Kit window")?;
    let monitor_x = monitor.x().context("failed to read monitor x")?;
    let monitor_y = monitor.y().context("failed to read monitor y")?;
    let monitor_w = monitor.width().context("failed to read monitor width")?;
    let monitor_h = monitor.height().context("failed to read monitor height")?;

    let crop_left = (window_x - SCRIPT_KIT_SELFIE_MARGIN).max(monitor_x);
    let crop_top = (window_y - SCRIPT_KIT_SELFIE_MARGIN).max(monitor_y);
    let crop_right = (window_x + window_w as i32 + SCRIPT_KIT_SELFIE_MARGIN)
        .min(monitor_x + monitor_w as i32);
    let crop_bottom = (window_y + window_h as i32 + SCRIPT_KIT_SELFIE_MARGIN)
        .min(monitor_y + monitor_h as i32);
    let crop_w = (crop_right - crop_left).max(1) as u32;
    let crop_h = (crop_bottom - crop_top).max(1) as u32;

    let relative_x = (crop_left - monitor_x).max(0) as u32;
    let relative_y = (crop_top - monitor_y).max(0) as u32;
    let image = monitor
        .capture_region(relative_x, relative_y, crop_w, crop_h)
        .context("failed to capture composited Script Kit desktop region")?;

    let created_at = chrono::Local::now();
    let timestamp = created_at.format("%Y%m%d-%H%M%S-%3f").to_string();
    let state_slug = slugify_script_kit_selfie_state(&captured_state);
    let receipt_id = format!("{timestamp}-{state_slug}");
    let dir = script_kit_selfie_output_dir();
    std::fs::create_dir_all(&dir).with_context(|| {
        format!(
            "failed to create Script Kit Selfie directory {}",
            dir.display()
        )
    })?;

    let png_path = dir.join(format!("{receipt_id}.png"));
    let receipt_path = dir.join(format!("{receipt_id}.json"));
    image
        .save(&png_path)
        .with_context(|| format!("failed to write {}", png_path.display()))?;

    let receipt = ScriptKitSelfieReceipt {
        schema_version: 1,
        command_id: SCRIPT_KIT_SELFIE_COMMAND_ID.to_string(),
        receipt_id,
        created_at: created_at.to_rfc3339(),
        state: captured_state,
        shortcut: SCRIPT_KIT_SELFIE_SHORTCUT.to_string(),
        capture_method: format!(
            "xcap.monitor.capture_region.composited_desktop.{}",
            captured_kind.capture_method_suffix()
        ),
        png_path: png_path.to_string_lossy().to_string(),
        receipt_path: receipt_path.to_string_lossy().to_string(),
        window_bounds: ScriptKitSelfieBounds {
            x: window_x,
            y: window_y,
            width: window_w,
            height: window_h,
        },
        monitor_bounds: ScriptKitSelfieBounds {
            x: monitor_x,
            y: monitor_y,
            width: monitor_w,
            height: monitor_h,
        },
        crop_bounds: ScriptKitSelfieBounds {
            x: crop_left,
            y: crop_top,
            width: crop_w,
            height: crop_h,
        },
        image_width: image.width(),
        image_height: image.height(),
    };

    let receipt_json = serde_json::to_vec_pretty(&receipt)?;
    std::fs::write(&receipt_path, receipt_json)
        .with_context(|| format!("failed to write {}", receipt_path.display()))?;

    Ok(receipt)
}

#[cfg(test)]
mod selfie_capture_tests {
    use super::{
        classify_script_kit_selfie_candidate, select_script_kit_selfie_candidate_index,
        slugify_script_kit_selfie_state, ScriptKitSelfieCandidateSnapshot,
        ScriptKitSelfieWindowKind,
    };

    #[test]
    fn selfie_state_slug_is_filename_safe() {
        assert_eq!(
            slugify_script_kit_selfie_state("Current App Commands/View"),
            "current-app-commands-view"
        );
        assert_eq!(slugify_script_kit_selfie_state(""), "unknown-state");
    }

    #[test]
    fn selfie_prefers_dictation_then_notes_before_main_window() {
        let candidates = vec![
            ScriptKitSelfieCandidateSnapshot {
                title: "Script Kit".to_string(),
                app_name: "Script Kit".to_string(),
                focused: true,
                width: 1200,
                height: 900,
            },
            ScriptKitSelfieCandidateSnapshot {
                title: "Notes".to_string(),
                app_name: "Script Kit".to_string(),
                focused: false,
                width: 350,
                height: 280,
            },
            ScriptKitSelfieCandidateSnapshot {
                title: "Dictation".to_string(),
                app_name: "Script Kit".to_string(),
                focused: false,
                width: 520,
                height: 120,
            },
        ];

        let index = select_script_kit_selfie_candidate_index(&candidates, true).unwrap();
        assert_eq!(index, 2);
        assert_eq!(
            classify_script_kit_selfie_candidate(&candidates[index], true),
            ScriptKitSelfieWindowKind::Dictation
        );
    }

    #[test]
    fn selfie_prefers_notes_over_focused_main_when_dictation_is_absent() {
        let candidates = vec![
            ScriptKitSelfieCandidateSnapshot {
                title: "Script Kit".to_string(),
                app_name: "Script Kit".to_string(),
                focused: true,
                width: 1200,
                height: 900,
            },
            ScriptKitSelfieCandidateSnapshot {
                title: "Notes".to_string(),
                app_name: "Script Kit".to_string(),
                focused: false,
                width: 350,
                height: 280,
            },
        ];

        let index = select_script_kit_selfie_candidate_index(&candidates, false).unwrap();
        assert_eq!(index, 1);
        assert_eq!(
            classify_script_kit_selfie_candidate(&candidates[index], false),
            ScriptKitSelfieWindowKind::Notes
        );
    }

    #[test]
    fn selfie_recognizes_titleless_dictation_overlay_when_dictation_is_open() {
        let candidates = vec![
            ScriptKitSelfieCandidateSnapshot {
                title: "Script Kit".to_string(),
                app_name: "Script Kit".to_string(),
                focused: true,
                width: 1200,
                height: 900,
            },
            ScriptKitSelfieCandidateSnapshot {
                title: "".to_string(),
                app_name: "Script Kit".to_string(),
                focused: false,
                width: 520,
                height: 120,
            },
        ];

        let index = select_script_kit_selfie_candidate_index(&candidates, true).unwrap();
        assert_eq!(index, 1);
        assert_eq!(
            classify_script_kit_selfie_candidate(&candidates[index], true),
            ScriptKitSelfieWindowKind::Dictation
        );
    }
}
