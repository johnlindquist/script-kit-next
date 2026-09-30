//! Source-level contract for the native main-window footer surface owner.

const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");
const VIBRANCY_CONFIG_SOURCE: &str = include_str!("../src/platform/vibrancy_config.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let open = after_start
        .find('{')
        .unwrap_or_else(|| panic!("missing function body for: {signature}"));
    let mut depth = 0usize;
    for (offset, ch) in after_start[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after_start[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

/// Shrink-only ratchet: main-window views that still opt OUT of the native
/// footer. The Tips browser shipped 2026-07-11 with a hand-rolled hint-strip
/// footer because `TipsView` silently joined this `None` group; every NEW
/// view must own a native footer surface (shared FooterButtonConfig buttons)
/// and may only appear here after an explicit, documented decision. Fixing a
/// legacy view means DELETING its entry — never add one.
#[test]
fn main_window_views_without_native_footer_are_ratcheted() {
    const LEGACY_GPUI_FOOTER_VIEWS: &[&str] = &["ActionsDialog"];

    let body = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    let segments: Vec<&str> = body.split("=>").collect();
    let mut none_views: Vec<String> = Vec::new();
    for index in 1..segments.len() {
        if !segments[index].trim_start().starts_with("None") {
            continue;
        }
        let mut pattern = segments[index - 1];
        while let Some(position) = pattern.find("AppView::") {
            let after = &pattern[position + "AppView::".len()..];
            let name: String = after
                .chars()
                .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                .collect();
            if !name.is_empty() {
                none_views.push(name);
            }
            pattern = after;
        }
    }

    assert!(
        !none_views.is_empty(),
        "expected to parse the legacy `=> None` arms of native_footer_surface"
    );
    for view in &none_views {
        assert!(
            LEGACY_GPUI_FOOTER_VIEWS.contains(&view.as_str()),
            "AppView::{view} maps to None in native_footer_surface but is not a \
             grandfathered legacy surface. New main-window views MUST return \
             Some(surface) and declare native footer buttons in \
             main_window_footer_buttons_for_current_view — do not extend the \
             legacy list; it is shrink-only."
        );
    }
    for legacy in LEGACY_GPUI_FOOTER_VIEWS {
        assert!(
            none_views.iter().any(|view| view == legacy),
            "AppView::{legacy} no longer opts out of the native footer — delete it \
             from LEGACY_GPUI_FOOTER_VIEWS (shrink-only ratchet)"
        );
    }
}

/// The six OF-57 migrations use the same action authorities as their physical
/// keyboard handlers; native footer labels must never become decorative.
#[test]
fn migrated_footer_buttons_route_to_real_surface_actions() {
    let buttons = function_body(
        UI_WINDOW_SOURCE,
        "fn main_window_footer_buttons_for_current_view",
    );
    for expected in [
        "about_footer_buttons(enabled)",
        "micro_prompt_footer_buttons(enabled)",
        "sdk_reference_footer_buttons(enabled, has_selection)",
        "script_template_catalog_footer_buttons(enabled, has_selection)",
        "create_ai_preset_footer_buttons(enabled, !name.trim().is_empty())",
        "notes_browse_footer_buttons(",
    ] {
        assert!(
            buttons.contains(expected),
            "migrated footer config must use its truthful button grammar: {expected}"
        );
    }

    let dispatch = function_body(
        UI_WINDOW_SOURCE,
        "pub(crate) fn dispatch_main_window_footer_action",
    );
    for expected in [
        "self.dismiss_about(cx);",
        "self.submit_arg_prompt_from_current_state(&prompt_id, cx);",
        "self.copy_selected_sdk_reference_markdown(cx)",
        "self.show_naming_dialog_for_script_template(template, window, cx);",
        "self.copy_selected_script_template_markdown(cx)",
        "self.handle_create_ai_preset_key(\"enter\", window, cx);",
        "self.handle_create_ai_preset_key(\"tab\", window, cx);",
        "self.activate_selected_note_search_result_from_footer(cx)",
        "self.close_attachment_portal_cancel(cx);",
    ] {
        assert!(
            dispatch.contains(expected),
            "migrated footer action must dispatch real surface behavior: {expected}"
        );
    }
}

#[test]
fn tips_view_keeps_native_footer_with_try_it_primary_action() {
    let footer_map = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    assert!(
        footer_map.contains("AppView::TipsView { .. } => Some(\"tips\")"),
        "TipsView must keep the main-window native footer active"
    );

    // The ↵ label and enabled state come from `Tip::try_it` (unit-tested in `src/tips.rs`);
    // this lock only keeps the capsule's Run routed to the same try path as Enter.
    let footer_dispatch = function_body(
        UI_WINDOW_SOURCE,
        "pub(crate) fn dispatch_main_window_footer_action",
    );
    assert!(
        footer_dispatch.contains("AppView::TipsView { .. }")
            && footer_dispatch.contains("self.tips_try_selected(window, cx);"),
        "Help footer Run must try the selected tip through the same path as Enter"
    );
}

#[test]
fn script_issues_view_keeps_native_footer_and_fix_in_agent_primary_action() {
    let footer_map = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    assert!(
        footer_map.contains("AppView::ScriptIssuesView { .. } => Some(\"script_issues\")"),
        "ScriptIssuesView must keep the main-window native footer active"
    );

    let footer_buttons = function_body(
        UI_WINDOW_SOURCE,
        "fn main_window_footer_buttons_for_current_view",
    );
    assert!(
        footer_buttons.contains("AppView::ScriptIssuesView { .. }")
            && footer_buttons
                .contains("FooterButtonConfig::new(FooterAction::Run, \"↵\", \"Fix in Agent\")")
            && footer_buttons
                .contains("FooterButtonConfig::new(FooterAction::Apply, \"⌘C\", \"Copy Issues\")"),
        "ScriptIssuesView footer must expose Fix in Agent and Copy Issues"
    );

    let footer_dispatch = function_body(
        UI_WINDOW_SOURCE,
        "pub(crate) fn dispatch_main_window_footer_action",
    );
    assert!(
        footer_dispatch.contains("AppView::ScriptIssuesView { report }")
            && footer_dispatch.contains("self.fix_script_issues_in_agent(&report, cx);"),
        "ScriptIssuesView footer Run must submit diagnostics to Agent Chat"
    );

    assert!(
        footer_dispatch.contains("AppView::ScriptIssuesView { report }")
            && footer_dispatch.contains("self.copy_script_issues_to_clipboard(&report, cx);"),
        "ScriptIssuesView footer Copy Issues must copy diagnostics"
    );
}

#[test]
fn native_footer_visual_effect_refresh_invalidates_without_synchronous_redisplay() {
    // The shared host-refresh body lives in `refresh_footer_host_impl`;
    // `refresh_main_footer_host` is a thin wrapper that only chooses glyph
    // ownership.
    let refresh = function_body(FOOTER_POPUP_SOURCE, "unsafe fn refresh_footer_host_impl");
    let invalidate = function_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn invalidate_footer_effect_view_theme",
    );

    assert!(
        refresh.contains("footer_content_changed")
            && refresh.contains("footer_visuals_changed")
            && refresh.contains("effect_theme_changed")
            && refresh.contains("invalidate_footer_effect_view_theme(")
            && refresh.contains("effect_theme_changed\n            || footer_geometry_changed"),
        "native footer refresh must invalidate the AppKit visual-effect background after theme/material/geometry changes"
    );
    assert!(
        refresh.contains("if footer_content_changed")
            && refresh.contains("} else if footer_visuals_changed {")
            && refresh.contains(
                "recolor_footer_hint_subviews_with_visual_theme(hints_view, theme, native_visual_theme);"
            )
            && refresh.contains("layout_footer_hints(hints_view, text_color, &config.buttons, theme);"),
        "theme-only native footer refreshes must recolor existing AppKit hint subviews instead of rebuilding buttons"
    );
    assert!(
        invalidate.contains("effect_theme_changed")
            && invalidate.contains("setNeedsLayout: YES")
            && invalidate.contains("setNeedsDisplay: YES")
            && invalidate.contains("footer_layer")
            && invalidate.contains("setNeedsDisplay")
            && !invalidate.contains("layoutSubtreeIfNeeded")
            && !invalidate.contains("displayIfNeeded"),
        "native footer visual-effect refresh should invalidate layout/display without forcing synchronous AppKit display"
    );
}

#[test]
fn native_vibrancy_config_skips_redundant_same_window_reapply() {
    let body = function_body(
        VIBRANCY_CONFIG_SOURCE,
        "pub fn configure_window_vibrancy_material_for_appearance",
    );

    assert!(
        VIBRANCY_CONFIG_SOURCE.contains("static LAST_MAIN_WINDOW_VIBRANCY_SIGNATURE")
            && body.contains("let signature = (")
            && body.contains("window as usize,")
            && body.contains("background_tint,")
            && body.contains("glass_bits,")
            && body.contains("guard.as_ref() == Some(&signature)")
            && body.contains("return;")
            && body.find("guard.as_ref() == Some(&signature)")
                < body.find("configure_visual_effect_views_recursive("),
        "theme preview must not recursively reconfigure native vibrancy when window, appearance, and material are unchanged"
    );
}

#[test]
fn ui_window_delegates_footer_surface_to_app_view_contract() {
    let body = function_body(UI_WINDOW_SOURCE, "fn main_window_footer_surface");
    assert!(
        body.contains("self.current_view.native_footer_surface()"),
        "ui_window must delegate footer surface identity to AppView::native_footer_surface"
    );
    assert!(
        !body.contains("match &self.current_view"),
        "ui_window must not duplicate the AppView footer surface map"
    );
}

#[test]
fn main_window_footer_config_exposes_slot_model_contract() {
    for needle in [
        "pub(crate) const MAIN_WINDOW_FOOTER_MAX_ACTION_SLOTS: usize = 3",
        "pub(crate) enum FooterSlotRole",
        "ActionSlot",
        "ContextChip",
        "pub(crate) struct MainWindowFooterSlotModel",
        "pub action_slot_count: usize",
        "pub context_chip_count: usize",
        "pub duplicate_shortcut_keys: Vec<String>",
        "pub violation: Option<&'static str>",
        "pub(crate) fn footer_button_slot_role",
        "pub(crate) fn slot_model(&self) -> MainWindowFooterSlotModel",
        "pub(crate) fn slot_contract_violation(&self) -> Option<&'static str>",
    ] {
        assert!(
            FOOTER_POPUP_SOURCE.contains(needle),
            "footer config must expose semantic slot-model contract: {needle}"
        );
    }

    assert!(
        FOOTER_POPUP_SOURCE.contains("let model = config.slot_model();")
            && FOOTER_POPUP_SOURCE.contains("debug_assert!(")
            && FOOTER_POPUP_SOURCE.contains("tracing::warn!")
            && FOOTER_POPUP_SOURCE.contains("duplicate_shortcut_keys"),
        "MainWindowFooterConfig::new must audit the slot model without mutating button availability"
    );
}

#[test]
fn footer_context_chips_do_not_count_as_action_slots() {
    let role_body = function_body(FOOTER_POPUP_SOURCE, "pub(crate) fn footer_button_slot_role");
    assert!(
        role_body.contains("FooterAction::Cwd | FooterAction::AgentModel")
            && role_body.contains("FooterSlotRole::ContextChip")
            && role_body.contains("matches!(button.action, FooterAction::Ai)")
            && role_body.contains("crate::components::footer_chrome::FOOTER_MIC_ICON_TOKEN")
            && role_body.contains("FooterSlotRole::ActionSlot"),
        "cwd, agent/model, and mic-key AI footer entries must be context chips, not action slots"
    );

    let slot_model_body = function_body(FOOTER_POPUP_SOURCE, "pub(crate) fn slot_model(");
    assert!(
        slot_model_body.contains("FooterSlotRole::ActionSlot")
            && slot_model_body.contains("action_slot_count += 1")
            && slot_model_body.contains("FooterSlotRole::ContextChip")
            && slot_model_body.contains("context_chip_count += 1")
            && slot_model_body.contains("too_many_action_slots")
            && slot_model_body.contains("duplicate_shortcut_keys"),
        "slot_model must count action slots separately from context chips and report violations"
    );
}

#[test]
fn cwd_and_agent_model_are_not_rendered_as_main_window_footer_chips() {
    assert!(
        !UI_WINDOW_SOURCE.contains("FooterButtonConfig::new(FooterAction::Cwd")
            && !UI_WINDOW_SOURCE.contains("FooterButtonConfig::new(FooterAction::AgentModel"),
        "cwd and Agent/model are shared-header controls and must not be duplicated as native footer buttons"
    );
    assert!(
        !UI_WINDOW_SOURCE.contains("prepend_global_main_window_left_chips")
            && !UI_WINDOW_SOURCE.contains("global_main_window_left_chip_buttons")
            && !UI_WINDOW_SOURCE.contains("current_view_shows_global_left_chips"),
        "main-window footer config must not have a global cwd/model prepending path"
    );
}
