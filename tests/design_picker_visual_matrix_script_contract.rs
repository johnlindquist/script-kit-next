//! Phase 5 — contract for the agentic visual-matrix script.
//!
//! The actual screenshot capture requires a running launcher with the
//! Phase 1 Design Picker compiled in; that's a user-side step. This test
//! pins the script's structural contract so the catalog ids, flags, and
//! RPC method names stay in sync with the spec at
//! `.goals/design-variants-overhaul.md`.

use std::fs;
use std::process::Command;

const SCRIPT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/agentic/design-picker-visual-matrix.ts"
);

fn script() -> String {
    fs::read_to_string(SCRIPT_PATH).expect("matrix script must exist")
}

#[test]
fn matrix_script_lists_all_25_catalog_ids() {
    let body = script();
    let required = [
        "script-kit-classic",
        "pro-dense",
        "ambient-quiet",
        "focus-zen",
        "minimal-ink",
        "retro-terminal",
        "paper-print",
        "glass-frost",
        "neon-cyber",
        "apple-hig",
        "high-density-list",
        "accessibility-high-contrast",
        "retro-amber",
        "editorial-brutalist",
        "brutalist-grid",
        "liquid-glass-compact",
        "synthwave",
        "material-you",
        "mocha-warm",
        "ocean-deep",
        "pastel-mist",
        "playful-pop",
        "mono-contrast",
        "command-center",
        "gallery-visual",
    ];
    for id in required {
        assert!(
            body.contains(id),
            "matrix script missing required catalog id `{}`",
            id
        );
    }
}

#[test]
fn matrix_script_supports_required_flags() {
    let body = script();
    let required_flags = [
        "--capture-screenshots",
        "--verify-state-receipts",
        "--cleanup",
        "--dry-run",
        "--sizes",
        "--designs",
        "--session",
    ];
    for flag in required_flags {
        assert!(
            body.contains(flag),
            "matrix script missing required flag `{}`",
            flag
        );
    }
}

#[test]
fn matrix_script_uses_real_capture_rpc_methods() {
    let body = script();
    assert!(
        body.contains("kit/state"),
        "must verify state via kit/state"
    );
    assert!(
        body.contains("computer/capture_native_window"),
        "must capture window via computer/capture_native_window"
    );
    assert!(
        body.contains("computer/get_frontmost_native_window"),
        "must locate launcher window via computer/get_frontmost_native_window"
    );
    assert!(
        body.contains("designPicker"),
        "must assert semanticSurface=designPicker"
    );
}

#[test]
fn matrix_script_dry_run_lists_50_cells() {
    let output = Command::new("bun")
        .arg(SCRIPT_PATH)
        .arg("--dry-run")
        .output();
    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("skipping live dry-run (bun unavailable): {e}");
            return;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cells = stdout.lines().filter(|l| l.starts_with("PLAN ")).count();
    assert_eq!(
        cells, 50,
        "default dry-run must plan 25 designs × 2 sizes = 50 cells, got {}",
        cells
    );
    assert!(
        stdout.contains("\"totalCells\":50"),
        "JSON envelope must report totalCells=50"
    );
}

#[test]
// @lat: [[verification#Design Picker persistence]]
fn matrix_script_asserts_persisted_active_id_after_restart() {
    let body = script();
    assert!(
        body.contains("--expect-persisted-active-id"),
        "matrix script must expose the --expect-persisted-active-id flag"
    );
    assert!(
        body.contains("persistedActiveId"),
        "matrix script must verify the design.persistedActiveId field"
    );
    assert!(
        body.contains("fallbackApplied"),
        "matrix script must verify the design.fallbackApplied field"
    );
    assert!(
        body.contains("state.design"),
        "matrix script must read the design state receipt from state.design"
    );
}
