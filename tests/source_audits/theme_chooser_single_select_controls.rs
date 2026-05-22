use super::read_source;

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|section| section.split(end).next())
        .unwrap_or_else(|| panic!("missing source section between `{start}` and `{end}`"))
}

#[test]
fn theme_chooser_customize_opacity_controls_are_single_select() {
    let source = read_source("src/render_builtins/theme_chooser.rs");
    let customize_controls = read_source("src/render_builtins/theme_chooser_customize_controls.rs");

    assert!(!source.contains("OPACITY_MATCH_TOLERANCE"));
    assert!(source.contains("fn closest_float_preset_index("));
    assert!(source.contains("ThemeChooserSliderBinding::SecondaryTextOpacity"));
    assert!(source.contains("Self::apply_text_opacity_preset("));
    assert!(source.contains("ThemeChooserSliderBinding::FocusedBackgroundOpacity"));
    assert!(source.contains("Self::apply_focused_background_opacity_preset("));
    assert!(
        customize_controls.contains("let current_opacity_index = Self::find_opacity_preset_index(")
    );
    assert!(customize_controls.contains("let is_current = i == current_opacity_index;"));
}

#[test]
fn theme_chooser_customize_opacity_controls_cover_full_percent_range() {
    let source = read_source("src/render_builtins/theme_chooser.rs");

    assert!(source.contains(
        "const OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert!(source.contains(
        "const TEXT_OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert!(source.contains(
        "const FOCUSED_BACKGROUND_OPACITY_PRESETS: &'static [(f32, &'static str)] = &[\n        (0.00, \"0%\"),"
    ));
    assert_eq!(
        source.matches("(1.00, \"100%\"),").count(),
        3,
        "each theme designer opacity control should expose a 100% endpoint"
    );
}

#[test]
fn theme_chooser_exposes_user_theme_management_and_gradient_actions() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let actions = read_source("src/render_builtins/actions.rs");
    let user_themes = read_source("src/theme/user_themes.rs");
    let theme_types = read_source("src/theme/types.rs");
    let render_impl = read_source("src/main_sections/render_impl.rs");

    assert!(chooser.contains("fn theme_chooser_catalog()"));
    assert!(chooser.contains("theme::user_themes::list_user_themes()"));
    assert!(chooser.contains("theme::user_themes::load_user_theme"));
    assert!(chooser.contains("save_current_theme_as_user_theme"));
    assert!(chooser.contains("delete_selected_user_theme"));
    assert!(chooser.contains("cycle_theme_chooser_gradient"));
    assert!(chooser.contains("self.apply_theme_chooser_theme(next_theme, reason, cx);"));
    assert!(
        !chooser.contains("this.apply_and_persist_theme("),
        "Theme Designer customization clicks should preview only; Done/Enter owns persistence"
    );
    assert!(
        !chooser
            .contains("\"theme_chooser_mouse_click\",\n                                    true"),
        "Theme Designer row clicks should not persist active theme.json"
    );
    assert!(
        !chooser.contains("persist_theme_and_sync_all_windows(\n                    cx,\n                    self.theme.as_ref(),\n                    reason"),
        "Save as user theme should write the library preset without applying active theme.json"
    );

    for action_id in [
        "theme_chooser_save_as_user_theme",
        "theme_chooser_edit_theme_as_text",
        "theme_chooser_update_user_theme",
        "theme_chooser_delete_user_theme",
        "theme_chooser_restore_deleted_user_theme",
        "theme_chooser_gradient_cycle",
    ] {
        assert!(
            actions.contains(action_id) && chooser.contains(action_id),
            "Theme Designer action `{action_id}` must be exposed in actions and executed"
        );
    }

    assert!(user_themes.contains("pub fn save_theme_as_user_theme("));
    assert!(user_themes.contains("pub fn save_user_theme_unique("));
    assert!(user_themes.contains("pub fn resolve_user_theme_name("));
    assert!(user_themes.contains("pub fn save_theme_to_user_theme_slug("));
    assert!(user_themes.contains("pub fn delete_user_theme_with_backup("));
    assert!(user_themes.contains("pub fn restore_user_theme_backup("));
    assert!(user_themes.contains("pub fn load_user_theme("));
    assert!(user_themes.contains(".get(\"hover\")"));
    assert!(user_themes.contains(".get(\"selected\")"));
    assert!(theme_types.contains("pub struct BackgroundGradient"));
    assert!(theme_types.contains("pub fn active_background_gradient(&self)"));
    assert!(render_impl.contains("theme_background_gradient_layers(\"bg-layer\", &self.theme)"));
}

