use std::path::Path;

fn read_source(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// Clipboard sediment may track copied content for brain storage, but that path
/// must not resurrect the removed post-copy popup/HUD feature. This guard exists
/// because stale config/docs previously made the deleted UI look supported.
#[test]
fn clipboard_sediment_has_no_post_copy_ui_path() {
    assert!(
        !Path::new("src/clipboard_history/post_copy.rs").exists(),
        "post-copy UI hooks must not come back for clipboard sediment"
    );
    assert!(
        !Path::new("src/clipboard_history/tap_window.rs").exists(),
        "the removed tap-window state machine must not come back for copy tracking"
    );

    let clipboard_mod = read_source("src/clipboard_history/mod.rs");
    let config_types = read_source("src/config/types.rs");
    let config_loader = read_source("src/config/loader.rs");
    let preflight = read_source("src/main_entry/preflight.rs");
    let app_run_setup = read_source("src/main_entry/app_run_setup.rs");
    let config_cli = read_source("scripts/config-cli.ts");
    let config_schema = read_source("scripts/config-schema.ts");

    for (label, source) in [
        ("clipboard module", clipboard_mod.as_str()),
        ("config types", config_types.as_str()),
        ("config loader", config_loader.as_str()),
        ("preflight", preflight.as_str()),
        ("app run setup", app_run_setup.as_str()),
        ("config CLI", config_cli.as_str()),
        ("config schema", config_schema.as_str()),
    ] {
        for forbidden in [
            "clipboardHistoryPostCopyMenu",
            "ClipboardHistoryPostCopyMenuConfig",
            "PostCopyMenuConfig",
            "configure_post_copy_menu",
            "install_post_copy_tracker",
            "register_kept_hud_whisper",
            "request_kept_hud_whisper",
            "notify_text_copy_stored",
            "clipboardPostCopyMenu",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} must not expose removed post-copy UI/config path; found `{forbidden}`"
            );
        }
    }
}
