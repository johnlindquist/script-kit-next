//! Source-level contract for unknown `triggerBuiltin` preserving the active surface.
//!
//! Unknown names are handled inside the shared dispatcher. They log and return
//! `None` without mutating `current_view`; the stdin arm still calls the named
//! post-dispatch re-key helper, which reads the unchanged view and therefore
//! preserves the existing automation `semanticSurface`.

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

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn unknown_trigger_builtin_path_logs_without_mutating_view_or_registry() {
    let dispatch_body = body_of(DISPATCH, "pub fn dispatch_trigger_builtin_name(");
    assert!(
        dispatch_body.contains("self.log_unknown_trigger_builtin(name);")
            && dispatch_body.contains("return None;"),
        "unknown triggerBuiltin names must log and return None from the shared dispatcher"
    );
    assert!(
        !dispatch_body.contains("self.current_view ="),
        "unknown triggerBuiltin names must not reset or otherwise mutate current_view"
    );
    assert!(
        !dispatch_body.contains("update_automation_semantic_surface("),
        "unknown triggerBuiltin names must not write the automation registry inside dispatch"
    );

    let unknown_log_body = body_of(DISPATCH, "fn log_unknown_trigger_builtin(");
    assert!(
        unknown_log_body.contains("trigger_builtin_unknown"),
        "unknown triggerBuiltin logging must keep the structured counter/log category"
    );
    assert!(
        !unknown_log_body.contains("current_view")
            && !unknown_log_body.contains("update_automation_semantic_surface("),
        "unknown triggerBuiltin logging must stay observational only"
    );
}

#[test]
fn stdin_trigger_builtin_arms_do_not_inline_unknown_name_handling() {
    for (src, path) in DISPATCHERS {
        let arm = trigger_builtin_arm(src, path);
        assert!(
            arm.contains("view.dispatch_trigger_builtin(cmd, window, ctx)"),
            "{path}: triggerBuiltin arm must delegate unknown-name behavior to the shared dispatcher"
        );
        assert!(
            arm.contains("rekey_main_automation_surface_after_trigger_builtin_dispatch"),
            "{path}: triggerBuiltin arm must run the named post-dispatch re-key helper"
        );
        assert!(
            !arm.contains("Unknown built-in:") && !arm.contains("self.log_unknown_trigger_builtin"),
            "{path}: triggerBuiltin arm must not inline unknown-name logging or handling"
        );
    }
}