#[test]
fn theme_chooser_exposes_management_status_and_safe_user_theme_actions() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let actions = read_source("src/render_builtins/actions.rs");
    let collector = read_source("src/app_layout/collect_elements.rs");
    let user_themes = read_source("src/theme/user_themes.rs");

    for symbol in [
        "ThemeChooserManagementState",
        "ThemeChooserManagementStatus",
        "theme_chooser_is_dirty",
        "suggested_theme_chooser_save_name",
        "update_selected_user_theme",
        "confirm_delete_selected_user_theme",
        "restore_last_deleted_user_theme",
    ] {
        assert!(
            chooser.contains(symbol),
            "missing ThemeChooser management symbol `{symbol}`"
        );
    }

    for action in [
        "theme_chooser_save_as_user_theme",
        "theme_chooser_edit_theme_as_text",
        "theme_chooser_update_user_theme",
        "theme_chooser_delete_user_theme",
        "theme_chooser_restore_deleted_user_theme",
    ] {
        assert!(
            actions.contains(action),
            "actions dialog must expose `{action}`"
        );
        assert!(
            chooser.contains(action),
            "ThemeChooser must execute `{action}`"
        );
        assert!(
            collector.contains(action),
            "getElements must expose `{action}`"
        );
    }

    for semantic in [
        "status:theme-chooser-dirty-state",
        "control:theme-chooser:save-name",
        "button:theme-chooser-update-user-theme",
        "button:theme-chooser-restore-deleted-user-theme",
        "action_disabled",
    ] {
        assert!(
            collector.contains(semantic),
            "getElements missing `{semantic}`"
        );
    }

    for persistence in [
        "resolve_user_theme_name",
        "save_theme_to_user_theme_slug",
        "delete_user_theme_with_backup",
        "restore_user_theme_backup",
    ] {
        assert!(
            user_themes.contains(persistence),
            "user_themes missing `{persistence}`"
        );
    }

    assert!(
        !chooser.contains("Custom Theme {}"),
        "Theme Designer save-copy names should not be timestamp-only"
    );
}

