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

#[test]
fn editor_choice_popup_has_no_box_card_chrome() {
    let source = include_str!("../src/editor/mod.rs");
    assert!(
        !source.contains(".rounded_md()"),
        "editor choice popup should not use rounded card chrome"
    );
    assert!(
        !source.contains(".border_1()"),
        "editor choice popup should not use bordered card chrome"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"editor_choice_popup\",\"border_absent\":true,\"rounded_absent\":true,\"status\":\"pass\"}}");
}

#[test]
fn form_prompt_wrapper_has_no_prompt_footer_or_hardcoded_hex() {
    let source = include_str!("../src/render_prompts/form/render.rs");
    assert!(
        !source.contains("PromptFooter::new("),
        "form prompt wrapper should not use PromptFooter"
    );
    assert!(
        !source.contains("rgb(0x"),
        "form prompt wrapper should not contain hardcoded hex rgb colors"
    );
    assert!(
        !source.contains("rgba(0x"),
        "form prompt wrapper should not contain hardcoded hex rgba colors"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"form_prompt\",\"footer_absent\":true,\"hardcoded_hex_absent\":true,\"status\":\"pass\"}}");
}

#[test]
fn path_prompt_render_has_tracing_and_no_hardcoded_hex() {
    let outer_source = include_str!("../src/render_prompts/path.rs");
    let inner_source = include_str!("../src/prompts/path/render.rs");

    assert!(
        outer_source.contains("tracing::info!("),
        "path prompt outer render should emit tracing::info"
    );
    assert!(
        !outer_source.contains("rgb(0x"),
        "path prompt outer render should not contain hardcoded hex rgb colors"
    );
    assert!(
        !inner_source.contains("PromptFooter::new("),
        "path prompt inner render should not use PromptFooter"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"path\",\"tracing_present\":true,\"hardcoded_hex_absent\":true,\"status\":\"pass\"}}");
}

#[test]
fn template_prompt_render_has_tracing_and_uses_shared_helpers() {
    let source = include_str!("../src/prompts/template/render.rs");
    let outer_source = include_str!("../src/render_prompts/other.rs");

    assert!(
        outer_source.contains("surface = \"render_prompts::template\""),
        "template prompt outer render should emit tracing with surface tag"
    );
    assert!(
        source.contains("prompt_field_style("),
        "template prompt should use shared prompt_field_style"
    );
    assert!(
        !source.contains("rgb(0x"),
        "template prompt should not contain hardcoded hex rgb colors"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"template\",\"tracing_present\":true,\"shared_helpers\":true,\"status\":\"pass\"}}");
}

#[test]
fn drop_prompt_uses_whisper_chrome_not_heavy_borders() {
    let source = include_str!("../src/prompts/drop.rs");

    assert!(
        !source.contains(".border_2()"),
        "drop prompt should not use heavy border_2 chrome"
    );
    assert!(
        !source.contains(".rounded(px(8."))
            && !source.contains(".rounded_md()")
            && !source.contains(".rounded_lg()"),
        "drop prompt should not use rounded corners (whisper chrome = sharp edges)"
    );
    assert!(
        source.contains("OPACITY_GHOST"),
        "drop prompt should use ghost opacity constants for whisper chrome"
    );
    let outer_source = include_str!("../src/render_prompts/other.rs");
    assert!(
        outer_source.contains("surface = \"render_prompts::drop\""),
        "drop prompt outer render should emit tracing with surface tag"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"drop\",\"whisper_chrome\":true,\"tracing_present\":true,\"status\":\"pass\"}}");
}

#[test]
fn naming_prompt_render_has_tracing_and_uses_shared_helpers() {
    let source = include_str!("../src/prompts/naming/render.rs");
    let outer_source = include_str!("../src/render_prompts/other.rs");

    assert!(
        outer_source.contains("surface = \"render_prompts::naming\""),
        "naming prompt outer render should emit tracing with surface tag"
    );
    assert!(
        source.contains("prompt_form_intro("),
        "naming prompt should use shared prompt_form_intro"
    );
    assert!(
        source.contains("prompt_form_section("),
        "naming prompt should use shared prompt_form_section"
    );
    assert!(
        !source.contains("rgb(0x"),
        "naming prompt should not contain hardcoded hex rgb colors"
    );
    assert!(
        !source.contains("PromptFooter::new("),
        "naming prompt should not use PromptFooter"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"naming\",\"tracing_present\":true,\"shared_helpers\":true,\"status\":\"pass\"}}");
}
