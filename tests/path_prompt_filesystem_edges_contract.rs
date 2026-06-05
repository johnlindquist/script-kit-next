//! Source-level contract for PathPrompt filesystem edge receipts.

const PATH_PROMPT_SOURCE: &str = include_str!("../src/prompts/path/prompt.rs");
const PATH_TYPES_SOURCE: &str = include_str!("../src/prompts/path/types.rs");
const COLLECT_ELEMENTS_SOURCE: &str = include_str!("../src/app_layout/collect_elements.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const QUERY_OPS_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const SDK_SOURCE: &str = include_str!("../scripts/kit-sdk.ts");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let end_rel = source[start_ix..]
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &source[start_ix..start_ix + end_rel]
}

#[test]
fn path_prompt_load_status_has_explicit_filesystem_edge_kinds() {
    assert!(
        PATH_TYPES_SOURCE.contains("pub enum PathLoadStatusKind"),
        "PathPrompt must expose a machine-stable load-status enum"
    );
    for required in [
        "Ready",
        "Empty",
        "FilteredEmpty",
        "Missing",
        "NotDirectory",
        "PermissionDenied",
        "ReadError",
    ] {
        assert!(
            PATH_TYPES_SOURCE.contains(required),
            "PathLoadStatusKind missing {required}"
        );
    }
    assert!(
        PATH_TYPES_SOURCE.contains("hidden_policy")
            && PATH_TYPES_SOURCE.contains("hidden_count")
            && PATH_TYPES_SOURCE.contains("failed_entry_count"),
        "PathPrompt status must record hidden-file policy and skipped/failed counts"
    );
}

#[test]
fn path_prompt_load_entries_does_not_collapse_error_edges_to_empty() {
    let load_entries = source_between(
        PATH_PROMPT_SOURCE,
        "pub(super) fn load_entries",
        "\n    /// Update filtered entries",
    );
    for required in [
        "PathLoadStatusKind::Missing",
        "PathLoadStatusKind::NotDirectory",
        "PathLoadStatusKind::PermissionDenied",
        "PathLoadStatusKind::ReadError",
        "PathLoadStatusKind::Empty",
        "name.starts_with('.')",
        "is_symlink",
    ] {
        assert!(
            load_entries.contains(required),
            "load_entries missing explicit edge handling token: {required}"
        );
    }
    assert!(
        !load_entries.contains("read_dir.flatten()"),
        "PathPrompt must count per-entry read failures instead of flattening them away"
    );
}

#[test]
fn path_prompt_get_state_exposes_path_receipts() {
    let state_result_variant = source_between(
        QUERY_OPS_VARIANTS,
        "#[serde(rename = \"stateResult\")]",
        "\n    // ============================================================\n    // ELEMENT QUERY",
    );
    assert!(
        state_result_variant.contains("#[serde(rename = \"path\""),
        "StateResult must expose an optional PathPrompt `path` payload"
    );

    let get_state_block = source_between(
        PROMPT_HANDLER_SOURCE,
        "let path_state = match &self.current_view",
        "\n\n                // Create the response",
    );
    assert!(
        get_state_block.contains("path_prompt.automation_state()"),
        "PathPrompt getState must use prompt-owned automation_state"
    );

    for required in [
        "\"currentPath\"",
        "\"entryCount\"",
        "\"visibleEntryCount\"",
        "\"status\"",
        "\"hiddenPolicy\"",
        "\"isSymlink\"",
    ] {
        assert!(
            PATH_PROMPT_SOURCE.contains(required),
            "PathPrompt automation state missing {required}"
        );
    }
}

#[test]
fn path_prompt_get_elements_exposes_status_and_symlink_rows() {
    let collector = source_between(
        COLLECT_ELEMENTS_SOURCE,
        "fn collect_path_prompt_elements",
        "\n    fn collect_env_prompt_elements",
    );
    for required in [
        "\"path-status\"",
        "status_kind: Some(path_prompt.visible_status_kind().as_str().to_string())",
        "\"path_status\"",
        "\"symlink\"",
        "\"directory\"",
        "\"file\"",
    ] {
        assert!(
            collector.contains(required),
            "PathPrompt getElements collector missing token: {required}"
        );
    }
}

#[test]
fn sdk_get_state_surfaces_path_prompt_receipts() {
    assert!(
        SDK_SOURCE.contains("export interface PathPromptState"),
        "SDK getState type must expose PathPrompt state receipts"
    );
    assert!(
        SDK_SOURCE.contains("path?: PathPromptState"),
        "PromptState and StateResultMessage must carry optional path state"
    );
    assert!(
        SDK_SOURCE.contains("path: state.path"),
        "global getState() must pass through the PathPrompt state payload"
    );
}
