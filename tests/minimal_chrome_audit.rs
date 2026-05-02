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
    let start = source
        .find("fn render_choices_popup")
        .expect("render_choices_popup exists");
    let popup_source = &source[start..];
    assert!(
        !popup_source.contains(".rounded_md()"),
        "editor choice popup should not use rounded card chrome"
    );
    assert!(
        !popup_source.contains(".border_1()"),
        "editor choice popup should not use bordered card chrome"
    );
    assert!(
        popup_source.contains("AppChromeColors::from_theme")
            && popup_source.contains("inline_dropdown_surface_rgba")
            && popup_source.contains("selection_rgba")
            && popup_source.contains("text_primary_hex"),
        "editor choice popup should resolve inline dropdown chrome through AppChromeColors"
    );
    assert!(
        !popup_source.contains("colors.background.main.to_rgb()")
            && !popup_source.contains("colors.accent.selected")
            && !popup_source.contains("colors.text.on_accent"),
        "editor choice popup should avoid raw background/accent/on-accent color paths"
    );
    assert!(
        !popup_source.contains(".cursor_pointer()") || popup_source.contains(".on_mouse_down("),
        "pointer cursor should only be used when choice rows are actually clickable"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"editor_choice_popup\",\"border_absent\":true,\"rounded_absent\":true,\"chrome_tokens\":true,\"status\":\"pass\"}}");
}

#[test]
fn editor_footer_contract_is_single_source() {
    let editor_source = include_str!("../src/editor/mod.rs");
    let wrapper_source = include_str!("../src/render_prompts/editor.rs");
    assert!(
        !editor_source.contains("PromptFooter"),
        "EditorPrompt should not reference PromptFooter; footer ownership belongs to the wrapper"
    );
    assert!(
        !wrapper_source.contains("PromptFooter::new("),
        "editor wrapper should not instantiate PromptFooter"
    );
    assert!(
        wrapper_source.contains("main_window_footer_slot(self.clickable_universal_hint_strip(cx))"),
        "editor wrapper should route universal hints through main_window_footer_slot"
    );
    assert!(
        !wrapper_source.contains("40px fixed height"),
        "editor wrapper comments should not hard-code stale footer height"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"editor_footer\",\"single_source\":true,\"status\":\"pass\"}}");
}

#[test]
fn editor_layout_info_has_editor_content_branch() {
    let source = include_str!("../src/app_layout/build_layout_info.rs");
    assert!(
        source.contains("AppView::EditorPrompt")
            && source.contains("EditorContent")
            && source.contains("LayoutComponentType::Prompt"),
        "build_layout_info should expose an EditorContent prompt branch"
    );
    assert!(
        source.contains("return LayoutInfo"),
        "editor layout branch should return before adding main-menu ScriptList/PreviewPanel components"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"editor_layout_info\",\"editor_content_branch\":true,\"status\":\"pass\"}}");
}

#[test]
fn terminal_layout_info_has_terminal_content_branch() {
    let source = include_str!("../src/app_layout/build_layout_info.rs");
    let branch_start = source
        .find("AppView::EditorPrompt")
        .expect("prompt layout branch exists");
    let prompt_branch = &source[branch_start..];
    assert!(
        prompt_branch.contains("AppView::TermPrompt")
            && prompt_branch.contains("AppView::QuickTerminalView")
            && prompt_branch.contains("TerminalContent")
            && prompt_branch.contains("LayoutComponentType::Prompt"),
        "terminal layout info should expose TerminalContent for SDK and quick terminal views"
    );
    assert!(
        prompt_branch.contains("return LayoutInfo"),
        "terminal layout branch should return before adding launcher list/preview components"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"terminal_layout_info\",\"terminal_content_branch\":true,\"status\":\"pass\"}}");
}

