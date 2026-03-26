//! Source-audit tests for the minimal chrome migration.
//!
//! These tests verify that migrated prompt surfaces use the shared hint-strip
//! footer and chrome-token dividers instead of the old PromptFooter component.

#[test]
fn arg_prompt_uses_shared_chrome_divider() {
    let source = include_str!("../src/render_prompts/arg/render.rs");
    assert!(
        source.contains("SectionDivider::new()")
            || (source.contains("DIVIDER_HEIGHT") && source.contains("DIVIDER_OPACITY")),
        "arg prompt divider should use the shared chrome contract"
    );
    assert!(
        !source.contains("ALPHA_DIVIDER"),
        "arg prompt should not use inline ALPHA_DIVIDER constant for its list divider"
    );
}

#[test]
fn clipboard_history_no_longer_uses_prompt_footer() {
    let source = include_str!("../src/render_builtins/clipboard.rs");
    assert!(
        !source.contains("PromptFooter::new("),
        "clipboard history should use hint strip, not PromptFooter"
    );
    assert!(
        source.contains("render_simple_hint_strip("),
        "clipboard history should render a minimal hint strip footer"
    );
}

#[test]
fn emoji_picker_no_longer_uses_prompt_footer() {
    let source = include_str!("../src/render_builtins/emoji_picker.rs");
    assert!(
        !source.contains("PromptFooter::new("),
        "emoji picker should use hint strip, not PromptFooter"
    );
    assert!(
        source.contains("render_simple_hint_strip("),
        "emoji picker should render a minimal hint strip footer"
    );
}

#[test]
fn file_search_no_longer_uses_prompt_footer() {
    let source = include_str!("../src/render_builtins/file_search.rs");
    assert!(
        !source.contains("PromptFooter::new("),
        "file search should use hint strip, not PromptFooter"
    );
    assert!(
        source.contains("render_simple_hint_strip("),
        "file search should render a minimal hint strip footer"
    );
}
