//! Source-level contract for the retired repo-local Codex Stop hook.
//!
//! The legacy Stop hook and `.codex/hooks.json` were removed by commit
//! `4534aac79 Remove legacy Codex hooks.` This test intentionally guards the
//! current contract: this repository no longer ships that hook surface, so
//! clean checkouts must not require `.codex/hooks/*` files to compile.

use std::path::Path;

#[test]
fn legacy_codex_stop_hook_is_not_a_repo_contract() {
    assert!(
        !Path::new(".codex/hooks.json").exists(),
        "legacy repo-local Codex hooks config was intentionally removed"
    );
    assert!(
        !Path::new(".codex/hooks/stop-continue-agentic-testing.ts").exists(),
        "legacy Stop continuation hook was intentionally removed"
    );
    assert!(
        !Path::new(".codex/hooks/marketing-infographics.md").exists(),
        "legacy marketing continuation prompt file was intentionally removed"
    );
}

#[test]
fn codex_directory_only_keeps_the_repo_skill_link() {
    assert!(
        Path::new(".codex/skills").exists(),
        ".codex remains only as the repo-local skills compatibility link"
    );
    assert!(
        !Path::new(".codex/hooks").exists(),
        "no replacement repo-local Codex hook directory exists in this checkout"
    );
}
