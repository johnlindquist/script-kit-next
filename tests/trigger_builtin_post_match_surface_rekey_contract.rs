//! Source-level contract for the `triggerBuiltin` semantic-surface re-key.
//!
//! `triggerBuiltin` changes the main window's active `AppView`, then
//! automation metadata must be re-keyed from that post-dispatch view. The
//! contract is intentionally named in production code so future agents see
//! the behavior instead of rediscovering the raw registry call shape.

const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");
const RUNTIME_STDIN_MATCH_CORE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        RUNTIME_STDIN_MATCH_CORE,
        "src/main_entry/runtime_stdin_match_core.rs",
    ),
    (RUNTIME_STDIN, "src/main_entry/runtime_stdin.rs"),
    (APP_RUN_SETUP, "src/main_entry/app_run_setup.rs"),
];

fn body_of<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let start = source
        .find(fn_name)
        .unwrap_or_else(|| panic!("missing function: {fn_name}"));
    let brace_rel = source[start..]
        .find(" {\n")
        .unwrap_or_else(|| panic!("missing function body opener: {fn_name}"));
    let body_start = start + brace_rel + 3;
    let bytes = source.as_bytes();
    let mut depth = 1_i32;
    let mut i = body_start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    &source[body_start..i]
}

fn trigger_builtin_arm<'a>(source: &'a str, path: &str) -> &'a str {
    let start = source
        .find("ref cmd @ ExternalCommand::TriggerBuiltin { .. } => {")
        .unwrap_or_else(|| panic!("{path}: missing triggerBuiltin arm"));
    let arm = &source[start..];
    let end = arm
        .find("ExternalCommand::SimulateKey")
        .unwrap_or(arm.len());
    &arm[..end]
}

fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn rekey_helper_reads_post_dispatch_view_and_updates_registry() {
    let body = body_of(
        DISPATCH,
        "pub(crate) fn rekey_main_automation_surface_after_trigger_builtin_dispatch(",
    );
    let body_compact = compact(body);

    assert!(
        body_compact.contains("self.rekey_main_automation_surface_from_current_view()"),
        "triggerBuiltin re-key helper must delegate to the shared current-view automation surface owner"
    );
    assert!(
        !body.contains("update_automation_semantic_surface("),
        "triggerBuiltin re-key helper must not copy the raw registry write; route through the shared owner"
    );
    assert!(
        !body.contains("upsert_automation_window"),
        "re-key helper must not upsert the whole window; stdin dispatchers do not own bounds/focus/title"
    );
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn every_dispatcher_calls_named_rekey_after_trigger_dispatch() {
    for (src, path) in DISPATCHERS {
        let arm = trigger_builtin_arm(src, path);
        let arm_compact = compact(arm);
        let dispatch_idx = arm_compact
            .find("view.dispatch_trigger_builtin(cmd,window,ctx)")
            .unwrap_or_else(|| panic!("{path}: triggerBuiltin arm must dispatch through helper"));
        let rekey_idx = arm_compact
            .find("view.rekey_main_automation_surface_after_trigger_builtin_dispatch()")
            .unwrap_or_else(|| panic!("{path}: triggerBuiltin arm must call named re-key helper"));

        assert!(
            dispatch_idx < rekey_idx,
            "{path}: triggerBuiltin arm must dispatch before re-keying semanticSurface"
        );
        assert!(
            !arm.contains("update_automation_semantic_surface("),
            "{path}: triggerBuiltin arm must use the named re-key helper, not duplicate raw registry calls"
        );
    }
}

#[test]
fn every_dispatcher_keeps_hide_path_script_list_rekey() {
    let hide_rekey = "crate::windows::update_automation_semantic_surface(\n                                    \"main\",\n                                    Some(\"scriptList\".to_string()),\n                                );";
    for (src, path) in DISPATCHERS {
        assert!(
            src.contains(hide_rekey),
            "{path} must keep the hide-path semanticSurface reset to scriptList"
        );
    }
}
