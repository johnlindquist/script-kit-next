//! Phase 6 source-audit baseline for the design-variants overhaul.
//!
//! Spec target: "no production module outside `src/designs/legacy_migration.rs`
//! references `DesignVariant::` symbols." Today there are ~173 such call sites
//! tracked in the survey; Phase 6 sweeps them out and tightens this audit.
//!
//! Until then, this test fixes two contracts:
//!
//! 1. The migration module IS allowed to reference `DesignVariant::` and
//!    therefore must contain at least one such reference (otherwise the
//!    bridge has been removed too early).
//! 2. The legacy-mapping integration test (`tests/legacy_design_variant_migration.rs`)
//!    is also allowed and must continue to assert against `DesignVariant`.
//!
//! A `#[ignore]`d enforcement test pins the eventual Phase-6 state: zero
//! production references outside the migration module. Codex/the user can
//! flip `#[ignore]` off once the sweep finishes.

use std::fs;
use std::path::{Path, PathBuf};

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
            // Skip vendored crates, target, and node_modules.
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

fn count_design_variant_references(root: &Path) -> usize {
    let mut files = Vec::new();
    walk_rust_files(root, &mut files);
    let mut total = 0;
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        total += text.matches("DesignVariant::").count();
    }
    total
}

#[test]
fn migration_module_still_references_design_variant() {
    let path = repo_root().join("src/designs/legacy_migration.rs");
    let text = fs::read_to_string(&path).expect("legacy_migration.rs must exist");
    assert!(
        text.contains("DesignVariant"),
        "src/designs/legacy_migration.rs must keep at least one `DesignVariant` reference to bridge legacy configs"
    );
}

#[test]
fn migration_integration_test_still_references_design_variant() {
    let path = repo_root().join("tests/legacy_design_variant_migration.rs");
    let text = fs::read_to_string(&path).expect("legacy migration test must exist");
    assert!(
        text.contains("DesignVariant"),
        "tests/legacy_design_variant_migration.rs must keep asserting against `DesignVariant`"
    );
}

#[test]
fn design_variant_call_sites_have_a_known_phase6_ceiling() {
    // Measured at Phase-1 landing: 285 `DesignVariant::` occurrences across
    // src/ + tests/ (re-measured 2026-06-09 at 286 on main). The ceiling here
    // is a soft cap: it prevents regression growth and lets the Phase 6 sweep
    // ratchet downward by lowering this number as call sites are migrated to
    // the registry.
    let total = count_design_variant_references(&repo_root().join("src"));
    let total_tests = count_design_variant_references(&repo_root().join("tests"));
    let grand_total = total + total_tests;
    const CEILING: usize = 286;
    assert!(
        grand_total <= CEILING,
        "`DesignVariant::` reference count grew to {} (ceiling {}). \
         Phase 6 should drive this down, not up.",
        grand_total,
        CEILING
    );
}

#[test]
#[ignore = "Phase 6 endpoint: enable after sweeping every production callsite to the registry"]
fn phase6_no_design_variant_outside_migration() {
    let src = repo_root().join("src");
    let mut files = Vec::new();
    walk_rust_files(&src, &mut files);
    let mut offenders = Vec::new();
    for path in files {
        // Allow the migration module and its sibling enum definition.
        let rel = path
            .strip_prefix(repo_root())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel == "src/designs/legacy_migration.rs" || rel == "src/designs/core/variant.rs" {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text.contains("DesignVariant::") {
            offenders.push(rel);
        }
    }
    assert!(
        offenders.is_empty(),
        "Phase 6 violation: `DesignVariant::` referenced outside legacy_migration.rs in {} file(s): {:?}",
        offenders.len(),
        offenders
    );
}