#[test]
fn theme_chooser_management_buttons_use_theme_aware_hover_states() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let button = read_source("src/components/button/component.rs");

    assert!(
        button.contains("pub(crate) fn hover_background_token"),
        "shared Button must continue to expose the hover token used by Theme Designer management buttons"
    );
    assert!(
        chooser.contains("enum ThemeChooserManagementButtonKind"),
        "Theme Designer should make primary/neutral/destructive management button intent explicit"
    );
    assert!(
        chooser.contains("fn render_theme_chooser_management_button"),
        "Theme Designer management buttons should share one local helper"
    );
    assert!(
        chooser.contains("const THEME_CHOOSER_MANAGEMENT_HOVER_ALPHA_FLOOR"),
        "Theme Designer management hover should have a local visibility floor"
    );
    assert!(
        chooser.contains("fn theme_chooser_rgba_alpha_floor(")
            && chooser.contains("fn theme_chooser_management_hover_token("),
        "Theme Designer management hover should normalize neutral/destructive hover tokens locally"
    );

    let hover_token_helper = chooser
        .split("fn theme_chooser_management_hover_token")
        .nth(1)
        .and_then(|section| {
            section
                .split("fn render_theme_chooser_management_button")
                .next()
        })
        .expect("missing management hover token helper body");
    assert!(
        hover_token_helper.contains("ThemeChooserManagementButtonKind::Primary")
            && hover_token_helper.contains("ThemeChooserManagementButtonKind::Neutral")
            && hover_token_helper.contains("ThemeChooserManagementButtonKind::Destructive")
            && hover_token_helper.contains("theme_chooser_rgba_alpha_floor"),
        "primary should preserve shared hover, while neutral/destructive should receive a visible local floor"
    );

    let helper = chooser
        .split("fn render_theme_chooser_management_button")
        .nth(1)
        .and_then(|section| section.split("fn ").next())
        .expect("missing management button helper body");
    assert!(
        helper.contains(".when(!disabled")
            && helper.contains(".hover(")
            && helper.contains("if disabled"),
        "management button hover must be enabled-only and disabled clicks must remain guarded"
    );
    assert!(
        helper.contains("kind: ThemeChooserManagementButtonKind")
            && helper.contains("theme_chooser_management_hover_token(kind, hover_rgba)")
            && !helper.contains("_kind: ThemeChooserManagementButtonKind"),
        "management button kind must drive hover behavior instead of being ignored"
    );

    let render_section = chooser
        .split("let management_section =")
        .nth(1)
        .and_then(|section| section.split("let colors_section =").next())
        .expect("missing Theme Designer management section");
    assert_eq!(
        render_section
            .matches("render_theme_chooser_management_button(")
            .count(),
        4,
        "Save Copy, Update, Delete, and Restore should all use the shared local helper"
    );

    for id in [
        "theme-chooser-save-copy-button",
        "theme-chooser-update-user-theme-button",
        "theme-chooser-delete-user-theme-button",
        "theme-chooser-restore-user-theme-button",
    ] {
        assert!(
            render_section.contains(id),
            "management button id `{id}` must be preserved"
        );
    }

    assert!(
        chooser.contains("ButtonColors::from_theme") && chooser.contains("self.theme.as_ref()"),
        "hover styling should be derived from the active theme"
    );
    assert!(
        chooser.contains("ButtonVariant::Primary") && chooser.contains("ButtonVariant::Ghost"),
        "Save Copy should use primary hover semantics and neutral buttons should use ghost hover semantics"
    );
    assert!(
        render_section.contains("destructive_hover_rgba") && render_section.contains("ui_error"),
        "Delete must keep destructive styling instead of using a plain neutral hover"
    );
}

#[test]
fn theme_chooser_customize_cards_use_whisper_tokens_not_panel_tokens() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let render_code = chooser
        .split("#[cfg(test)]")
        .next()
        .expect("render code should precede tests");
    let customize_section = source_between(
        render_code,
        "fn render_theme_chooser_customize_section(",
        "fn theme_chooser_overlay_token(",
    );
    let management_section = source_between(
        render_code,
        "let management_section =",
        "// 1. COLORS SECTION",
    );

    assert!(
        customize_section.contains(".bg(rgba(chrome.whisper_surface_rgba))"),
        "Theme Designer customize cards should use the shared whisper surface"
    );
    assert!(
        customize_section.contains(".border_color(rgba(chrome.whisper_border_rgba))"),
        "Theme Designer customize cards should use the shared whisper border"
    );
    assert!(
        !customize_section.contains("panel_surface_rgba"),
        "customize cards must not use panel_surface_rgba"
    );
    assert!(
        !customize_section.contains("badge_border_rgba"),
        "customize card borders must not use badge_border_rgba"
    );
    assert!(
        management_section.contains("chrome.accent_badge_bg_rgba")
            && management_section
                .matches("chrome.whisper_surface_rgba")
                .count()
                == 3
            && management_section
                .matches("chrome.whisper_border_rgba")
                .count()
                == 3,
        "Save Copy should keep accent chrome while Update/Delete/Restore use whisper chrome"
    );
    assert!(
        !management_section.contains("chrome.panel_surface_rgba"),
        "Theme Designer management buttons must not use panel_surface_rgba"
    );
    assert!(
        !management_section.contains("chrome.badge_border_rgba"),
        "Theme Designer management buttons must not use badge_border_rgba"
    );
}

