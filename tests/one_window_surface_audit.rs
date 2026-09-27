//! The one-window rule, checked on the source: Script Kit draws on its main window only.
//!
//! Production code only: not a test file, not inside a `#[cfg(test)]` item or a module declared
//! under one, not a comment. Across `src/` and `crates/`:
//! 1. no `open_window(` outside the main window's setup, the snap guide and dev binaries;
//! 2. no native `NSWindow`/`NSPanel` allocation outside the footer's glass capsule panel;
//! 3. none of the retired surface files exist;
//! 4. none of the retired surface names are used.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED_OPEN_WINDOW: &[(&str, &str)] = &[
    ("src/main_entry/", "the main window"),
    (
        "src/window_control/snap_overlay.rs",
        "the snap guide drawn over other apps' windows, not a Script Kit surface",
    ),
    ("src/bin/", "dev binaries, not shipped in the app"),
];

const ALLOWED_NATIVE_WINDOW: &[(&str, &str)] = &[(
    "src/footer_popup_glass_geometry.rs",
    "the borderless child panel pinned under the main window that carries the ↵ glass capsules",
)];

const RETIRED_FILES: &[&str] = &[
    "src/ai/agent_chat/ui/chat_window.rs",
    "src/ai/agent_chat/ui/history_popup.rs",
    "src/notes/window/window_ops.rs",
    "src/dictation/window.rs",
    "src/dictation/microphone_popup_window.rs",
    "src/actions/window.rs",
    "src/footer_popup_overlay.rs",
    "src/confirm",
    "src/toast_popup.rs",
    "src/toast_manager",
    "src/undo_offer.rs",
    "src/warning_banner.rs",
    "src/hud_manager",
];

const RETIRED_NAMES: &[&str] = &[
    r"\bshow_error_toast\b",
    r"\bToastManager\b",
    r"\bshow_hud\b",
    r"\bHudManager\b",
    r"\bopen_confirm_(window|dialog)\b",
    r"\bConfirmWindow\b",
    r"\bUndoOfferWindow\b",
    r"\bWarningBanner\b",
    r"\bopen_actions_window\b",
    r"\bActionsWindow\b",
    r"\bopen_chat_window\b",
    r"\bis_chat_window_open\b",
    r"\bChatWindow\b",
    r"\bopen_history_popup\b",
    r"\bHistoryPopup\b",
    r"\bopen_notes_window\b",
    r"\bNotesWindow\b",
    r"\bopen_dictation_window\b",
    r"\bMicrophonePopup\w*\b",
    r"\bDictationWindow\b",
    r"\bconfirm_modal_shell\b",
    r"\brender_main_window_modal_dim_layer\b",
    // ⌘K in a prompt stages in place; there is no popover to anchor or dismiss.
    r"\brender_actions_backdrop\w*\b",
    r"\bActionsBackdropConfig\b",
    r"\bprompt_actions_dialog_offsets\b",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if matches!(name, "target" | "node_modules" | "vendor" | ".git") {
                continue;
            }
            walk_rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn test_attr() -> Regex {
    Regex::new(r"^\s*#\[cfg\((all\()?test\b").unwrap()
}

fn is_test_path(rel: &str) -> bool {
    Regex::new(r"(^|/)tests?(/|\.rs$)|_tests?\.rs$|test_support/|(^|/)benches/")
        .unwrap()
        .is_match(rel)
}

fn braces(line: &str) -> i64 {
    line.matches('{').count() as i64 - line.matches('}').count() as i64
}

/// `(line number, trimmed line)` for every line outside `#[cfg(test)]` items and comments.
fn production_lines(text: &str) -> Vec<(usize, &str)> {
    let attr = test_attr();
    let mut lines = Vec::new();
    let (mut pending, mut depth, mut skipping) = (false, 0i64, false);
    for (i, line) in text.lines().enumerate() {
        let s = line.trim();
        if skipping {
            depth += braces(s);
            if depth <= 0 {
                skipping = false;
            }
            continue;
        }
        if attr.is_match(s) {
            pending = true;
            continue;
        }
        if pending {
            if s.starts_with("#[") || s.is_empty() {
                continue;
            }
            pending = false;
            depth = braces(s);
            skipping = depth > 0 || !(s.ends_with(';') || s.ends_with('}'));
            continue;
        }
        if s.starts_with("//") {
            continue;
        }
        lines.push((i + 1, s));
    }
    lines
}

/// Whether `rel`'s `mod` declaration, or an ancestor module's, carries `#[cfg(test)]`.
fn declared_under_test(root: &Path, rel: &str) -> bool {
    let Some(stem) = rel
        .strip_prefix("src/")
        .and_then(|rest| rest.strip_suffix(".rs"))
    else {
        return false;
    };
    let attr = test_attr();
    let mut parts: Vec<&str> = stem.split('/').collect();
    if parts.last() == Some(&"mod") {
        parts.pop();
    }
    while let Some((name, parent)) = parts.split_last() {
        if parent.is_empty() && matches!(*name, "lib" | "main") {
            break;
        }
        let candidates = if parent.is_empty() {
            vec![root.join("src/lib.rs"), root.join("src/main.rs")]
        } else {
            let base = root.join("src").join(parent.join("/"));
            vec![base.join("mod.rs"), base.with_extension("rs")]
        };
        let decl = Regex::new(&format!(r"^\s*(pub(\([^)]*\))?\s+)?mod\s+{name}\s*;")).unwrap();
        for candidate in candidates {
            let Ok(text) = fs::read_to_string(&candidate) else {
                continue;
            };
            let lines: Vec<&str> = text.lines().collect();
            for (j, line) in lines.iter().enumerate() {
                if !decl.is_match(line) {
                    continue;
                }
                for prev in lines[..j].iter().rev() {
                    let p = prev.trim();
                    if !(p.starts_with("#[") || p.starts_with("//")) {
                        break;
                    }
                    if attr.is_match(prev) {
                        return true;
                    }
                }
            }
        }
        parts = parent.to_vec();
    }
    false
}

struct Site {
    at: String,
    code: String,
}

/// Every production line in `src/` and `crates/` that matches `pattern`, with its file.
fn production_matches(pattern: &Regex) -> Vec<(String, Site)> {
    let root = repo_root();
    let mut files = Vec::new();
    walk_rust_files(&root.join("src"), &mut files);
    walk_rust_files(&root.join("crates"), &mut files);
    files.sort();
    let mut sites = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if is_test_path(&rel) || declared_under_test(&root, &rel) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if !pattern.is_match(&text) {
            continue;
        }
        for (line, code) in production_lines(&text) {
            if pattern.is_match(code) {
                sites.push((
                    rel.clone(),
                    Site {
                        at: format!("{rel}:{line}"),
                        code: code.chars().take(120).collect(),
                    },
                ));
            }
        }
    }
    sites
}

