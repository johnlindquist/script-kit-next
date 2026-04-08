// --- Expanded-view surfaces (preview IS the decision) ---
const CLIPBOARD_HISTORY_ENTRY_SOURCE: &str = include_str!("../../src/render_builtins/clipboard.rs");
const CLIPBOARD_HISTORY_LAYOUT_SOURCE: &str =
    include_str!("../../src/render_builtins/clipboard_history_layout.rs");
const FILE_SEARCH_ENTRY_SOURCE: &str = include_str!("../../src/render_builtins/file_search.rs");

// file_search_layout.rs is now a legacy stub — expanded/mini contract
// assertions use the live file_search.rs entry source instead.

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
            || source.contains("render_minimal_list_prompt_scaffold_footer_aware(")
            || source.contains("render_minimal_list_prompt_shell("),
        "{name} should use a shared minimal chrome helper (hint strip or minimal scaffold)"
    );

    assert!(
        !source.contains(&prompt_footer_needle),
        "{name} should not use PromptFooter::new"
    );

    let uses_scaffold = source.contains("render_minimal_list_prompt_scaffold(")
        || source.contains("render_minimal_list_prompt_scaffold_footer_aware(")
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
            || layout_source.contains("render_expanded_view_scaffold_with_hints(")
            || layout_source.contains("render_expanded_view_scaffold_with_footer(")
            || layout_source.contains("render_expanded_view_prompt_shell("),
        "{name} layout should use shared expanded-view scaffold/shell"
    );

    // Layout should route footer rendering through the main-window native footer slot
    assert!(
        layout_source.contains("main_window_footer_slot("),
        "{name} layout should route footer rendering through the main-window native footer slot"
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
        "{{\"audit\":\"expanded_contract\",\"surface\":\"{name}\",\"scaffold_used\":{},\"footer_slot\":true,\"divider_absent\":true,\"footer_absent\":true,\"layout_mode\":\"expanded\"}}",
        layout_source.contains("render_expanded_view_scaffold(")
            || layout_source.contains("render_expanded_view_scaffold_with_hints(")
            || layout_source.contains("render_expanded_view_scaffold_with_footer(")
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
    // The live rendering is fully in file_search.rs (entry source).
    // file_search_layout.rs is a legacy stub with no chrome markers.
    assert_expanded_builtin_surface(
        "file_search",
        FILE_SEARCH_ENTRY_SOURCE,
        FILE_SEARCH_ENTRY_SOURCE,
    );
}

// ---- File search mini footer contract ----