#[test]
fn theme_gradients_propagate_to_secondary_windows() {
    let ui_foundation = read_source("src/ui_foundation/mod.rs");
    let main_window = read_source("src/main_sections/render_impl.rs");
    let notes = read_source("src/notes/window/render.rs");
    let hud = read_source("src/hud_manager/mod.rs");
    let dictation = read_source("src/dictation/window.rs");

    assert!(ui_foundation.contains("pub fn get_theme_background_gradients(theme: &Theme)"));
    assert!(ui_foundation.contains("pub fn theme_background_gradient_layers("));
    assert!(ui_foundation.contains("theme.active_background_gradient()"));

    for (surface, source, id_prefix) in [
        ("main window", main_window.as_str(), "\"bg-layer\""),
        ("Notes", notes.as_str(), "\"notes-bg-layer\""),
        ("HUD", hud.as_str(), "\"hud-bg-layer\""),
        ("Dictation", dictation.as_str(), "\"dictation-bg-layer\""),
    ] {
        assert!(
            source.contains("theme_background_gradient_layers(") && source.contains(id_prefix),
            "{surface} must render the active theme gradient through the shared layer helper"
        );
    }

    assert!(
        dictation.contains("\"dictation-preview-bg-layer\""),
        "Dictation Storybook/static preview must use the same active theme gradient path"
    );
}

#[test]
fn theme_chooser_controls_are_devtools_visible_and_drivable() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let collector = read_source("src/app_layout/collect_elements.rs");
    let prompt_handler = read_source("src/prompt_handler/mod.rs");
    let protocol = read_source("src/protocol/types/batch_wait.rs");
    let color_picker = read_source("vendor/gpui-component/crates/ui/src/color_picker.rs");

    for (picker_control, hex_control, label) in [
        ("accent-color", "accent-color-hex", "Accent Color"),
        (
            "gradient-base-from",
            "gradient-base-from-hex",
            "Gradient Base From",
        ),
        (
            "gradient-base-to",
            "gradient-base-to-hex",
            "Gradient Base To",
        ),
    ] {
        assert!(
            collector.contains(&format!(
                "\"control:theme-chooser:{picker_control}\".to_string()"
            )) && collector.contains("protocol::ElementType::ColorPicker")
                && collector.contains(&format!("\"{label}\"")),
            "getElements must expose Theme Designer color picker `{picker_control}`"
        );
        assert!(
            collector.contains(&format!(
                "\"control:theme-chooser:{hex_control}\".to_string()"
            )) && collector.contains("protocol::ElementType::Input")
                && collector.contains(&format!("\"{label} Hex\"")),
            "getElements must expose Theme Designer typed hex input `{hex_control}` beside `{picker_control}`"
        );
        assert!(
            chooser.contains(&format!("\"{picker_control}\""))
                && chooser.contains(&format!("\"{hex_control}\"")),
            "Theme Designer devtools setter must handle `{picker_control}` and `{hex_control}`"
        );
    }

    for control in [
        "surface-opacity",
        "secondary-text-opacity",
        "focused-background-opacity",
        "vibrancy-enabled",
        "gradient-enabled",
        "gradient-base-angle",
        "gradient-base-opacity",
        "ui-font-size",
        "gradient-layer-",
    ] {
        assert!(
            collector.contains(control),
            "getElements must expose Theme Designer control `{control}`"
        );
        assert!(
            chooser.contains(control),
            "Theme Designer devtools setter must handle control `{control}`"
        );
    }

    assert!(
        collector.contains("format!(\"control:theme-chooser:gradient-layer-{ordinal}-from\")")
            && collector
                .contains("format!(\"control:theme-chooser:gradient-layer-{ordinal}-from-hex\")")
            && collector.contains("format!(\"control:theme-chooser:gradient-layer-{ordinal}-to\")")
            && collector
                .contains("format!(\"control:theme-chooser:gradient-layer-{ordinal}-to-hex\")"),
        "dynamic gradient layer color rows must expose both picker and typed hex controls"
    );

    for element_type in [
        "ElementType::Slider",
        "ElementType::ColorPicker",
        "ElementType::Input",
        "ElementType::Toggle",
    ] {
        assert!(
            collector.contains(element_type),
            "Theme Designer controls must expose semantic {element_type} elements"
        );
    }

    assert!(protocol.contains("SetThemeControl"));
    assert!(chooser.contains("strip_prefix(\"control:theme-chooser:\")"));
    assert!(prompt_handler.contains("set_theme_chooser_control_from_devtools"));
    assert!(prompt_handler.contains("\"setThemeControl\".to_string()"));
    assert!(prompt_handler.contains("setThemeControl requires ThemeChooserView"));
    assert!(!prompt_handler.contains(".set_theme_chooser_control_from_devtools(&control, &value, cx)\n                                                .ok()"));
    assert!(
        color_picker.contains("pub enum ColorPickerEvent")
            && color_picker.contains("Change(Option<Hsla>)")
            && color_picker.contains(".open(state.open)")
            && color_picker.contains(".on_open_change("),
        "Theme Designer should keep relying on gpui-component ColorPicker popup open/close and change semantics"
    );
}

