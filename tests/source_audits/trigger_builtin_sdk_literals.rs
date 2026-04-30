//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR1:
//! publish-time failure for unknown `triggerBuiltin` names in TS / JS.
//!
//! This source audit scans every `.ts`, `.tsx`, `.mts`, `.mjs`, and `.js`
//! file under `scripts/` and `tests/` for two patterns:
//!
//! 1. The function-call form: `triggerBuiltin("some-name")` or
//!    `triggerBuiltin('some-name')` or `triggerBuiltin(`some-name`)`.
//! 2. The JSON-literal form used by `scripts/agentic/index.ts`:
//!    `{"type":"triggerBuiltin","name":"some-name"}` or the newer
//!    `"builtinId":"builtin/..."` variant.
//!
//! Every extracted name must resolve via the canonical
//! [`TriggerBuiltinRegistry`] — if a Bun-land typo or a dropped
//! registration leaves a literal that nothing in Rust can route, this
//! test fails before `cargo test` finishes, not silently at runtime.
//!
//! The audit deliberately reads the registry via the public accessor
//! so regressions in the registry API also surface here.

use script_kit_gpui::builtins::trigger_registry::registry;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to walk for SDK / script sources.
const SCAN_ROOTS: &[&str] = &["scripts", "tests"];

/// File extensions treated as TS / JS source.
const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts", "js", "mjs", "cjs"];

/// Collect every TS / JS source path under [`SCAN_ROOTS`].
fn collect_source_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in SCAN_ROOTS {
        walk(Path::new(root), &mut out);
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
        {
            out.push(path);
        }
    }
}

/// Extract every `triggerBuiltin` literal from one source file. Returns
/// a list of `(line_number, extracted_name)` pairs.
fn extract_literals(source: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        extract_function_call_literals(line, idx + 1, &mut out);
        extract_json_literals(line, idx + 1, &mut out);
    }
    out
}

/// Match `triggerBuiltin("X")`, `triggerBuiltin('X')`, or with backticks.
fn extract_function_call_literals(line: &str, line_no: usize, out: &mut Vec<(usize, String)>) {
    let needle = "triggerBuiltin(";
    let mut cursor = 0;
    while let Some(rel) = line[cursor..].find(needle) {
        let call_start = cursor + rel + needle.len();
        let rest = &line[call_start..];
        let trimmed = rest.trim_start();
        let padding = rest.len() - trimmed.len();
        let literal_start = call_start + padding;
        let Some(first) = trimmed.chars().next() else {
            cursor = call_start;
            continue;
        };
        if matches!(first, '"' | '\'' | '`') {
            let quote = first;
            let after_quote = literal_start + quote.len_utf8();
            if let Some(end_rel) = line[after_quote..].find(quote) {
                let name = &line[after_quote..after_quote + end_rel];
                out.push((line_no, name.to_string()));
                cursor = after_quote + end_rel + quote.len_utf8();
                continue;
            }
        }
        cursor = call_start;
    }
}

/// Match the JSON-literal form used by `scripts/agentic/index.ts`:
/// `"type":"triggerBuiltin","name":"X"` or `"builtinId":"X"`.
fn extract_json_literals(line: &str, line_no: usize, out: &mut Vec<(usize, String)>) {
    let anchor = r#""type":"triggerBuiltin""#;
    let mut cursor = 0;
    while let Some(rel) = line[cursor..].find(anchor) {
        let anchor_end = cursor + rel + anchor.len();
        let rest = &line[anchor_end..];
        for key in [r#""name":""#, r#""builtinId":""#] {
            if let Some(key_rel) = rest.find(key) {
                let key_end = anchor_end + key_rel + key.len();
                if let Some(close_rel) = line[key_end..].find('"') {
                    let name = &line[key_end..key_end + close_rel];
                    out.push((line_no, name.to_string()));
                }
            }
        }
        cursor = anchor_end;
    }
}

/// Any literal equal to this bootstrap token is tolerated — it is the
/// placeholder used in the SDK reference doc generator, not a real call.
const ALLOWED_DOC_LITERALS: &[&str] = &["<name>", "name", "<built-in-name>"];

#[test]
fn every_trigger_builtin_literal_in_ts_resolves() {
    let registry = registry();
    let mut unknown: Vec<String> = Vec::new();

    for path in collect_source_files() {
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        for (line_no, raw_name) in extract_literals(&source) {
            if ALLOWED_DOC_LITERALS.contains(&raw_name.as_str()) {
                continue;
            }
            if registry.resolve(&raw_name).is_none() {
                unknown.push(format!(
                    "{}:{}: triggerBuiltin literal `{}` does not resolve via TriggerBuiltinRegistry",
                    path.display(),
                    line_no,
                    raw_name
                ));
            }
        }
    }

    assert!(
        unknown.is_empty(),
        "Unknown triggerBuiltin literals in TS/JS sources:\n{}\n\n\
         Every `triggerBuiltin(\"X\")` or `\"type\":\"triggerBuiltin\",\"name\":\"X\"` literal \
         must resolve via the canonical registry in src/builtins/trigger_registry.rs. \
         If `X` is a new built-in, add a variant to `TriggerBuiltin`. If `X` is a typo, \
         fix it — a runtime no-op is not acceptable feedback.",
        unknown.join("\n")
    );
}

#[test]
fn extract_literals_finds_function_call_form() {
    let mut out = Vec::new();
    extract_function_call_literals(r#"  triggerBuiltin("clipboard-history");"#, 1, &mut out);
    assert_eq!(out, vec![(1usize, "clipboard-history".to_string())]);
}

#[test]
fn extract_literals_finds_json_name_form() {
    let mut out = Vec::new();
    extract_json_literals(
        r#"send(session, '{"type":"triggerBuiltin","name":"tab-ai"}')"#,
        42,
        &mut out,
    );
    assert_eq!(out, vec![(42usize, "tab-ai".to_string())]);
}

#[test]
fn extract_literals_finds_json_builtin_id_form() {
    let mut out = Vec::new();
    extract_json_literals(
        r#"send('{"type":"triggerBuiltin","builtinId":"builtin/clipboard-history"}')"#,
        1,
        &mut out,
    );
    assert_eq!(out, vec![(1usize, "builtin/clipboard-history".to_string())]);
}

#[test]
fn extract_literals_skips_comments_only_when_quoted() {
    // The extractor deliberately does not try to skip comments — the
    // registry resolution catches doc mentions because their names are
    // either in ALLOWED_DOC_LITERALS or do not look like literal calls.
    let mut out = Vec::new();
    extract_function_call_literals("// triggerBuiltin(bar) — doc only", 1, &mut out);
    assert!(
        out.is_empty(),
        "bare identifier (no quotes) should not be extracted: {out:?}"
    );
}
