//! Source-level contract test for the `tool-hide-rpc-surface-reset`
//! user story (Run 2 Pass #19 side finding, closed in Pass #21).
//!
//! Pass #19 added a re-key to `"scriptList"` inside `hide_main_window_helper`
//! — but hides issued via the stdin `ExternalCommand::Hide` RPC path take a
//! different code path that does NOT route through the helper. That left
//! `AutomationWindowInfo.semanticSurface` stuck at whatever the last
//! subview was (e.g. `"browserTabs"`) after a hide RPC, diverging from
//! the live `getState.promptType`.
//!
//! The fix now schedules the reset through
//! `defer_reset_to_script_list_after_main_window_hidden`, so the Pass #19
//! surface re-key still happens but only after the native hide has been
//! enqueued. This prevents a visible ScriptList frame while AppKit is still
//! closing the panel.
//!
//! If any dispatcher silently drops either call, the hide-RPC path for
//! that entry point regresses. This contract pins the fix in source.

const STDIN_CORE_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const STDIN_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");

fn hide_body<'a>(name: &str, source: &'a str) -> &'a str {
    let hide_start = source
        .find("ExternalCommand::Hide {")
        .unwrap_or_else(|| panic!("{name} missing ExternalCommand::Hide arm"));
    let next_arm = source[hide_start..]
        .find("ExternalCommand::SetFilter")
        .expect("Hide arm must be followed by SetFilter arm");
    &source[hide_start..hide_start + next_arm]
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_schedule_hidden_script_list_reset() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let hide_body = hide_body(name, source);
        assert!(
            hide_body.contains("view.defer_reset_to_script_list_after_main_window_hidden("),
            "{name} `ExternalCommand::Hide` arm must schedule the hidden \
             ScriptList reset. Without it, the inner view can stay on \
             whatever subview was active when hide fired."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_do_not_reset_or_rekey_before_native_hide() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let hide_body = hide_body(name, source);
        let hide_idx = hide_body
            .find("platform::defer_hide_main_window(ctx);")
            .unwrap_or_else(|| panic!("{name} Hide arm missing defer_hide_main_window"));
        assert!(
            !hide_body[..hide_idx].contains("view.reset_to_script_list(ctx);")
                && !hide_body[..hide_idx].contains("update_automation_semantic_surface("),
            "{name} `ExternalCommand::Hide` arm must not reset or re-key \
             to ScriptList before the native hide is enqueued."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_sequence_native_hide_then_hidden_reset() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let hide_body = hide_body(name, source);
        let hide_idx = hide_body
            .find("platform::defer_hide_main_window(ctx);")
            .unwrap_or_else(|| panic!("{name} Hide arm missing defer_hide_main_window"));
        let deferred_reset_idx = hide_body
            .find("view.defer_reset_to_script_list_after_main_window_hidden(")
            .unwrap_or_else(|| panic!("{name} Hide arm missing hidden reset scheduling"));

        assert!(
            hide_idx < deferred_reset_idx,
            "{name} `ExternalCommand::Hide` arm must enqueue native hide \
             before scheduling its hidden ScriptList reset."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_main_window_helper_defers_reset_after_native_hide() {
    // The non-RPC hide helper owns the visible hotkey/menu close path.
    // It must not mutate the visible route to ScriptList before the
    // deferred native hide is enqueued; instead it schedules the reset
    // through the hidden-reset helper.
    const HELPER_SOURCE: &str = include_str!("../src/main_sections/window_visibility.rs");
    let helper_start = HELPER_SOURCE
        .find("fn hide_main_window_helper(")
        .expect("src/main_sections/window_visibility.rs must define `hide_main_window_helper`");
    let helper_end = HELPER_SOURCE[helper_start..]
        .find("pub fn show_main_window()")
        .map(|offset| helper_start + offset)
        .unwrap_or(HELPER_SOURCE.len());
    let helper_body = &HELPER_SOURCE[helper_start..helper_end];
    assert!(
        helper_body.contains("view.defer_reset_to_script_list_after_main_window_hidden("),
        "`hide_main_window_helper` must schedule the hidden ScriptList reset"
    );
    let hide_idx = helper_body
        .find("platform::defer_hide_main_window(cx);")
        .expect("hide helper must enqueue native hide");
    let reset_idx = helper_body
        .find("view.defer_reset_to_script_list_after_main_window_hidden(")
        .expect("hide helper must schedule hidden reset");
    assert!(
        hide_idx < reset_idx,
        "`hide_main_window_helper` must schedule its ScriptList reset after native hide is enqueued"
    );
    assert!(
        !helper_body[..hide_idx].contains("view.reset_to_script_list(ctx);")
            && !helper_body[..hide_idx].contains("update_automation_semantic_surface("),
        "`hide_main_window_helper` must not reset or re-key to ScriptList before the native hide"
    );
}