fn allowed(rel: &str, table: &[(&str, &str)]) -> bool {
    table.iter().any(|(prefix, _)| rel.starts_with(prefix))
}

fn listing(sites: &[&Site]) -> String {
    sites
        .iter()
        .map(|site| format!("  {}  {}", site.at, site.code))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn no_window_opens_outside_the_main_window() {
    let sites = production_matches(&Regex::new(r"\bopen_window\(").unwrap());
    assert!(
        sites
            .iter()
            .any(|(rel, _)| rel.starts_with("src/main_entry/")),
        "the audit should find the main window's own open_window in src/main_entry/"
    );
    let auxiliary: Vec<&Site> = sites
        .iter()
        .filter(|(rel, _)| !allowed(rel, ALLOWED_OPEN_WINDOW))
        .map(|(_, site)| site)
        .collect();
    assert!(
        auxiliary.is_empty(),
        "every surface draws in the main window; these open another window:\n{}",
        listing(&auxiliary)
    );
}

#[test]
fn no_native_window_is_allocated_outside_the_glass_capsule_panel() {
    let sites = production_matches(
        &Regex::new(
            r"class!\((NSPanel|NSWindow)\)\s*,\s*alloc|NS(Panel|Window)::alloc|initWithContentRect",
        )
        .unwrap(),
    );
    assert!(
        sites
            .iter()
            .any(|(rel, _)| allowed(rel, ALLOWED_NATIVE_WINDOW)),
        "the audit should find the ↵ glass capsule panel's allocation"
    );
    let native: Vec<&Site> = sites
        .iter()
        .filter(|(rel, _)| !allowed(rel, ALLOWED_NATIVE_WINDOW))
        .map(|(_, site)| site)
        .collect();
    assert!(
        native.is_empty(),
        "every surface draws in the main window; these allocate a native window or panel:\n{}",
        listing(&native)
    );
}

#[test]
fn retired_surface_files_are_gone() {
    let root = repo_root();
    let present: Vec<&str> = RETIRED_FILES
        .iter()
        .copied()
        .filter(|path| root.join(path).exists())
        .collect();
    assert!(
        present.is_empty(),
        "retired window, popup, modal, toast and HUD files are back: {present:?}"
    );
}

#[test]
fn retired_surface_names_are_unused() {
    let sites = production_matches(&Regex::new(&RETIRED_NAMES.join("|")).unwrap());
    let used: Vec<&Site> = sites.iter().map(|(_, site)| site).collect();
    assert!(
        used.is_empty(),
        "retired window, popup, modal, toast and HUD names are used again:\n{}",
        listing(&used)
    );
}
