// --- Expanded-view surfaces (preview IS the decision) ---
const CLIPBOARD_HISTORY_ENTRY_SOURCE: &str = include_str!("../../src/render_builtins/clipboard.rs");
const CLIPBOARD_HISTORY_LAYOUT_SOURCE: &str =
    include_str!("../../src/render_builtins/clipboard_history_layout.rs");
const FILE_SEARCH_ENTRY_SOURCE: &str = include_str!("../../src/render_builtins/file_search.rs");
const FILE_SEARCH_LAYOUT_SOURCE: &str =
    include_str!("../../src/render_builtins/file_search_layout.rs");

// --- Minimal-list surfaces (name IS the content) ---
const WINDOW_SWITCHER_SOURCE: &str = include_str!("../../src/render_builtins/window_switcher.rs");
const APP_LAUNCHER_SOURCE: &str = include_str!("../../src/render_builtins/app_launcher.rs");
const CURRENT_APP_COMMANDS_SOURCE: &str =
    include_str!("../../src/render_builtins/current_app_commands.rs");
const PROCESS_MANAGER_SOURCE: &str = include_str!("../../src/render_builtins/process_manager.rs");

/// Assert that a minimal-list builtin surface uses shared chrome infrastructure.
///
/// Minimal surfaces render name-is-the-content lists and use scaffolds or
/// manual chrome tokens (hint strip, header padding, section dividers).
fn assert_minimal_builtin_surface(name: &str, source: &str) {
    let prompt_footer_needle = ["PromptFooter", "::new("].concat();

    assert!(
        source.contains("render_simple_hint_strip(")
            || source.contains("HintStrip::new(")
            || source.contains("render_minimal_list_prompt_scaffold(")
            || source.contains("render_minimal_list_prompt_shell("),
        "{name} should use a shared minimal chrome helper (hint strip or minimal scaffold)"
    );

    assert!(
        !source.contains(&prompt_footer_needle),
        "{name} should not use PromptFooter::new"
    );

    let uses_scaffold = source.contains("render_minimal_list_prompt_scaffold(")
        || source.contains("render_minimal_list_prompt_shell(");

    if !uses_scaffold {
        assert!(
            source.contains("HEADER_PADDING_X") && source.contains("HEADER_PADDING_Y"),
            "{name} should use shared chrome header padding tokens"
        );

        assert!(
            source.contains("SectionDivider::new()")
                || source.contains("border_t(px(DIVIDER_HEIGHT))")
                || source.contains("border_b(px(DIVIDER_HEIGHT))"),
            "{name} should use shared minimal chrome structure tokens"
        );
    }
}

/// Assert the expanded-view contract for preview-driven builtins.
///
/// Expanded surfaces show a list + preview split where users cannot
/// confidently select without seeing the content. Per .impeccable.md:
/// - Must use the shared expanded-view scaffold or shell
/// - Must NOT use SectionDivider (spacing defines structure)
/// - Must NOT use PromptFooter (scaffold owns the footer)
/// - Entry file must emit PromptChromeAudit::expanded
fn assert_expanded_builtin_surface(name: &str, entry_source: &str, layout_source: &str) {
    let prompt_footer_needle = ["PromptFooter", "::new("].concat();
    let divider_needle = ["SectionDivider", "::new()"].concat();

    // Layout must route through the shared expanded-view scaffold or shell
    assert!(
        layout_source.contains("render_expanded_view_scaffold(")
            || layout_source.contains("render_expanded_view_prompt_shell(")
            // Clipboard history uses manual expanded layout with shared hint strip
            || (layout_source.contains("render_simple_hint_strip(")
                && layout_source.contains("HEADER_PADDING_X")),
        "{name} layout should use shared expanded-view scaffold/shell or manual expanded layout with shared tokens"
    );

    // Must NOT use old PromptFooter
    assert!(
        !layout_source.contains(&prompt_footer_needle),
        "{name} layout should not use PromptFooter::new (scaffold/shell owns the footer)"
    );

    // Must NOT use SectionDivider — expanded view uses spacing, not dividers
    assert!(
        !layout_source.contains(&divider_needle),
        "{name} layout should not use SectionDivider — expanded shell uses spacing, not dividers"
    );

    // Entry file must declare expanded layout mode
    assert!(
        entry_source.contains(&format!("PromptChromeAudit::expanded(\"{name}\"")),
        "{name} entry should emit PromptChromeAudit::expanded(\"{name}\"...) for runtime observability"
    );

    // Entry file must NOT still claim minimal layout
    assert!(
        !entry_source.contains("PromptChromeAudit::minimal("),
        "{name} entry should not emit a minimal chrome audit (it is an expanded surface)"
    );

    eprintln!(
        "{{\"audit\":\"expanded_contract\",\"surface\":\"{name}\",\"scaffold_used\":{},\"divider_absent\":true,\"footer_absent\":true,\"layout_mode\":\"expanded\"}}",
        layout_source.contains("render_expanded_view_scaffold(")
            || layout_source.contains("render_expanded_view_prompt_shell(")
    );
}

// ---- Expanded-view contract tests ----

#[test]
fn clipboard_history_enforces_expanded_view_contract() {
    assert_expanded_builtin_surface(
        "clipboard_history",
        CLIPBOARD_HISTORY_ENTRY_SOURCE,
        CLIPBOARD_HISTORY_LAYOUT_SOURCE,
    );
}

#[test]
fn file_search_enforces_expanded_view_contract() {
    assert_expanded_builtin_surface(
        "file_search",
        FILE_SEARCH_ENTRY_SOURCE,
        FILE_SEARCH_LAYOUT_SOURCE,
    );
}

// ---- Minimal-list contract tests ----

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