#[test]
fn file_search_mini_footer_uses_live_hint_contract() {
    // The live rendering is in file_search.rs (entry source).
    assert!(
        FILE_SEARCH_ENTRY_SOURCE.contains("emit_prompt_hint_audit(\"file_search\""),
        "file_search should emit a prompt hint audit"
    );
    assert!(
        FILE_SEARCH_ENTRY_SOURCE.contains("let file_search_hints = if")
            && (FILE_SEARCH_ENTRY_SOURCE.contains("render_minimal_list_prompt_scaffold(")
                || FILE_SEARCH_ENTRY_SOURCE
                    .contains("render_minimal_list_prompt_shell_with_footer(")),
        "file_search mini mode should compute live hints and route them through a minimal scaffold"
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

// ---- Actions parity contract tests ----
// Ensures footer hints are truthful: surfaces that advertise ⌘K Actions must
// actually wire the shared ActionsDialog, and surfaces without actions must not
// advertise them.

const EMOJI_PICKER_SOURCE: &str = include_str!("../../src/render_builtins/emoji_picker.rs");
const THEME_CHOOSER_SOURCE: &str = include_str!("../../src/render_builtins/theme_chooser.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../../src/app_impl/actions_dialog.rs");

fn production_source(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

/// Surfaces that advertise universal_prompt_hints() must also wire the shared
/// actions dialog (either locally via route_key_to_actions_dialog or by being
/// listed as SharedDialog in actions_support_for_view).
#[test]
fn surfaces_that_advertise_universal_hints_must_have_actions_support() {
    for (source, label) in [(EMOJI_PICKER_SOURCE, "emoji_picker")] {
        let advertises_actions = source.contains("universal_prompt_hints()");
        // Either the surface locally wires the router, or the canonical resolver
        // maps it to SharedDialog (checked via the ActionsDialogHost variant name).
        let has_local_router = source.contains("route_key_to_actions_dialog(");
        let variant_name = match label {
            "app_launcher" => "AppLauncher",
            "window_switcher" => "WindowSwitcher",
            "emoji_picker" => "EmojiPicker",
            "current_app_commands" => "CurrentAppCommands",
            "process_manager" => "ProcessManager",
            _ => "",
        };
        let in_canonical_resolver =
            ACTIONS_DIALOG_SOURCE.contains(&format!("ActionsDialogHost::{variant_name}"));

        assert!(
            !advertises_actions || has_local_router || in_canonical_resolver,
            "{label} advertises universal hints but has no shared actions routing"
        );
    }
}

/// Surfaces explicitly excluded from shared actions must not use universal hints.
#[test]
fn no_actions_surfaces_must_not_advertise_universal_hints() {
    for (source, label) in [
        (APP_LAUNCHER_SOURCE, "app_launcher"),
        (WINDOW_SWITCHER_SOURCE, "window_switcher"),
        (CURRENT_APP_COMMANDS_SOURCE, "current_app_commands"),
        (PROCESS_MANAGER_SOURCE, "process_manager"),
    ] {
        let source = production_source(source);
        assert!(
            !source.contains("universal_prompt_hints()"),
            "{label} does not support shared actions but advertises universal hints"
        );
    }
}

/// ThemeChooser stays truthful: no universal actions hints, custom footer only.
#[test]
fn theme_chooser_stays_truthful_and_does_not_advertise_actions() {
    let source = production_source(THEME_CHOOSER_SOURCE);
    assert!(
        !source.contains("universal_prompt_hints()"),
        "theme_chooser should not use universal actions hints without shared actions support"
    );
    assert!(
        source.contains("render_simple_hint_strip(")
            || source.contains("render_minimal_list_prompt_scaffold("),
        "theme_chooser should keep a truthful custom footer"
    );
}

/// AppLauncher must NOT be in the SharedDialog path — it has no Cmd+K interceptor arm.
#[test]
fn app_launcher_excluded_from_shared_actions_resolver() {
    let resolver_start = ACTIONS_DIALOG_SOURCE
        .find("fn actions_support_for_view")
        .expect("canonical resolver not found");
    let resolver_fn = &ACTIONS_DIALOG_SOURCE[resolver_start..];

    assert!(
        !resolver_fn.contains("ActionsSupport::SharedDialog(ActionsDialogHost::AppLauncher)"),
        "app_launcher should not be mapped to SharedDialog (no Cmd+K toggle exists)"
    );
}

/// ThemeChooser must NOT be in the SharedDialog path.
#[test]
fn theme_chooser_excluded_from_shared_actions_resolver() {
    let resolver_start = ACTIONS_DIALOG_SOURCE
        .find("fn actions_support_for_view")
        .expect("canonical resolver not found");
    let resolver_fn = &ACTIONS_DIALOG_SOURCE[resolver_start..];

    assert!(
        !resolver_fn.contains("ActionsSupport::SharedDialog(ActionsDialogHost::ThemeChooser)"),
        "theme_chooser should not be mapped to SharedDialog"
    );
}

/// Emoji picker must have both: universal hints AND actual actions wiring.
#[test]
fn emoji_picker_advertises_and_wires_shared_actions() {
    assert!(
        EMOJI_PICKER_SOURCE.contains("universal_prompt_hints()"),
        "emoji_picker should use universal hints (it supports shared actions)"
    );
    assert!(
        EMOJI_PICKER_SOURCE.contains("route_key_to_actions_dialog("),
        "emoji_picker must locally wire the shared actions router"
    );
}

// ---- Native footer contract tests ----

const UI_WINDOW_SOURCE: &str = include_str!("../../src/app_impl/ui_window.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../../src/footer_popup.rs");

#[test]
fn ui_window_resolves_native_footer_from_view() {
    assert!(
        UI_WINDOW_SOURCE.contains("fn main_window_footer_config("),
        "ui_window.rs must define a main-window footer config resolver"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("MainWindowFooterConfig::new("),
        "main-window footer resolver must build MainWindowFooterConfig values"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("main_window_footer_slot("),
        "ui_window.rs must expose a footer-slot helper for GPUI surfaces"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("sync_main_footer_popup(window, config.as_ref(),"),
        "native footer sync must pass the resolved config into footer_popup.rs"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_contract\",\"resolver\":true,\"slot_helper\":true,\"config_sync\":true}}"
    );
}

#[test]
fn footer_popup_accepts_config_driven_refresh() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("struct FooterButtonConfig"),
        "footer_popup.rs must define FooterButtonConfig"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("struct MainWindowFooterConfig"),
        "footer_popup.rs must define MainWindowFooterConfig"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("config: Option<&MainWindowFooterConfig>"),
        "sync_main_footer_popup must accept Option<&MainWindowFooterConfig>"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_popup_contract\",\"button_config\":true,\"footer_config\":true,\"config_param\":true}}"
    );
}

#[test]
fn footer_popup_uses_subtle_hover_for_native_footer_buttons() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("let hover_ns: id = ns_color_from_rgba(chrome.hover_rgba);"),
        "native footer hover should use chrome.hover_rgba so the blur stays subtle"
    );
    assert!(
        FOOTER_POPUP_SOURCE
            .contains("let selected_ns: id = ns_color_from_rgba(chrome.selection_rgba);"),
        "native footer selected state should still restore chrome.selection_rgba"
    );
}

#[test]
fn script_list_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_script_list/mod.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "render_script_list should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"script_list\",\"footer_slot\":true}}"
    );
}

