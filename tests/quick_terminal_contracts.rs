//! Contract tests for quick terminal mouse-wheel scrolling and modern interactions.
//!
//! These source-contract tests lock the presence of wheel scrolling, text selection,
//! copy/paste, and scroll indicator behavior in the quick terminal (TermPrompt).

const TERM_PROMPT_SOURCE: &str = include_str!("../src/term_prompt/mod.rs");
const TERMINAL_CREATION_SOURCE: &str = include_str!("../src/terminal/alacritty/handle_creation.rs");
const QUICK_TERMINAL_WARM_SOURCE: &str = include_str!("../src/app_impl/quick_terminal_warm.rs");
const THEME_FOCUS_SOURCE: &str = include_str!("../src/app_impl/theme_focus.rs");
const THEME_CHOOSER_SOURCE: &str = include_str!("../src/render_builtins/theme_chooser.rs");

#[test]
fn quick_terminal_mouse_wheel_and_modern_interaction_contract() {
    // --- Mouse wheel scrolling ---
    assert!(
        TERM_PROMPT_SOURCE.contains(".on_scroll_wheel("),
        "TermPrompt must register a mouse-wheel handler"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.scroll(whole_lines);"),
        "mouse-wheel handler must forward wheel movement into terminal scrollback"
    );
    // The scroll handler must trigger a view refresh after scrolling
    assert!(
        TERM_PROMPT_SOURCE.contains("cx.notify();"),
        "mouse-wheel handler must notify the view after scrolling"
    );

    // --- Copy / paste ---
    assert!(
        TERM_PROMPT_SOURCE.contains("if has_meta && key_str == \"c\""),
        "Cmd+C copy/SIGINT behavior must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("if has_meta && key_str == \"v\""),
        "Cmd+V paste behavior must remain implemented"
    );

    // --- Mouse text selection ---
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_selection(col, row);"),
        "single-click drag selection must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_semantic_selection(col, row);"),
        "double-click word selection must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_line_selection(col, row);"),
        "triple-click line selection must remain implemented"
    );

    // --- Scroll position indicator ---
    assert!(
        TERM_PROMPT_SOURCE.contains("let scroll_offset = self.terminal.display_offset();"),
        "scrollback indicator must remain implemented"
    );
}

#[test]
fn quick_terminal_theme_respects_light_dark_contract() {
    assert!(
        TERMINAL_CREATION_SOURCE.contains("pub fn new_with_theme("),
        "TerminalHandle must expose themed shell creation"
    );
    assert!(
        TERMINAL_CREATION_SOURCE.contains("pub fn with_command_and_theme("),
        "TerminalHandle must expose themed command creation"
    );
    assert!(
        TERMINAL_CREATION_SOURCE.contains(".map(ThemeAdapter::from_theme)"),
        "terminal creation must build its adapter from the active Script Kit theme"
    );

    assert!(
        TERM_PROMPT_SOURCE.contains("TerminalHandle::new_with_theme("),
        "TermPrompt must pass its theme into new terminal creation"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("TerminalHandle::with_command_and_theme("),
        "TermPrompt must pass its theme into command terminal creation"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("terminal.update_theme(&theme);"),
        "warm PTYs attached to TermPrompt must be rethemed before render"
    );
    assert!(
        QUICK_TERMINAL_WARM_SOURCE.contains("TerminalHandle::new_with_theme("),
        "Quick Terminal warm PTYs must be prewarmed with the current theme"
    );
    assert!(
        THEME_FOCUS_SOURCE.contains("pub(crate) fn sync_open_terminal_theme("),
        "theme changes must have a terminal propagation helper"
    );
    assert!(
        THEME_CHOOSER_SOURCE.contains("self.sync_open_terminal_theme(cx);"),
        "theme chooser previews and restores must propagate to an open terminal"
    );
}