#[test]
fn theme_chooser_color_rows_use_single_picker_with_typed_hex_input() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let new_controls = source_between(
        &chooser,
        "fn new_theme_chooser_color_controls(",
        "fn new_theme_chooser_gradient_controls(",
    );
    let render_row = source_between(
        &chooser,
        "fn render_theme_chooser_color_picker_row(",
        "pub(crate) fn submit_theme_chooser_from_input_enter(",
    );
    let sync_color = source_between(
        &chooser,
        "fn sync_color_control_value(",
        "fn sync_gradient_controls_from_theme(",
    );

    assert!(chooser.contains("pub(crate) struct ThemeChooserColorControls"));
    assert!(chooser.contains("picker: Entity<ColorPickerState>"));
    assert!(chooser.contains("hex_input: Entity<gpui_component::input::InputState>"));
    assert!(chooser.contains("fn new_theme_chooser_color_controls("));
    assert!(chooser.contains("gpui_component::input::InputEvent::Change"));
    assert!(chooser.contains("apply_theme_chooser_hex_text_if_valid"));
    assert!(chooser.contains("parse_theme_chooser_hex_input(text)"));
    assert!(chooser.contains("apply_theme_chooser_color_hex_change("));
    assert_eq!(
        new_controls.matches("ColorPickerState::new(window, cx)").count(),
        1,
        "each Theme Designer color control bundle should allocate exactly one gpui-component ColorPickerState"
    );
    assert_eq!(
        new_controls
            .matches("gpui_component::input::InputState::new(window, cx)")
            .count(),
        1,
        "each Theme Designer color control bundle should allocate exactly one always-visible typed hex InputState"
    );
    assert!(
        new_controls.contains("ColorPickerEvent::Change(Some(color))")
            && new_controls.contains(
                "this.apply_theme_chooser_color_change(binding, *color, cx);"
            )
            && new_controls.contains("ColorPickerEvent::Change(None) => {}"),
        "Theme Designer must route gpui-component ColorPickerEvent::Change through Theme Designer state"
    );
    assert!(
        new_controls.contains("gpui_component::input::InputEvent::Change")
            && new_controls.contains(
                "this.apply_theme_chooser_hex_text_if_valid(binding, &value, cx);"
            )
            && new_controls.contains("gpui_component::input::InputEvent::PressEnter { .. }")
            && new_controls.contains("gpui_component::input::InputEvent::Blur")
            && new_controls.contains("this.sync_theme_chooser_control_values(window, cx);"),
        "typed hex changes should preview valid colors while Enter/Blur reconcile the visible input"
    );
    assert!(
        render_row.contains("gpui_component::input::Input::new(&controls.hex_input)")
            && render_row.contains("ColorPicker::new(&controls.picker)"),
        "Theme Designer color row must expose typed hex editing beside the single picker swatch"
    );
    assert_eq!(
        render_row
            .matches("gpui_component::input::Input::new(&controls.hex_input)")
            .count(),
        1,
        "Theme Designer color row should render one visible typed hex input"
    );
    assert_eq!(
        render_row
            .matches("ColorPicker::new(&controls.picker)")
            .count(),
        1,
        "Theme Designer color row should render one gpui-component ColorPicker"
    );
    assert!(
        render_row.contains(".featured_colors(Self::theme_chooser_featured_colors())")
            && render_row.contains(".with_size(Size::Small)"),
        "Theme Designer should keep the adopted ColorPicker configured as the row swatch"
    );
    assert!(
        !chooser.contains(".w(px(18.0))\n                            .h(px(18.0))"),
        "Theme Designer color rows should not render a duplicate manual color square"
    );
    assert!(
        !render_row.contains(".bg(rgb(_hex))")
            && !render_row.contains(".bg(rgb(hex))")
            && !render_row.contains(".bg(hsla("),
        "Theme Designer color row should not add a second hand-drawn color swatch"
    );
    assert!(
        sync_color.contains("Self::sync_color_picker_entity_value(&controls.picker, hex, window, cx);")
            && sync_color.contains("if input.focus_handle(cx).is_focused(window) {\n                return;\n            }")
            && sync_color.contains("input.set_value(next, window, cx);"),
        "focused typed hex input must not be overwritten during Theme Designer sync"
    );
}