#[test]
fn select_drop_layout_info_has_prompt_owned_branches() {
    let source = include_str!("../src/app_layout/build_layout_info.rs");
    assert!(
        source.contains("AppView::SelectPrompt")
            && source.contains("SelectChoices")
            && source.contains("LayoutComponentType::List"),
        "select layout info should report its prompt-owned list instead of launcher list/preview"
    );
    assert!(
        source.contains("AppView::DropPrompt")
            && source.contains("DropContent")
            && source.contains("LayoutComponentType::Prompt"),
        "drop layout info should report prompt-owned drop content instead of launcher list/preview"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"select_drop_layout_info\",\"prompt_owned_branches\":true,\"status\":\"pass\"}}");
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
fn form_prompt_fields_use_shared_chrome_and_focus_contracts() {
    let form_source = include_str!("../src/form_prompt.rs");
    let colors_source = include_str!("../src/components/form_fields/colors.rs");
    let text_field_source = include_str!("../src/components/form_fields/text_field/render.rs");
    let text_area_source = include_str!("../src/components/form_fields/text_area/render.rs");
    let text_field_model = include_str!("../src/components/form_fields/text_field/mod.rs");

    assert!(
        colors_source.contains("AppChromeColors::from_theme")
            && colors_source.contains("placeholder_text_rgba"),
        "form field colors should resolve text through shared chrome tokens"
    );
    assert!(
        !colors_source.contains("theme.colors.text.muted")
            && !colors_source.contains("theme.colors.text.secondary"),
        "form fields should not double-dim from muted/secondary base text colors"
    );
    assert!(
        !text_field_source.contains("rgb(colors.text")
            && !text_field_source.contains("rgb(colors.placeholder")
            && !text_field_source.contains("rgb(colors.label")
            && !text_area_source.contains("rgb(colors.text")
            && !text_area_source.contains("rgb(colors.placeholder")
            && !text_area_source.contains("rgb(colors.label"),
        "form field renderers should consume resolved Rgba text colors directly"
    );
    assert!(
        text_area_source.contains("cursor_element")
            && text_area_source.contains("self.cursor_position")
            && text_area_source.contains("slice_by_char_range"),
        "textarea should render a visible char-safe cursor"
    );
    assert!(
        form_source.contains("to_ascii_lowercase()")
            && text_field_model.contains("eq_ignore_ascii_case(\"password\")"),
        "form field type dispatch and password masking should be case-insensitive"
    );
    assert!(
        form_source.contains("focus_field_at")
            && form_source.contains("form-field-slot-")
            && form_source.contains("self.focused_index = index.min"),
        "form prompt field clicks should synchronize parent focused_index"
    );
    assert!(
        !text_field_model.contains("self.value.insert_str(self.cursor_position")
            && !text_field_model.contains("self.value.remove(self.cursor_position)"),
        "text field code must not treat char indices as byte indices"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"form_fields\",\"chrome_tokens\":true,\"cursor_visible\":true,\"focus_sync\":true,\"status\":\"pass\"}}");
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
    assert!(
        inner_source.contains("prompt_text_palette("),
        "path prompt inner render should use shared prompt text palette"
    );
    assert!(
        !inner_source.contains("<< 8")
            && !inner_source.contains("0x99")
            && !inner_source.contains("0xCC"),
        "path prompt inner render should not build local packed-alpha text colors"
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
        source.contains("prompt_text_palette("),
        "template prompt should use shared prompt_text_palette"
    );
    assert!(
        source.contains("template-fields-scroll") && source.contains(".overflow_y_scrollbar()"),
        "template prompt should keep placeholder fields in a scrollable region"
    );
    assert!(
        outer_source.contains("template_prompt_hints()")
            && outer_source.contains("emit_surface_prompt_hint_audit("),
        "template prompt should use truthful surface-specific footer hints"
    );
    let ui_window_source = include_str!("../src/app_impl/ui_window.rs");
    assert!(
        ui_window_source.contains("AppView::TemplatePrompt")
            && ui_window_source
                .contains("FooterButtonConfig::new(FooterAction::Run, \"↵\", \"Submit\")")
            && ui_window_source
                .contains("FooterButtonConfig::new(FooterAction::Ai, \"⇥\", \"Next Field\")"),
        "native template footer should advertise Submit and Next Field instead of Run/AI"
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
    assert!(
        source.contains("\"native_footer_spacer\"")
            && source.contains("\"custom_hint_strip\"")
            && source.contains("\"quick_terminal_uses_native_footer\"")
            && source.contains("\"terminal_owns_contextual_footer\""),
        "term prompt chrome audit should distinguish SDK terminal and Quick Terminal footer ownership"
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
        webcam_fn.contains("clickable_webcam_hint_strip("),
        "webcam prompt should use the capture-specific clickable hint strip"
    );
    assert!(
        webcam_fn.contains("universal_prompt_hints_with_primary_label(\"Capture Photo\")")
            && webcam_fn.contains("emit_surface_prompt_hint_audit("),
        "webcam prompt should advertise Capture Photo through surface-specific hint auditing"
    );
    assert!(
        !webcam_fn.contains("rgb(0x"),
        "webcam prompt wrapper should not contain hardcoded hex rgb colors"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"webcam\",\"chrome_audit_present\":true,\"hardcoded_hex_absent\":true,\"status\":\"pass\"}}");
}

#[test]
fn mic_protocol_stays_stubbed_until_media_surface_exists() {
    let execute_script = include_str!("../src/execute_script/mod.rs");
    let prompt_messages = include_str!("../src/main_sections/prompt_messages.rs");
    let prompt_handler = include_str!("../src/prompt_handler/mod.rs");
    let app_view_state = include_str!("../src/main_sections/app_view_state.rs");
    let ui_window = include_str!("../src/app_impl/ui_window.rs");
    let micro_render = include_str!("../src/render_prompts/micro.rs");
    let micro_fn_start = micro_render
        .find("fn render_micro_prompt(")
        .expect("micro render fn exists");
    let micro_fn_end = micro_render[micro_fn_start..]
        .find("\n    fn ")
        .map(|ix| micro_fn_start + ix)
        .unwrap_or_else(|| {
            micro_render[micro_fn_start..]
                .find("\n#[cfg(test)]")
                .map(|ix| micro_fn_start + ix)
                .unwrap_or(micro_render.len())
        });
    let micro_fn = &micro_render[micro_fn_start..micro_fn_end];

    assert!(
        execute_script.contains("Message::Mic { id }")
            && execute_script.contains("Some(PromptMessage::MicComingSoon { id })"),
        "SDK mic() should stay routed to the explicit coming-soon prompt message until a real media surface exists"
    );
    assert!(
        prompt_messages.contains("MicComingSoon"),
        "mic() should keep a named stub message instead of borrowing the micro prompt"
    );
    assert!(
        prompt_handler.contains("self.show_prompt_coming_soon_toast(\"mic()\", cx)"),
        "mic() should surface as coming-soon feedback rather than launcher footer chrome"
    );
    assert!(
        !app_view_state.contains("MicView") && !ui_window.contains("mic_prompt"),
        "mic() must not expose native-footer surface metadata until a real MicView is implemented"
    );
    assert!(
        micro_fn.contains("\"render_prompts::micro\"")
            && micro_fn.contains("\"ultra_compact_no_footer\"")
            && !micro_fn.contains("PromptFooter::new("),
        "micro() is the ultra-compact text prompt and should not be repurposed as microphone UI"
    );
    eprintln!("{{\"audit\":\"minimal_chrome\",\"surface\":\"mic_stub\",\"mic_coming_soon\":true,\"micro_prompt_distinct\":true,\"status\":\"pass\"}}");
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