#[test]
fn prompt_wrapper_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_prompts/other.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "render_wrapped_prompt_entity should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"prompt_wrapper\",\"footer_slot\":true}}"
    );
}

#[test]
fn clipboard_history_routes_footer_through_native_slot() {
    assert!(
        CLIPBOARD_HISTORY_LAYOUT_SOURCE.contains("render_expanded_view_scaffold_with_footer("),
        "clipboard_history should use the footer-aware expanded scaffold"
    );
    assert!(
        CLIPBOARD_HISTORY_LAYOUT_SOURCE.contains("main_window_footer_slot("),
        "clipboard_history should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"clipboard_history\",\"footer_slot\":true}}"
    );
}

#[test]
fn file_search_routes_footer_through_native_slot() {
    assert!(
        FILE_SEARCH_ENTRY_SOURCE.contains("main_window_footer_slot("),
        "file_search should route footer rendering through the main-window native footer slot"
    );
    assert!(
        FILE_SEARCH_ENTRY_SOURCE.contains("render_expanded_view_scaffold_with_footer(")
            || FILE_SEARCH_ENTRY_SOURCE.contains("render_minimal_list_prompt_shell_with_footer("),
        "file_search should use footer-aware scaffold helpers"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"file_search\",\"footer_slot\":true}}"
    );
}

#[test]
fn mini_prompt_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_prompts/mini.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "mini_prompt should route footer rendering through the main-window native footer slot"
    );
    assert!(
        source.contains("render_minimal_list_prompt_shell_with_footer("),
        "mini_prompt should use the footer-aware minimal list prompt shell"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"mini_prompt\",\"footer_slot\":true}}"
    );
}

#[test]
fn div_prompt_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_prompts/div.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "div_prompt should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"div_prompt\",\"footer_slot\":true}}"
    );
}

#[test]
fn form_prompt_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_prompts/form/render.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "form_prompt should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"form_prompt\",\"footer_slot\":true}}"
    );
}

#[test]
fn editor_prompt_routes_footer_through_native_slot() {
    let source = include_str!("../../src/render_prompts/editor.rs");
    assert!(
        source.contains("main_window_footer_slot("),
        "editor_prompt should route footer rendering through the main-window native footer slot"
    );

    eprintln!(
        "{{\"audit\":\"native_footer_surface\",\"surface\":\"editor_prompt\",\"footer_slot\":true}}"
    );
}

/// Summary audit: verifies all migrated surfaces route through the native footer
/// slot, emitting a single JSON envelope listing all covered surfaces.
#[test]
fn native_footer_migration_coverage_summary() {
    let surfaces: &[(&str, &str)] = &[
        (
            "script_list",
            include_str!("../../src/render_script_list/mod.rs"),
        ),
        (
            "prompt_wrapper",
            include_str!("../../src/render_prompts/other.rs"),
        ),
        (
            "clipboard_history",
            include_str!("../../src/render_builtins/clipboard_history_layout.rs"),
        ),
        (
            "file_search",
            include_str!("../../src/render_builtins/file_search.rs"),
        ),
        (
            "mini_prompt",
            include_str!("../../src/render_prompts/mini.rs"),
        ),
        (
            "div_prompt",
            include_str!("../../src/render_prompts/div.rs"),
        ),
        (
            "form_prompt",
            include_str!("../../src/render_prompts/form/render.rs"),
        ),
        (
            "editor_prompt",
            include_str!("../../src/render_prompts/editor.rs"),
        ),
    ];

    let mut covered = Vec::new();
    for (name, source) in surfaces {
        assert!(
            source.contains("main_window_footer_slot("),
            "{name} must route footer through main_window_footer_slot"
        );
        covered.push(*name);
    }

    let covered_json: Vec<String> = covered.iter().map(|s| format!("\"{}\"", s)).collect();
    eprintln!(
        "{{\"audit\":\"native_footer_migration_summary\",\"covered_surfaces\":[{}],\"total\":{}}}",
        covered_json.join(","),
        covered.len(),
    );
}

/// The canonical resolver must list all ActionsDialogHost variants that have
/// focus-restore mappings, ensuring the two stay in sync.
#[test]
fn canonical_resolver_covers_all_focus_restore_hosts() {
    let resolver_start = ACTIONS_DIALOG_SOURCE
        .find("fn actions_support_for_view")
        .expect("canonical resolver not found");
    let resolver_fn = &ACTIONS_DIALOG_SOURCE[resolver_start..];

    for host in [
        "ActionsDialogHost::MainList",
        "ActionsDialogHost::ClipboardHistory",
        "ActionsDialogHost::EmojiPicker",
        "ActionsDialogHost::FileSearch",
        "ActionsDialogHost::ChatPrompt",
        "ActionsDialogHost::ArgPrompt",
        "ActionsDialogHost::WebcamPrompt",
        "ActionsDialogHost::AcpChat",
    ] {
        assert!(
            resolver_fn.contains(host),
            "canonical actions resolver should map {host}"
        );
    }
}
