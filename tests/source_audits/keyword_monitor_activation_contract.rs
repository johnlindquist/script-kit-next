//! Source-audit contracts for keyword monitor activation.

use super::read_source;

const KEYWORD_MANAGER_PATH: &str = "src/keyword_manager/mod.rs";
const REFRESH_SCRIPTLETS_PATH: &str = "src/app_impl/refresh_scriptlets.rs";

fn macos_init_keyword_manager_region(source: &str) -> &str {
    source
        .split("#[cfg(target_os = \"macos\")]\npub fn init_keyword_manager")
        .nth(1)
        .and_then(|rest| rest.split("#[cfg(not(target_os = \"macos\"))]").next())
        .expect("macOS init_keyword_manager region should exist")
}

fn update_keyword_triggers_region(source: &str) -> &str {
    source
        .split("#[cfg(target_os = \"macos\")]\npub fn update_keyword_triggers_for_file")
        .nth(1)
        .and_then(|rest| rest.split("#[cfg(not(target_os = \"macos\"))]").next())
        .expect("macOS update_keyword_triggers_for_file region should exist")
}

fn handle_scriptlet_file_change_region(source: &str) -> &str {
    source
        .split("pub(crate) fn handle_scriptlet_file_change")
        .nth(1)
        .and_then(|rest| rest.split("/// Full refresh").next())
        .expect("handle_scriptlet_file_change region should exist")
}

#[test]
fn init_keyword_manager_enables_monitor_even_when_startup_trigger_count_is_zero() {
    let source = read_source(KEYWORD_MANAGER_PATH);
    let region = macos_init_keyword_manager_region(&source);
    let enable_idx = region
        .find("guard.enable()?")
        .expect("init_keyword_manager should enable the keyword monitor");

    assert!(
        !region[..enable_idx].contains("return Ok(Some(0))"),
        "startup must not return before enabling the monitor when zero triggers are loaded"
    );
    assert!(
        region.contains("No keyword triggers found at startup; enabling keyword monitor for future scriptlet updates"),
        "zero-trigger startup should log that the monitor still starts for future updates"
    );
}

#[test]
fn keyword_file_update_enables_monitor_when_disabled_and_triggers_exist() {
    let source = read_source(KEYWORD_MANAGER_PATH);
    let region = update_keyword_triggers_region(&source);

    for required in [
        "enabled_before",
        "trigger_count_after",
        "should_enable_monitor_after_trigger_update(enabled_before, trigger_count_after)",
        "guard.enable()",
        "keyword_trigger_update_applied",
    ] {
        assert!(
            region.contains(required),
            "keyword update path should contain activation guard: {required}"
        );
    }

    let update_idx = region
        .find("guard.update_triggers_for_file")
        .expect("trigger diff should be applied");
    let enable_idx = region
        .find("guard.enable()")
        .expect("monitor should be enabled after trigger update when needed");
    assert!(
        update_idx < enable_idx,
        "trigger diff must be applied before deciding whether to enable the monitor"
    );
}

#[test]
fn scriptlet_file_change_always_updates_keyword_triggers_before_metadata_diff_gating() {
    let source = read_source(REFRESH_SCRIPTLETS_PATH);
    let region = handle_scriptlet_file_change_region(&source);
    let update_idx = region
        .find("crate::keyword_manager::update_keyword_triggers_for_file")
        .expect("scriptlet file changes should always update keyword triggers");
    let diff_idx = region
        .find("let diff = diff_scriptlets")
        .expect("scriptlet metadata diff should still exist");

    assert!(
        update_idx < diff_idx,
        "keyword trigger updates must run before registration metadata diff gating"
    );
}

#[test]
fn keyword_activation_logs_are_content_light() {
    let source = read_source(KEYWORD_MANAGER_PATH);
    let region = update_keyword_triggers_region(&source);

    for required in [
        "keyword_trigger_update_applied",
        "scriptlet_count",
        "keyword_trigger_count",
        "added",
        "removed",
        "updated",
        "trigger_count_before",
        "trigger_count_after",
        "enabled_before",
        "enabled_after",
        "enable_attempted",
        "enable_failed",
    ] {
        assert!(
            region.contains(required),
            "keyword activation log should include content-light field: {required}"
        );
    }

    for forbidden in [
        "keyword = %kw",
        "trigger = %trigger",
        "trigger = %result.trigger",
        "replacement = %replacement",
        "replacement = ?",
        "content = %content",
        "content = ?",
        "raw_content",
        "clipboard",
    ] {
        assert!(
            !region.contains(forbidden),
            "keyword activation logs must not expose trigger or replacement content: {forbidden}"
        );
    }
}
