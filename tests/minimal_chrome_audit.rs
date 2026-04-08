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
        source.contains("render_minimal_list_prompt_shell_with_footer("),
        "arg prompt should delegate layout to the footer-aware shared minimal list prompt shell"
    );
    assert!(
        source.contains("main_window_footer_slot("),
        "arg prompt should route its GPUI footer through main_window_footer_slot"
    );
    assert!(
        !source.contains("ALPHA_DIVIDER"),
        "arg prompt should not use inline ALPHA_DIVIDER constant for its list divider"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"arg\",\"shell\":\"minimal_list_with_footer\",\"status\":\"pass\"}}");
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

    // Layout must route through the shared expanded-view scaffold
    assert!(
        layout_source.contains("render_expanded_view_scaffold_with_hints(")
            || layout_source.contains("render_expanded_view_scaffold_with_footer(")
            || layout_source.contains("render_expanded_view_scaffold("),
        "clipboard history layout should route through the shared expanded-view scaffold"
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
        source.contains("main_window_footer_slot("),
        "emoji picker should route its GPUI footer through main_window_footer_slot"
    );
    assert!(
        source.contains("render_simple_hint_strip("),
        "emoji picker should build its hint strip via render_simple_hint_strip"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"emoji_picker\",\"shell\":\"native_footer_slot\",\"status\":\"pass\"}}");
}

#[test]
fn file_search_uses_shared_expanded_view_contract() {
    let entry_source = include_str!("../src/render_builtins/file_search.rs");

    // Entry file must declare expanded layout mode
    assert!(
        entry_source.contains("PromptChromeAudit::expanded(\"file_search\""),
        "file search entry should emit PromptChromeAudit::expanded for runtime audit"
    );
    assert!(
        !entry_source.contains("PromptChromeAudit::minimal("),
        "file search entry should not emit a minimal chrome audit"
    );

    // Live source must route through the shared expanded-view scaffold
    assert!(
        entry_source.contains("render_expanded_view_scaffold(")
            || entry_source.contains("render_expanded_view_scaffold_with_hints(")
            || entry_source.contains("render_expanded_view_scaffold_with_footer("),
        "file search should route through the shared expanded-view scaffold"
    );

    // Must use universal hints and emit hint audit
    assert!(
        entry_source.contains("emit_prompt_hint_audit(\"file_search\""),
        "file search should emit prompt hint audit"
    );

    // No old PromptFooter
    assert!(
        !entry_source.contains("PromptFooter::new("),
        "file search should not use PromptFooter"
    );

    // No SectionDivider — expanded view uses spacing, not dividers
    let divider_call = "SectionDivider".to_owned() + "::new()";
    assert!(
        !entry_source.contains(&divider_call),
        "file search should not use SectionDivider — expanded view uses spacing"
    );

    // Legacy layout file must not contain chrome markers
    let layout_source = include_str!("../src/render_builtins/file_search_layout.rs");
    assert!(
        !layout_source.contains("render_minimal_list_prompt_scaffold("),
        "legacy file_search_layout.rs must not contain chrome markers"
    );
    assert!(
        !layout_source.contains("render_expanded_view_scaffold("),
        "legacy file_search_layout.rs must not contain expanded scaffold markers"
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
    let rounded_needle = ".rounded".to_owned() + "(px(8.";
    assert!(
        !source.contains(&rounded_needle),
        "drop prompt should not use rounded corners - whisper chrome means sharp edges"
    );
    assert!(
        !source.contains(".rounded_md()"),
        "drop prompt should not use rounded_md chrome"
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
fn term_prompt_uses_chrome_audit_and_no_hardcoded_wrapper_colors() {
    let source = include_str!("../src/render_prompts/term.rs");

    assert!(
        source.contains("PromptChromeAudit::editor("),
        "term prompt should emit the shared chrome audit contract"
    );
    assert!(
        !source.contains("rgb(0x"),
        "term prompt wrapper should not contain hardcoded hex rgb colors"
    );
    assert!(
        !source.contains("PromptFooter::new("),
        "term prompt should not use PromptFooter"
    );
    assert!(
        source.contains("render_terminal_prompt_hint_strip("),
        "term prompt should use terminal-specific hint strip"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"term\",\"chrome_audit_present\":true,\"hardcoded_hex_absent\":true,\"status\":\"pass\"}}");
}

#[test]
fn chat_prompt_has_chrome_audit_and_custom_footer_exception() {
    let source = include_str!("../src/render_prompts/other.rs");

    assert!(
        source.contains("surface: \"render_prompts::chat\""),
        "chat prompt should declare the shared chrome audit surface tag"
    );
    // Chat owns its footer — verify no duplicate universal footer is forced in
    let chat_fn_start = source
        .find("fn render_chat_prompt(")
        .expect("chat fn exists");
    let chat_fn_end = source[chat_fn_start..]
        .find("\n    fn ")
        .map(|ix| chat_fn_start + ix)
        .unwrap_or(source.len());
    let chat_fn = &source[chat_fn_start..chat_fn_end];
    assert!(
        chat_fn.contains(", None)"),
        "chat prompt should pass None for footer to avoid duplicate"
    );
    assert!(
        chat_fn.contains("footer_mode: \"custom\""),
        "chat prompt should declare custom footer mode in chrome audit"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"chat\",\"chrome_audit_present\":true,\"custom_footer\":true,\"status\":\"pass\"}}");
}

#[test]
fn webcam_prompt_has_chrome_audit_and_no_redundant_chrome() {
    let source = include_str!("../src/render_prompts/other.rs");

    assert!(
        source.contains("PromptChromeAudit::exception("),
        "webcam prompt should emit the shared chrome audit exception"
    );
    let webcam_fn_start = source
        .find("fn render_webcam_prompt(")
        .expect("webcam fn exists");
    let webcam_fn_end = source[webcam_fn_start..]
        .find("\n    fn ")
        .map(|ix| webcam_fn_start + ix)
        .unwrap_or(source.len());
    let webcam_fn = &source[webcam_fn_start..webcam_fn_end];
    assert!(
        !webcam_fn.contains("PromptFooter::new("),
        "webcam prompt should not use PromptFooter"
    );
    assert!(
        webcam_fn.contains("clickable_universal_hint_strip("),
        "webcam prompt should use the clickable hint strip"
    );
    assert!(
        !webcam_fn.contains("rgb(0x"),
        "webcam prompt wrapper should not contain hardcoded hex rgb colors"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"webcam\",\"chrome_audit_present\":true,\"hardcoded_hex_absent\":true,\"status\":\"pass\"}}");
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
