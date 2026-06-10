use std::fs;

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("section start must exist");
    let tail = &source[start_idx..];
    let end_idx = tail
        .find(end)
        .map(|idx| start_idx + idx)
        .unwrap_or(source.len());
    &source[start_idx..end_idx]
}

#[test]
fn agent_chat_empty_guidance_uses_shared_info_state() {
    let source = fs::read_to_string("src/ai/agent_chat/ui/view.rs")
        .expect("failed to read src/ai/agent_chat/ui/view.rs");

    assert!(
        source.contains("render_agent_chat_empty_guidance"),
        "Agent Chat empty composer must use shared InfoState guidance"
    );
    assert!(
        !source.contains("Type / for skills"),
        "old weak Agent Chat empty hint copy must not return"
    );
}

#[test]
fn shared_info_state_is_exported() {
    let source =
        fs::read_to_string("src/components/mod.rs").expect("failed to read src/components/mod.rs");

    assert!(source.contains("mod info_state"));
    assert!(source.contains("render_info_state"));
    assert!(source.contains("InfoStateSpec"));
}

#[test]
fn info_state_keeps_context_first_agent_chat_copy() {
    let source = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");

    assert!(source.contains("Ask with context"));
    assert!(source.contains("Use / for skills or @ to attach context"));
    assert!(source.contains("Attach files, scripts, clipboard, or history"));
    assert!(source.contains("InfoGuidanceItem::new(Some(\"⌘K\"), \"Show every chat action\")"));
    let agent_chat_spec = section_between(
        &source,
        "pub(crate) fn agent_chat_empty_guidance_spec",
        "pub(crate) fn render_agent_chat_empty_guidance",
    );
    assert!(
        !agent_chat_spec.contains(".footer_shortcut_note("),
        "Agent Chat empty guidance must keep Cmd+K in the normal row list so spacing stays consistent"
    );
    assert!(!source.contains("⌘N new"));
    assert!(!source.contains("⌘W close"));
}

#[test]
fn composer_empty_info_state_uses_main_view_columns_not_centered_card() {
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");

    assert!(
        !info.contains("InfoStateLayout::Centered | InfoStateLayout::ComposerEmpty"),
        "ComposerEmpty must not alias the old centered narrow help-card branch"
    );
    assert!(info.contains("InfoStateLayout::MainViewColumns"));
    assert!(info.contains("main_view_content_columns(def)"));
    assert!(info.contains(".pl(px(cols.text_column_x))"));
    assert!(info.contains(".pr(px(cols.content_right_inset_x))"));
    assert!(info.contains(".pt(px(cols.top_inset_y))"));
    assert!(
        info.contains(
            "render_info_content(&spec, theme, palette, metrics, !uses_main_view_columns)"
        ),
        "main-view info layouts should not keep the old fixed max-width cap"
    );
}

#[test]
fn agent_chat_empty_guidance_uses_comfortable_main_view_density() {
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");
    let spec = section_between(
        &info,
        "pub(crate) fn agent_chat_empty_guidance_spec",
        "pub(crate) fn render_agent_chat_empty_guidance",
    );

    assert!(spec.contains(".layout(InfoStateLayout::ComposerEmpty)"));
    assert!(spec.contains(".density(InfoStateDensity::Comfortable)"));
}

#[test]
fn agent_chat_empty_guidance_slot_does_not_center_the_info_state() {
    let agent_chat = fs::read_to_string("src/ai/agent_chat/ui/view.rs")
        .expect("failed to read src/ai/agent_chat/ui/view.rs");
    let middle = section_between(
        &agent_chat,
        "fn render_agent_chat_middle_area",
        "if show_sidecar",
    );

    assert!(middle.contains("render_agent_chat_empty_guidance"));
    assert!(
        !middle.contains(".items_center()") && !middle.contains(".justify_center()"),
        "AgentChat middle area must not center ComposerEmpty; InfoState owns that layout"
    );
    assert!(middle.contains(".w_full()"));
    assert!(middle.contains(".h_full()"));
}

