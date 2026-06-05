//! Source-level contract for DropPrompt native file-drop wiring and redacted receipts.

const DROP_SOURCE: &str = include_str!("../src/prompts/drop.rs");
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
fn drop_prompt_wires_gpui_external_paths_to_prompt_state() {
    assert!(
        DROP_SOURCE.contains("ExternalPaths"),
        "DropPrompt must use GPUI ExternalPaths for native file drops"
    );
    assert!(
        DROP_SOURCE.contains(".on_drop(cx.listener(|this, paths: &ExternalPaths"),
        "DropPrompt render must attach a native .on_drop handler"
    );
    assert!(
        DROP_SOURCE.contains("this.handle_external_paths(paths, cx)"),
        "DropPrompt .on_drop must route through the prompt-owned handler"
    );
    assert!(
        DROP_SOURCE.contains("DroppedFile::from_path(path.as_path())"),
        "native paths must be converted into DroppedFile metadata"
    );
}

#[test]
fn drop_prompt_automation_state_is_redacted_and_sdk_submit_stays_full_fidelity() {
    let state_result_variant = source_between(
        QUERY_OPS_VARIANTS,
        "#[serde(rename = \"stateResult\")]",
        "\n    // ============================================================\n    // ELEMENT QUERY",
    );
    assert!(
        state_result_variant.contains("#[serde(rename = \"drop\""),
        "StateResult must expose a prompt-specific `drop` payload"
    );
    assert!(
        state_result_variant.contains("drop_state: Option<serde_json::Value>"),
        "DropPrompt state should be optional outside DropPrompt"
    );

    let drop_state_block = source_between(
        PROMPT_HANDLER_SOURCE,
        "let drop_state = match &self.current_view",
        "\n\n                // Create the response",
    );
    for required in [
        "\"fileCount\"",
        "\"files\"",
        "file.automation_metadata(index)",
    ] {
        assert!(
            drop_state_block.contains(required),
            "DropPrompt getState payload missing redacted metadata token: {required}"
        );
    }
    for forbidden in ["\"path\"", "file.path"] {
        assert!(
            !drop_state_block.contains(forbidden),
            "DropPrompt getState must not expose paths; found {forbidden}"
        );
    }

    let submit_block = source_between(
        DROP_SOURCE,
        "pub(crate) fn submit",
        "\n    }\n\n    /// Cancel",
    );
    assert!(
        submit_block.contains("\"path\": f.path"),
        "DropPrompt submit must preserve full SDK file path payload"
    );
    assert!(
        submit_block.contains("\"name\": f.name") && submit_block.contains("\"size\": f.size"),
        "DropPrompt submit must preserve SDK name and size payload"
    );
}

#[test]
fn drop_prompt_get_elements_redacts_paths_and_marks_file_rows() {
    let collector = source_between(
        COLLECT_ELEMENTS_SOURCE,
        "fn collect_drop_prompt_elements",
        "\n    fn collect_template_prompt_elements",
    );
    assert!(
        collector.contains("ElementInfo::list(\"dropped-files\""),
        "DropPrompt getElements must expose the dropped-files list"
    );
    assert!(
        collector.contains("file.automation_metadata(index).to_string()"),
        "DropPrompt file rows must carry redacted automation metadata"
    );
    assert!(
        collector.contains("kind: Some(\"dropped_file\".to_string())"),
        "DropPrompt file rows must expose a stable dropped_file kind"
    );
    assert!(
        collector.contains("selectable: Some(false)"),
        "DropPrompt file metadata rows must not look executable/selectable"
    );
    assert!(
        !collector.contains("file.path.clone()"),
        "DropPrompt getElements must not expose full paths as row values"
    );
}

#[test]
fn sdk_get_state_surfaces_drop_prompt_receipts() {
    assert!(
        SDK_SOURCE.contains("export interface DropPromptState"),
        "SDK getState type must expose DropPrompt state receipts"
    );
    assert!(
        SDK_SOURCE.contains("drop?: DropPromptState"),
        "PromptState and StateResultMessage must carry optional drop state"
    );
    assert!(
        SDK_SOURCE.contains("drop: state.drop"),
        "global getState() must pass through the DropPrompt state payload"
    );
}