#[test]
fn theme_chooser_edit_as_text_action_materializes_user_theme_before_opening_editor() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let actions = read_source("src/render_builtins/actions.rs");
    let collector = read_source("src/app_layout/collect_elements.rs");

    assert!(actions.contains("\"theme_chooser_edit_theme_as_text\""));
    assert!(actions.contains("\"Edit Theme as Text\""));
    assert!(actions.contains("IconName::Pencil"));
    assert!(collector.contains("button:theme-chooser-edit-theme-as-text"));
    assert!(chooser.contains("fn materialize_theme_chooser_text_edit_file("));
    assert!(chooser.contains("theme::user_themes::find_user_theme(&saved.slug)"));
    assert!(chooser.contains("theme::user_themes::find_user_theme(slug)"));
    assert!(chooser.contains("theme::user_themes::save_theme_as_user_theme(&snapshot.save_name"));
    assert!(chooser.contains("crate::script_creation::open_in_editor("));
    assert!(
        !chooser.contains("setup::theme_json_path") && !chooser.contains("theme.json"),
        "Edit-as-text should open a user-theme file, not the active ~/.scriptkit/theme.json override"
    );
}

#[test]
fn theme_chooser_slider_drag_preview_does_not_resync_native_vibrancy() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let gpui_integration = read_source("src/theme/gpui_integration.rs");
    let theme_service = read_source("src/theme/service.rs");
    let slider_change = chooser
        .split("fn apply_theme_chooser_slider_change(")
        .nth(1)
        .and_then(|section| section.split("fn apply_theme_chooser_color_change(").next())
        .expect("missing apply_theme_chooser_slider_change");
    let slider_preview = chooser
        .split("fn apply_theme_chooser_slider_theme(")
        .nth(1)
        .and_then(|section| {
            section
                .split("fn apply_theme_chooser_theme_preview(")
                .next()
        })
        .expect("missing apply_theme_chooser_slider_theme");
    let preview_helper = chooser
        .split("fn apply_theme_chooser_theme_preview(")
        .nth(1)
        .and_then(|section| section.split("fn apply_and_persist_theme(").next())
        .expect("missing apply_theme_chooser_theme_preview");
    let non_slider_preview = chooser
        .split("fn apply_theme_chooser_theme(")
        .nth(1)
        .and_then(|section| section.split("fn apply_theme_chooser_slider_theme(").next())
        .expect("missing apply_theme_chooser_theme");
    let persist_helper = chooser
        .split("fn apply_and_persist_theme(")
        .nth(1)
        .and_then(|section| section.split("fn mutate_theme_chooser_theme(").next())
        .expect("missing apply_and_persist_theme");
    let service_reload = theme_service
        .split("pub(crate) fn reload_theme_cache_sync_and_bump_revision(")
        .nth(1)
        .and_then(|section| section.split("/// Persist a theme to disk").next())
        .expect("missing reload_theme_cache_sync_and_bump_revision");

    assert!(
        slider_change.contains("self.apply_theme_chooser_slider_theme(")
            && slider_change.contains("self.mutate_theme_chooser_slider_theme("),
        "Theme Designer slider drags should route through the live slider preview path"
    );
    assert!(
        !slider_change.contains("self.apply_theme_chooser_theme(")
            && !slider_change.contains("self.mutate_theme_chooser_theme("),
        "Theme Designer slider drags must not route through native-sync preview helpers"
    );
    let slider_preview_compact = slider_preview
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        slider_preview_compact.contains(
            "self.apply_theme_chooser_theme_preview(next_theme, reason, false, mode.notify_parent(), cx);"
        ),
        "live slider previews should skip native vibrancy reconfiguration"
    );
    assert!(
        non_slider_preview.contains(
            "self.apply_theme_chooser_theme_preview(next_theme, reason, true, true, cx);"
        ),
        "non-slider theme previews should keep native vibrancy synchronized"
    );
    assert!(
        persist_helper.contains("self.apply_theme_chooser_theme(next_theme, reason, cx);")
            && persist_helper.contains("persist_theme_and_sync_all_windows"),
        "explicit Theme Designer commits should preview through the native-sync path before persisting"
    );
    assert!(
        service_reload.contains("sync_gpui_component_theme_for_theme_with_source("),
        "persisted themes should reload through the theme service path that syncs native window state"
    );
    assert!(
        preview_helper
            .contains("sync_theme_chooser_preview(cx, &self.theme, reason, sync_native_vibrancy);")
            && preview_helper.contains("if sync_native_vibrancy {")
            && preview_helper
                .contains("platform::configure_window_vibrancy_material_for_appearance")
            && preview_helper.contains("notify_parent: bool")
            && preview_helper.contains("if notify_parent {\n            cx.notify();"),
        "Theme Designer preview helper should honor the native-sync flag"
    );
    assert!(
        chooser.contains("Slider drags can skip native window material churn")
            && !chooser.contains(
                "applies both gpui-component colors and\n/// native vibrancy/material in one call"
            ),
        "Theme Designer preview sync comment should describe conditional native vibrancy ownership"
    );
    assert!(
        chooser.contains(
            "sync_gpui_component_theme_for_theme_with_source_and_native(\n        cx,\n        active_theme.as_ref(),\n        source,\n        sync_native_vibrancy,"
        ),
        "ThemeChooser preview sync must pass the slider native-sync flag into gpui theme integration"
    );
    assert!(
        gpui_integration.contains("sync_native_window: bool")
            && gpui_integration.contains("if sync_native_window {\n        sync_native_window_theme_for_theme(sk_theme, source);"),
        "gpui theme integration must allow high-frequency previews to skip native window reconfiguration"
    );
}

