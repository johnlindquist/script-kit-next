const CLIPBOARD_HISTORY_LAYOUT_SOURCE: &str =
    include_str!("../../src/render_builtins/clipboard_history_layout.rs");
const FILE_SEARCH_LAYOUT_SOURCE: &str =
    include_str!("../../src/render_builtins/file_search_layout.rs");
const WINDOW_SWITCHER_SOURCE: &str =
    include_str!("../../src/render_builtins/window_switcher.rs");
const APP_LAUNCHER_SOURCE: &str =
    include_str!("../../src/render_builtins/app_launcher.rs");
const CURRENT_APP_COMMANDS_SOURCE: &str =
    include_str!("../../src/render_builtins/current_app_commands.rs");
const PROCESS_MANAGER_SOURCE: &str =
    include_str!("../../src/render_builtins/process_manager.rs");

fn assert_minimal_builtin_surface(name: &str, source: &str) {
    let prompt_footer_needle = ["PromptFooter", "::new("].concat();

    assert!(
        source.contains("render_simple_hint_strip(") || source.contains("HintStrip::new("),
        "{name} should render a minimal hint strip footer"
    );

    assert!(
        !source.contains(&prompt_footer_needle),
        "{name} should not use PromptFooter::new"
    );

    assert!(
        source.contains("HEADER_PADDING_X") && source.contains("HEADER_PADDING_Y"),
        "{name} should use shared chrome header padding tokens"
    );

    assert!(
        source.contains("SectionDivider::new()")
            || source.contains("border_t(px(DIVIDER_HEIGHT))")
            || source.contains("border_b(px(DIVIDER_HEIGHT))"),
        "{name} should use the shared minimal divider contract"
    );
}

#[test]
fn clipboard_history_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("clipboard_history_layout", CLIPBOARD_HISTORY_LAYOUT_SOURCE);
}

#[test]
fn file_search_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("file_search_layout", FILE_SEARCH_LAYOUT_SOURCE);
}

#[test]
fn window_switcher_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("window_switcher", WINDOW_SWITCHER_SOURCE);
}

#[test]
fn app_launcher_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("app_launcher", APP_LAUNCHER_SOURCE);
}

#[test]
fn current_app_commands_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("current_app_commands", CURRENT_APP_COMMANDS_SOURCE);
}

#[test]
fn process_manager_uses_minimal_prompt_chrome() {
    assert_minimal_builtin_surface("process_manager", PROCESS_MANAGER_SOURCE);
}