#[test]
fn info_guidance_shortcuts_use_footer_keycaps_not_hint_strip() {
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");
    let guidance_row = section_between(&info, "fn render_guidance_row", "\n\n#[cfg(test)]");

    assert!(
        guidance_row.contains("crate::components::footer_chrome::render_footer_shortcut_keycaps"),
        "InfoState guidance shortcuts must render through footer keycaps"
    );
    assert!(
        info.contains(
            "crate::components::footer_chrome::footer_shortcut_keycaps_measured_width_px"
        ),
        "InfoState guidance shortcut column must size from measured footer keycap widths"
    );
    assert!(
        !guidance_row.contains("crate::components::hint_strip::render_inline_shortcut_keys")
            && !guidance_row.contains("shortcut_tokens_from_hint")
            && !guidance_row.contains("whisper_inline_shortcut_colors"),
        "InfoState guidance shortcuts must not regress to hint_strip inline shortcut styling"
    );
    assert!(
        !guidance_row.contains(".child(shortcut)")
            && !guidance_row.contains("child(shortcut.to_string())")
            && !guidance_row.contains("SharedString::from(shortcut)"),
        "InfoState guidance shortcuts must not regress to raw shortcut text"
    );
}

#[test]
fn info_footer_shortcut_notes_use_footer_keycaps_not_raw_text() {
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");
    let note_renderer = section_between(
        &info,
        "fn render_info_shortcut_note",
        "\nfn render_info_section",
    );

    assert!(
        info.contains("InfoShortcutNote") && info.contains("pub(crate) fn footer_shortcut_note("),
        "shortcut-bearing help notes should be modeled as shortcut plus text, not one raw string"
    );
    assert!(
        note_renderer.contains("crate::components::footer_chrome::render_footer_shortcut_keycaps"),
        "InfoState footer shortcut notes must render through footer keycaps"
    );
    assert!(
        note_renderer.contains("render_info_guidance_text(note.text, None, palette)"),
        "InfoState footer shortcut notes must share guidance row text styling instead of using a separate dim footer-note style"
    );
    assert!(
        !note_renderer.contains(".text_color(palette.hint)"),
        "shortcut-bearing help notes should not render as dim footer-note text when they visually behave like guidance rows"
    );
    assert!(
        !note_renderer.contains(".child(note.shortcut)")
            && !note_renderer.contains(".child(note.shortcut.to_string())")
            && !info.contains(".footer_note(\"⌘K shows every chat action.\")"),
        "InfoState footer shortcut notes must not regress to raw shortcut text"
    );
}

#[test]
fn footer_chrome_exposes_only_shared_shortcut_keycap_row_for_help_surfaces() {
    let footer = fs::read_to_string("src/components/footer_chrome.rs")
        .expect("failed to read src/components/footer_chrome.rs");
    let shortcut_renderer = section_between(
        &footer,
        "pub(crate) fn render_footer_shortcut_keycaps",
        "fn render_footer_keycap",
    );

    assert!(
        shortcut_renderer.contains("split_footer_shortcut(&shortcut)")
            && shortcut_renderer.contains("render_footer_shortcut_keycaps_from_tokens"),
        "shared footer shortcut keycap renderer must keep using the footer parser and token helper"
    );
    assert!(
        shortcut_renderer.contains("render_footer_keycap_with_metrics")
            && shortcut_renderer.contains("token.to_string()")
            && shortcut_renderer.contains("keycap_font_size_px")
            && shortcut_renderer.contains("keycap_height_px"),
        "shared footer shortcut token helper must keep using the footer keycap primitive"
    );
    assert!(
        footer.contains("fn render_footer_keycap("),
        "low-level footer keycap primitive should remain private; expose the row renderer only"
    );
}

#[test]
fn launcher_empty_guidance_uses_shared_info_state() {
    let launcher = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("failed to read src/render_script_list/mod.rs");
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");

    assert!(launcher.contains("render_launcher_empty_or_no_results"));
    assert!(info.contains("launcher_empty_or_no_results_spec"));
    assert!(info.contains("No scripts yet"));
    assert!(info.contains("Tags need a syntax prefix"));
    assert!(info.contains("active filter is narrowing"));
    assert!(info.contains("scripts, scriptlets, snippets, and built-in commands"));
    assert!(!launcher.contains("No scripts or snippets found"));
    assert!(!launcher.contains("Press ⌘N to create a new script"));
}