#[test]
fn theme_chooser_native_slider_drag_is_not_parent_reconciled_until_release() {
    let chooser = read_source("src/render_builtins/theme_chooser.rs");
    let slider = read_source("vendor/gpui-component/crates/ui/src/slider.rs");
    let new_slider = chooser
        .split("fn new_theme_chooser_slider(")
        .nth(1)
        .and_then(|section| section.split("fn new_theme_chooser_color_controls(").next())
        .expect("missing new_theme_chooser_slider");
    let sync_slider = chooser
        .split("fn sync_slider_entity_value(")
        .nth(1)
        .and_then(|section| section.split("fn sync_color_picker_entity_value(").next())
        .expect("missing sync_slider_entity_value");

    assert!(
        new_slider.contains("SliderEvent::Change(value)")
            && new_slider.contains("apply_theme_chooser_slider_drag_change(binding, *value, cx)"),
        "native slider Change must use the live-drag path"
    );
    assert!(
        new_slider.contains("SliderEvent::Release(value)")
            && new_slider.contains("apply_theme_chooser_slider_change(binding, *value, cx)"),
        "native slider Release must use the commit path"
    );
    assert!(
        chooser.contains("enum ThemeChooserSliderApplyMode")
            && chooser.contains("LiveDrag")
            && chooser.contains("Commit")
            && chooser.contains("mode.notify_parent()"),
        "ThemeChooser slider changes must distinguish live drag from release commit"
    );
    assert!(
        sync_slider.contains("slider.is_dragging()") && sync_slider.contains("return;"),
        "ThemeChooser must not sync model values into a slider while native drag is active"
    );
    assert!(
        slider.contains("Release(SliderValue)")
            && slider.contains("dragging: bool")
            && slider.contains("pub fn is_dragging(&self) -> bool")
            && slider.contains("fn handle_release(&mut self")
            && slider.contains(".on_mouse_up_out("),
        "vendor slider must expose drag lifecycle and clear active drag on mouse-up/out"
    );
}
