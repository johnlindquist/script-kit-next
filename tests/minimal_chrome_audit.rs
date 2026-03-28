//! Source-audit tests for the minimal chrome migration.
//!
//! These tests verify that migrated prompt surfaces use the correct shared
//! layout contract:
//! - Minimal-list surfaces: shared scaffold/shell with hint strip footer
//! - Expanded-view surfaces: shared expanded scaffold with no dividers
//!
//! Clipboard history and file search are expanded-view surfaces per
//! `.impeccable.md` (preview IS the decision — users can't pick without
//! seeing the content).

#[test]
fn arg_prompt_uses_shared_minimal_list_shell() {
    let source = include_str!("../src/render_prompts/arg/render.rs");
    assert!(
        source.contains("render_minimal_list_prompt_shell("),
        "arg prompt should delegate layout to the shared minimal list prompt shell"
    );
    assert!(
        !source.contains("ALPHA_DIVIDER"),
        "arg prompt should not use inline ALPHA_DIVIDER constant for its list divider"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"arg\",\"shell\":\"minimal_list\",\"status\":\"pass\"}}");
}

#[test]
fn clipboard_history_uses_shared_expanded_view_contract() {
    let entry_source = include_str!("../src/render_builtins/clipboard.rs");
    let layout_source = include_str!("../src/render_builtins/clipboard_history_layout.rs");

    // Entry file must declare expanded layout mode
    assert!(
        entry_source.contains("PromptChromeAudit::expanded(\"clipboard_history\""),
        "clipboard history entry should emit PromptChromeAudit::expanded for runtime audit"
    );
    assert!(
        !entry_source.contains("PromptChromeAudit::minimal("),
        "clipboard history entry should not emit a minimal chrome audit"
    );

    // Layout must not use old PromptFooter
    assert!(
        !layout_source.contains("PromptFooter::new("),
        "clipboard history layout should not use PromptFooter"
    );

    // Layout uses shared hint strip and header tokens (manual expanded layout)
    assert!(
        layout_source.contains("render_simple_hint_strip("),
        "clipboard history layout should use the shared hint strip footer"
    );

    // No SectionDivider — expanded view uses spacing, not dividers
    let divider_call = "SectionDivider".to_owned() + "::new()";
    assert!(
        !layout_source.contains(&divider_call),
        "clipboard history layout should not use SectionDivider — expanded view uses spacing"
    );

    eprintln!("{{\"audit\":\"expanded_contract\",\"surface\":\"clipboard_history\",\"layout_mode\":\"expanded\",\"divider_absent\":true,\"footer_shared\":true,\"status\":\"pass\"}}");
}

#[test]
fn emoji_picker_no_longer_uses_prompt_footer() {
    let source = include_str!("../src/render_builtins/emoji_picker.rs");
    assert!(
        !source.contains("PromptFooter::new("),
        "emoji picker should use hint strip, not PromptFooter"
    );
    assert!(
        source.contains("render_minimal_list_prompt_scaffold("),
        "emoji picker should use the shared minimal list prompt scaffold"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"emoji_picker\",\"shell\":\"minimal_scaffold\",\"status\":\"pass\"}}");
}

#[test]
fn file_search_uses_shared_expanded_view_contract() {
    let entry_source = include_str!("../src/render_builtins/file_search.rs");
    let layout_source = include_str!("../src/render_builtins/file_search_layout.rs");

    // Entry file must declare expanded layout mode
    assert!(
        entry_source.contains("PromptChromeAudit::expanded(\"file_search\""),
        "file search entry should emit PromptChromeAudit::expanded for runtime audit"
    );
    assert!(
        !entry_source.contains("PromptChromeAudit::minimal("),
        "file search entry should not emit a minimal chrome audit"
    );

    // Layout must route through the shared expanded-view scaffold
    assert!(
        layout_source.contains("render_expanded_view_scaffold("),
        "file search layout should route through the shared expanded-view scaffold"
    );

    // No old PromptFooter
    assert!(
        !layout_source.contains("PromptFooter::new("),
        "file search layout should not use PromptFooter"
    );

    // No SectionDivider — expanded view uses spacing, not dividers
    let divider_call = "SectionDivider".to_owned() + "::new()";
    assert!(
        !layout_source.contains(&divider_call),
        "file search layout should not use SectionDivider — expanded view uses spacing"
    );

    eprintln!("{{\"audit\":\"expanded_contract\",\"surface\":\"file_search\",\"scaffold_used\":true,\"layout_mode\":\"expanded\",\"divider_absent\":true,\"status\":\"pass\"}}");
}
