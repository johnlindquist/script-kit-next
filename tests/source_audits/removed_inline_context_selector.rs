//! The deprecated inline Agent Chat/Day context selector was deleted in favor of
//! routing `@` context through the main menu. These checks guard against stale
//! fixtures, scripts, or source names resurrecting the deleted visual surface.

use std::fs;
use std::path::{Path, PathBuf};

const SELF_PATH: &str = "tests/source_audits/removed_inline_context_selector.rs";

const SCAN_ROOTS: &[&str] = &[
    "src",
    "tests",
    "scripts/agentic",
    "scripts/devtools",
    "FEATURES.md",
    "GLOSSARY.md",
];

const FORBIDDEN_SNIPPETS: &[&str] = &[
    "file:screenflow",
    "eggo-expression-grid",
    "eggo-brand",
    "picker_popup",
    "mention-anchor-probe",
    "agent_chat_mention_popup",
    "agent_chat_popup_registry",
    "prompt_popup_fixture",
    "menu_syntax_trigger_popup",
    "menu_syntax_object_selector_popup",
    "inline context popup",
    "context picker popup",
];

#[test]
fn deleted_inline_context_selector_traces_do_not_return() {
    let mut failures = Vec::new();
    for path in source_files() {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for forbidden in FORBIDDEN_SNIPPETS {
            if content.contains(forbidden) {
                failures.push(format!("{} contains {forbidden:?}", path.display()));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "deleted inline context selector traces returned:\n{}",
        failures.join("\n")
    );
}

fn source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in SCAN_ROOTS {
        let path = Path::new(root);
        if path.is_file() {
            push_if_scannable(path, &mut files);
        } else {
            collect_files(path, &mut files);
        }
    }
    files
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("target" | "target-agent" | "node_modules" | "vendor" | "dist")
            ) {
                continue;
            }
            collect_files(&path, files);
        } else {
            push_if_scannable(&path, files);
        }
    }
}

fn push_if_scannable(path: &Path, files: &mut Vec<PathBuf>) {
    if path == Path::new(SELF_PATH) {
        return;
    }
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return;
    };
    if matches!(ext, "rs" | "ts" | "tsx" | "js" | "mjs" | "md") {
        files.push(path.to_path_buf());
    }
}
