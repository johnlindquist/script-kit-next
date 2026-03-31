//! Contract tests for quick terminal mouse-wheel scrolling and modern interactions.
//!
//! These source-contract tests lock the presence of wheel scrolling, text selection,
//! copy/paste, and scroll indicator behavior in the quick terminal (TermPrompt).

const TERM_PROMPT_SOURCE: &str = include_str!("../src/term_prompt/mod.rs");

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
