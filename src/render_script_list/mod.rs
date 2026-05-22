// Script list render method - extracted from app_render.rs
// This file is included via include!() macro in main.rs
use crate::ui_foundation::{
    is_key_down as sk_is_key_down, is_key_enter as sk_is_key_enter,
    is_key_escape as sk_is_key_escape, is_key_tab as sk_is_key_tab, is_key_up as sk_is_key_up,
};
use gpui_component::scroll::Scrollbar as GpuiScrollbar;

// --- merged from part_000.rs ---
fn app_shell_footer_colors(theme: &crate::theme::Theme) -> PromptFooterColors {
    PromptFooterColors::from_theme(theme)
}

fn script_list_footer_info_label(
    window_tweaker_enabled: bool,
    is_dark_mode: bool,
    opacity_percent: i32,
    material: &str,
    appearance: &str,
) -> Option<String> {
    if window_tweaker_enabled && !is_dark_mode {
        Some(format!(
            "{}% | {} | {} | ⌘-/+ ⌘M ⌘⇧A",
            opacity_percent, material, appearance
        ))
    } else {
        None
    }
}

fn inline_calc_list_item_title(formatted_result: &str) -> String {
    format!("= {}", formatted_result)
}

fn inline_calc_list_copy_hint() -> &'static str {
    "↵ Copy"
}

fn inline_calc_list_item_result_text_color(
    is_selected: bool,
    design_variant: DesignVariant,
    theme: &crate::theme::Theme,
    color_resolver: crate::theme::ColorResolver,
) -> u32 {
    if is_selected && design_variant != DesignVariant::Default {
        color_resolver.primary_accent()
    } else if is_selected {
        theme.colors.accent.selected
    } else {
        color_resolver.primary_text_color()
    }
}

fn inline_calc_list_item_hint_text_color(color_resolver: crate::theme::ColorResolver) -> u32 {
    color_resolver.empty_text_color()
}

#[derive(Clone, Copy)]
struct MenuSyntaxFormValueTypography {
    font_size: f32,
    line_height: f32,
}

fn menu_syntax_form_value_typography(design_variant: DesignVariant) -> MenuSyntaxFormValueTypography {
    let typography = get_tokens(design_variant).typography();
    let font_size = typography.font_size_sm;
    MenuSyntaxFormValueTypography {
        font_size,
        line_height: font_size * typography.line_height_normal,
    }
}

fn inline_calc_list_item_selected_overlay_rgba(
    theme: &crate::theme::Theme,
    color_resolver: crate::theme::ColorResolver,
) -> u32 {
    let selected_overlay_alpha =
        ((theme.get_opacity().selected.clamp(0.0, 1.0) * 255.0).round() as u32).max(0x2E);
    (color_resolver.primary_accent() << 8) | selected_overlay_alpha
}

fn render_inline_calc_list_item(
    calculator: &crate::calculator::CalculatorInlineResult,
    is_selected: bool,
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
    color_resolver: crate::theme::ColorResolver,
) -> AnyElement {
    let tokens = get_tokens(design_variant);
    let spacing = tokens.spacing();
    let typography = tokens.typography();

    let result_title = inline_calc_list_item_title(&calculator.formatted);
    let result_text_color =
        inline_calc_list_item_result_text_color(is_selected, design_variant, theme, color_resolver);
    let hint_text_color = inline_calc_list_item_hint_text_color(color_resolver);
    let hint_alpha = if is_selected { 0xD9 } else { 0x8C };

    div()
        .w_full()
        .h_full()
        .px(px(spacing.item_padding_x))
        .py(px(spacing.padding_xs))
        .when(is_selected, |div| {
            div.bg(rgba(inline_calc_list_item_selected_overlay_rgba(
                theme,
                color_resolver,
            )))
        })
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(spacing.gap_md))
        .child(
            div()
                .flex_1()
                .overflow_x_hidden()
                .text_size(px(typography.font_size_lg))
                .font_weight(typography.font_weight_semibold)
                .text_color(rgb(result_text_color))
                .child(result_title),
        )
        .child(
            div()
                .text_size(px(typography.font_size_xs))
                .text_color(rgba((hint_text_color << 8) | hint_alpha))
                .child(inline_calc_list_copy_hint()),
        )
        .into_any_element()
}

fn menu_syntax_hint_tone_color(
    theme: &crate::theme::Theme,
    tone: crate::menu_syntax::MenuSyntaxMainHintTone,
) -> u32 {
    match tone {
        crate::menu_syntax::MenuSyntaxMainHintTone::Neutral => theme.colors.text.secondary,
        crate::menu_syntax::MenuSyntaxMainHintTone::Accent => theme.colors.accent.selected,
        crate::menu_syntax::MenuSyntaxMainHintTone::Info => theme.colors.ui.info,
        crate::menu_syntax::MenuSyntaxMainHintTone::Warning => theme.colors.ui.warning,
        crate::menu_syntax::MenuSyntaxMainHintTone::Success => theme.colors.ui.success,
        crate::menu_syntax::MenuSyntaxMainHintTone::Muted => theme.colors.text.muted,
    }
}

fn menu_syntax_input_span_color(
    theme: &crate::theme::Theme,
    role: crate::menu_syntax::MenuSyntaxFragmentRole,
) -> u32 {
    match role {
        crate::menu_syntax::MenuSyntaxFragmentRole::Prefix
        | crate::menu_syntax::MenuSyntaxFragmentRole::Kv
        | crate::menu_syntax::MenuSyntaxFragmentRole::Tag
        | crate::menu_syntax::MenuSyntaxFragmentRole::Url => theme.colors.accent.selected,
        crate::menu_syntax::MenuSyntaxFragmentRole::ObjectRef => theme.colors.ui.success,
        crate::menu_syntax::MenuSyntaxFragmentRole::Date
        | crate::menu_syntax::MenuSyntaxFragmentRole::DateRange => theme.colors.ui.info,
        crate::menu_syntax::MenuSyntaxFragmentRole::Duration
        | crate::menu_syntax::MenuSyntaxFragmentRole::Priority => theme.colors.ui.warning,
        crate::menu_syntax::MenuSyntaxFragmentRole::Recurrence => theme.colors.ui.success,
        crate::menu_syntax::MenuSyntaxFragmentRole::Unresolved => theme.colors.text.muted,
        crate::menu_syntax::MenuSyntaxFragmentRole::Subject => theme.colors.text.primary,
    }
}

fn menu_syntax_input_span_role_name(
    role: crate::menu_syntax::MenuSyntaxFragmentRole,
) -> &'static str {
    crate::menu_syntax::input_span_role_name(role)
}

fn render_menu_syntax_hint_chip(
    theme: &crate::theme::Theme,
    chip: &crate::menu_syntax::MenuSyntaxMainHintChip,
) -> AnyElement {
    let color = menu_syntax_hint_tone_color(theme, chip.tone);
    div()
        .px(px(8.0))
        .py(px(3.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(rgba((color << 8) | 0x66))
        .bg(rgba((color << 8) | 0x18))
        .text_size(px(11.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(color))
        .child(chip.label.clone())
        .into_any_element()
}

fn render_menu_syntax_hint_row(
    theme: &crate::theme::Theme,
    row: &crate::menu_syntax::MenuSyntaxMainHintRow,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_start()
        .gap(px(12.0))
        .child(
            div()
                .w(px(76.0))
                .flex_shrink_0()
                .text_size(px(12.0))
                .line_height(px(18.0))
                .text_color(rgba((theme.colors.text.muted << 8) | 0xCC))
                .child(row.label.clone()),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .text_color(rgba((theme.colors.text.primary << 8) | 0xE6))
                .child(row.value.clone()),
        )
        .children(
            row.chips
                .iter()
                .map(|chip| render_menu_syntax_hint_chip(theme, chip)),
        )
        .into_any_element()
}

fn render_menu_syntax_fragment_preview_row(
    theme: &crate::theme::Theme,
    row: &crate::menu_syntax::MenuSyntaxFragmentPreviewRow,
) -> AnyElement {
    let color = menu_syntax_hint_tone_color(theme, row.tone);
    div()
        .w_full()
        .flex()
        .items_start()
        .gap(px(10.0))
        .child(
            div()
                .w(px(82.0))
                .flex_shrink_0()
                .px(px(7.0))
                .py(px(2.0))
                .rounded(px(5.0))
                .border_1()
                .border_color(rgba((color << 8) | 0x55))
                .bg(rgba((color << 8) | 0x14))
                .text_size(px(10.0))
                .line_height(px(14.0))
                .text_color(rgb(color))
                .child(format!("{:?}", row.role).to_ascii_lowercase()),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_size(px(12.0))
                .line_height(px(17.0))
                .text_color(rgba((theme.colors.text.primary << 8) | 0xE6))
                .child(format!("{}: {}", row.label, row.value)),
        )
        .children(
            row.chips
                .iter()
                .map(|chip| render_menu_syntax_hint_chip(theme, chip)),
        )
        .into_any_element()
}

fn render_menu_syntax_form_field(
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
    field: &crate::menu_syntax::MenuSyntaxFormFieldSnapshot,
    input: Option<Entity<gpui_component::input::InputState>>,
) -> AnyElement {
    let field_typography = menu_syntax_form_value_typography(design_variant);
    let border_color = if field.focused {
        rgba((theme.colors.accent.selected << 8) | 0xF2)
    } else {
        rgba((theme.colors.ui.border << 8) | 0x80)
    };
    let placeholder_color =
        rgba(crate::theme::AppChromeColors::from_theme(theme).placeholder_text_rgba);
    let input_height = crate::panel::CURSOR_HEIGHT_LG + (crate::panel::CURSOR_MARGIN_Y * 2.0);
    let mut field_node = div()
        .id(format!("menu-syntax-form-field-{}", field.id))
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(border_color)
        .bg(if field.focused {
            rgba((theme.colors.background.search_box << 8) | 0x3D)
        } else {
            rgba((theme.colors.background.search_box << 8) | 0x24)
        })
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgba((theme.colors.text.muted << 8) | 0xB3))
                        .child(field.label.clone()),
                ),
        );

    field_node = if let Some(input) = input {
        let input_element = gpui_component::input::Input::new(&input)
            .w_full()
            .h(px(input_height))
            .line_height(px(field_typography.line_height))
            .px(px(0.0))
            .py(px(0.0))
            .with_size(gpui_component::Size::Size(px(field_typography.font_size)))
            .appearance(false)
            .bordered(false)
            .focus_bordered(false);
        field_node.child(input_element)
    } else {
        let has_value = !field.value.trim().is_empty();
        let display_value = if has_value {
            field.value.clone()
        } else {
            field.placeholder.clone()
        };
        field_node.child(
            div()
                .w_full()
                .min_h(px(20.0))
                .flex()
                .items_center()
                .text_size(px(field_typography.font_size))
                .line_height(px(field_typography.line_height))
                .text_color(if has_value {
                    rgba((theme.colors.text.primary << 8) | 0xFF)
                } else {
                    placeholder_color
                })
                .child(
                    div()
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(display_value),
                ),
        )
    };

    field_node.into_any_element()
}

fn render_menu_syntax_form(
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
    form: &crate::menu_syntax::MenuSyntaxFormSnapshot,
    inputs: &[(String, Entity<gpui_component::input::InputState>)],
    _cx: &mut Context<ScriptListApp>,
) -> AnyElement {
    div()
        .id("menu-syntax-handler-form")
        .w_full()
        .flex()
        .items_start()
        .child(
            div()
                .id("menu-syntax-handler-form-fields")
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(form.fields.iter().map(|field| {
                    let input = inputs
                        .iter()
                        .find_map(|(id, input)| (id == &field.id).then(|| input.clone()));
                    render_menu_syntax_form_field(theme, design_variant, field, input)
                })),
        )
        .into_any_element()
}

fn render_menu_syntax_main_hint(
    hint: &crate::menu_syntax::MenuSyntaxMainHintSnapshot,
    scroll_handle: &ScrollHandle,
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
    form_inputs: &[(String, Entity<gpui_component::input::InputState>)],
    cx: &mut Context<ScriptListApp>,
) -> AnyElement {
    let accent = theme.colors.accent.selected;
    let border = theme.colors.ui.border;
    let body_text = rgba((theme.colors.text.secondary << 8) | 0xD9);
    let muted_text = rgba((theme.colors.text.muted << 8) | 0xB3);
    let examples = if hint.examples.is_empty() {
        hint.example.iter().cloned().collect::<Vec<_>>()
    } else {
        hint.examples.clone()
    };

    // Edge-to-edge: the hint content fills the main-list area directly with
    // no nested card chrome (no inner border, bg, or rounded corner). The
    // OUTER div carries the layout pad/centering so content has breathing
    // room, but there's no longer a visible "card within a card" — see
    // story `hint-card-fills-main-window-no-nested-container` (Run 12).
    let _ = border; // chrome dropped; kept binding to avoid widening unused-warning set
    gpui_component::scroll::ScrollableElement::vertical_scrollbar(
        div()
            .id("menu-syntax-main-hint-scroll")
            .w_full()
            .h_full()
            .flex()
            .items_start()
            .justify_center()
            .px(px(18.0))
            .pt(px(12.0))
            .pb(main_list_footer_overlay_total_padding() + px(12.0))
            .track_scroll(scroll_handle)
            .overflow_y_scroll(),
        scroll_handle,
    )
    .child(
        div()
            .w_full()
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(19.0))
                            .line_height(px(24.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(theme.colors.text.primary))
                            .child(hint.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .when_some(hint.mode_chip.as_ref(), |d, chip| {
                                d.child(render_menu_syntax_hint_chip(theme, chip))
                            })
                            // Multi-chip capture-validation row (Pass 22 added the data,
                            // Pass 23 wires the rendering): when `status_chips` is non-empty
                            // the snapshot has already inserted the mode chip at index 0 plus
                            // per-missing-field chips. Skip the mode_chip rendered above to
                            // avoid duplication, and render the rest.
                            .when(!hint.status_chips.is_empty(), |d| {
                                let skip = if hint.mode_chip.is_some() { 1 } else { 0 };
                                d.children(
                                    hint.status_chips
                                        .iter()
                                        .skip(skip)
                                        .map(|chip| render_menu_syntax_hint_chip(theme, chip)),
                                )
                            })
                            // Single-chip status (legacy non-capture path) — only render
                            // when the multi-chip path didn't already populate. Prevents
                            // double-chips on capture composer surfaces.
                            .when_some(
                                hint.status_chip
                                    .as_ref()
                                    .filter(|_| hint.status_chips.is_empty()),
                                |d, chip| d.child(render_menu_syntax_hint_chip(theme, chip)),
                            ),
                    ),
            )
            .when_some(hint.subtitle.as_ref(), |d, subtitle| {
                d.child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(19.0))
                        .text_color(body_text)
                        .child(subtitle.clone()),
                )
            })
            .when_some(hint.form.as_ref(), |d, form| {
                d.child(render_menu_syntax_form(
                    theme,
                    design_variant,
                    form,
                    form_inputs,
                    cx,
                ))
            })
            .when(!hint.rows.is_empty(), |d| {
                d.child(
                    div().flex().flex_col().gap(px(7.0)).children(
                        hint.rows
                            .iter()
                            .map(|row| render_menu_syntax_hint_row(theme, row)),
                    ),
                )
            })
            .when_some(hint.fragment_preview.as_ref(), |d, preview| {
                d.when(!preview.rows.is_empty(), |d| {
                    d.child(
                        div().flex().flex_col().gap(px(6.0)).children(
                            preview
                                .rows
                                .iter()
                                .map(|row| render_menu_syntax_fragment_preview_row(theme, row)),
                        ),
                    )
                })
            })
            .when_some(hint.warning.as_ref(), |d, warning| {
                d.child(
                    div()
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(rgba((theme.colors.ui.warning << 8) | 0x66))
                        .bg(rgba((theme.colors.ui.warning << 8) | 0x14))
                        .px(px(10.0))
                        .py(px(7.0))
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(rgb(theme.colors.ui.warning))
                        .child(warning.clone()),
                )
            })
            .child(div().h(px(1.0)).w_full().bg(rgba((border << 8) | 0x66)))
            .when_some(hint.primary_hint.as_ref(), |d, primary| {
                d.child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(accent))
                        .child(primary.clone()),
                )
            })
            .when_some(hint.secondary_hint.as_ref(), |d, secondary| {
                d.child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(muted_text)
                        .child(secondary.clone()),
                )
            })
            .when(!examples.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .line_height(px(14.0))
                                .text_color(rgba((theme.colors.text.muted << 8) | 0xB3))
                                .child(if examples.len() == 1 {
                                    "Example"
                                } else {
                                    "Examples"
                                }),
                        )
                        .child(
                            div()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(rgba((border << 8) | 0x66))
                                .bg(rgba((theme.colors.background.search_box << 8) | 0x4D))
                                .px(px(10.0))
                                .py(px(7.0))
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .text_size(px(12.0))
                                .line_height(px(17.0))
                                .font_family("JetBrains Mono")
                                .text_color(rgba((theme.colors.text.secondary << 8) | 0xE6))
                                .children(examples.iter().map(|example| {
                                    div()
                                        .w_full()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(example.clone())
                                })),
                        ),
                )
            }),
    )
    .into_any_element()
}

fn render_script_list_empty_state(
    filter_text_for_render: &str,
    has_active_filter: bool,
    empty_text_color: u32,
    empty_font_family: String,
) -> AnyElement {
    // Empty state rendering with icon and helpful messaging:
    // empty filter shows creation help; non-empty filter shows search recovery.
    use crate::designs::icon_variations::IconName;

    if filter_text_for_render.is_empty() {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(EMPTY_STATE_GAP))
            .font_family(empty_font_family)
            .child(
                svg()
                    .external_path(IconName::Code.external_path())
                    .size(px(EMPTY_STATE_ICON_SIZE))
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_ICON)),
            )
            .child(
                div()
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_MESSAGE))
                    .text_size(px(EMPTY_STATE_MESSAGE_FONT_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .child("No scripts or snippets found"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_HINT))
                    .child("Press ⌘N to create a new script"),
            )
            .into_any_element()
    } else {
        // Filtering but no results (including no fallbacks) - shouldn't normally happen.
        let filter_display = if filter_text_for_render.chars().count() > 30 {
            format!(
                "{}...",
                crate::utils::truncate_str_chars(filter_text_for_render, 27)
            )
        } else {
            filter_text_for_render.to_string()
        };
        let plain_hash_search = filter_text_for_render.starts_with('#')
            && filter_text_for_render
                .chars()
                .skip(1)
                .all(|ch| !ch.is_whitespace());
        let recovery_hint = if has_active_filter {
            "There are no search results with this filter applied."
        } else if plain_hash_search {
            "Plain #tag is launcher search. Use :#tag to filter tags, or ;todo ... #tag to label a capture."
        } else {
            "Try a different search term or press ⌘↵ to ask AI"
        };
        let syntax_tips = if has_active_filter {
            "Edit or remove a filter chip to widen the search."
        } else if plain_hash_search {
            "Examples: :#work · :tag:work · ;todo Buy milk #errands"
        } else {
            "Filters: type:script · type:scriptlet · shortcut:cmd+k · ;todo · ;note"
        };
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(EMPTY_STATE_GAP))
            .font_family(empty_font_family)
            .child(
                svg()
                    .external_path(IconName::MagnifyingGlass.external_path())
                    .size(px(EMPTY_STATE_ICON_SIZE))
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_ICON)),
            )
            .child(
                div()
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_MESSAGE))
                    .text_size(px(EMPTY_STATE_MESSAGE_FONT_SIZE))
                    .font_weight(FontWeight::MEDIUM)
                    .child(format!("No results for \"{}\"", filter_display)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_HINT))
                    .child(recovery_hint),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_TIPS))
                    .pt(px(EMPTY_STATE_TIPS_MARGIN_TOP))
                    .child(syntax_tips),
            )
            .into_any_element()
    }
}

impl ScriptListApp {
    fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let render_list_start = std::time::Instant::now();
        let filter_for_log = self.filter_text.clone();
        let is_mini = self.main_window_mode == MainWindowMode::Mini;

        // Get grouped or flat results based on filter state (cached) - MUST come first
        // to avoid borrow conflicts with theme access below
        // When filter is empty, use frecency-grouped results with RECENT/MAIN sections
        // When filtering, use flat fuzzy search results
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let get_results_elapsed = render_list_start.elapsed();

        // Deduplicate render logs: only log when meaningful state changes (not cursor blink)
        // This reduces log spam from ~2 logs/sec (cursor blink) to only on actual changes
        let state_changed = self.filter_text
            != self.main_menu_render_diagnostics.last_render_log_filter
            || self.selected_index != self.main_menu_render_diagnostics.last_render_log_selection
            || grouped_items.len() != self.main_menu_render_diagnostics.last_render_log_item_count;

        // Set flag for render_preview_panel to check (called later in this render)
        self.main_menu_render_diagnostics.log_this_render = state_changed;
        // Capture item count for deferred state update
        let item_count_for_log = grouped_items.len();

        if state_changed && logging::filter_perf_trace_enabled() {
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_SCRIPT_LIST_START] filter='{}' computed_filter='{}' selected_idx={}",
                    filter_for_log, self.computed_filter_text, self.selected_index
                ),
            );
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_GET_RESULTS] filter='{}' items={} results={} took={:.2}ms",
                    filter_for_log,
                    grouped_items.len(),
                    flat_results.len(),
                    get_results_elapsed.as_secs_f64() * 1000.0
                ),
            );
        }

        // NOTE: Removed per-frame logging here - was causing 6 log calls per frame
        // which includes mutex locks and file I/O. Log only on cache MISS in get_grouped_results_cached.
        // Clone for use in closures and to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // --- Storybook live-spec override (read-only, no state mutation) ---
        // grouped_items / flat_results are Arc<[T]> from cache; when the storybook
        // spec requires overrides we swap them for fresh Vecs wrapped in Arc.
        #[cfg(feature = "storybook")]
        let (grouped_items, flat_results, selected_index_for_render, filter_text_for_render) = {
            let live_spec = crate::storybook::adopted_main_menu_live_spec();
            let mut si = self.selected_index;
            let mut ft = self.filter_text.clone();
            let gi;
            let fr;
            if live_spec.force_empty_results {
                gi = std::sync::Arc::<[GroupedListItem]>::from(Vec::new());
                fr = std::sync::Arc::<[scripts::SearchResult]>::from(Vec::new());
                ft = live_spec
                    .filter_text_override
                    .unwrap_or("storybook-empty")
                    .to_string();
                si = 0;
            } else {
                if live_spec.prefer_first_result_selected {
                    if let Some(ix) = grouped_items
                        .iter()
                        .position(|item| matches!(item, GroupedListItem::Item(_)))
                    {
                        si = ix;
                    }
                }
                gi = grouped_items;
                fr = flat_results;
            }
            (gi, fr, si, ft)
        };
        #[cfg(not(feature = "storybook"))]
        let (selected_index_for_render, filter_text_for_render) =
            (self.selected_index, self.filter_text.clone());

        // Get design tokens for current design variant
        let tokens = get_tokens(self.current_design);
        let design_visual = tokens.visual();

        // Unified color, typography, and spacing resolution
        // Shell uses theme-first so non-default design variants keep the active
        // theme's colors while still using the variant's spacing and shape tokens.
        let color_resolver =
            crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design);
        let spacing_resolver = crate::theme::SpacingResolver::new(self.current_design);

        // For Default design, use header constants; for others, use spacing resolver
        let is_default_design = self.current_design == DesignVariant::Default;
        let design_spacing = tokens.spacing();

        let item_count = grouped_items.len();
        let _total_len = self.scripts.len() + self.scriptlets.len();

        // ============================================================
        // RENDER IS READ-ONLY
        // ============================================================
        // NOTE: State mutations (selection validation, list sync) are now done
        // in event handlers via sync_list_state() and validate_selection_bounds(),
        // not during render. This prevents the anti-pattern of mutating state
        // during render which can cause infinite render loops and inconsistent UI.
        //
        // Event handlers that call these methods:
        // - queue_filter_compute() - after filter text changes
        // - set_filter_text_immediate() - for immediate filter updates
        // - refresh_scripts() - after script reload
        // - reset_to_script_list() - on view transitions

        // ============================================================
        // IMMUTABLE BORROWS BLOCK - extract theme values for UI building
        // ============================================================

        // Extract theme values as owned copies for UI building
        let log_panel_bg = self.theme.colors.background.log_panel;
        let log_panel_border = self.theme.colors.ui.border;
        let log_panel_success = self.theme.colors.ui.success;

        // Pre-compute list item colors for closure (Copy type)
        let theme_colors = ListItemColors::from_theme(&self.theme);

        // NOTE: Removed P4 perf log - called every render frame, causing log spam

        // Build script list using uniform_list for proper virtualized scrolling
        // Use unified color resolver for consistent empty state styling
        let empty_text_color = color_resolver.empty_text_color();
        let empty_font_family = typography_resolver.primary_font().to_string();
        let handler_form_owns_input_for_render =
            self.menu_syntax_capture_form_owns_input_for(&filter_text_for_render);
        let show_launcher_ask_ai_hint = !handler_form_owns_input_for_render;
        let popup_owns_main_list = !handler_form_owns_input_for_render
            && (self.menu_syntax_object_selector_state.owns_main_list()
                || self.menu_syntax_trigger_popup_state.owns_main_list());
        let menu_syntax_owns_main_list = handler_form_owns_input_for_render
            || popup_owns_main_list
            || self
                .menu_syntax_mode
                .command_owns_input_for(&filter_text_for_render);
        // Keep guide cards available for bare/partial `:` refine entry, but
        // let completed advanced queries render their filtered results first.
        // Empty-state hint cards are handled by the item-count branch below.
        let advanced_query_guide_hint = (!menu_syntax_owns_main_list)
            .then(|| self.menu_syntax_main_hint_snapshot(&filter_text_for_render, true))
            .flatten()
            .filter(|hint| {
                matches!(
                    hint.kind,
                    crate::menu_syntax::MenuSyntaxMainHintKind::AdvancedQueryGuide
                )
            });

        let list_element: AnyElement = if menu_syntax_owns_main_list {
            self.menu_syntax_main_hint_snapshot(&filter_text_for_render, false)
                .map(|hint| {
                    render_menu_syntax_main_hint(
                        &hint,
                        &self.menu_syntax_main_hint_scroll_handle,
                        &self.theme,
                        self.current_design,
                        &self.menu_syntax_form_inputs,
                        cx,
                    )
                })
                .unwrap_or_else(|| div().w_full().h_full().into_any_element())
        } else if let Some(hint) = advanced_query_guide_hint {
            render_menu_syntax_main_hint(
                &hint,
                &self.menu_syntax_main_hint_scroll_handle,
                &self.theme,
                self.current_design,
                &self.menu_syntax_form_inputs,
                cx,
            )
        } else if item_count == 0 {
            if let Some(hint) = self.menu_syntax_main_hint_snapshot(&filter_text_for_render, true) {
                render_menu_syntax_main_hint(
                    &hint,
                    &self.menu_syntax_main_hint_scroll_handle,
                    &self.theme,
                    self.current_design,
                    &self.menu_syntax_form_inputs,
                    cx,
                )
            } else {
                let has_active_filter = self
                    .menu_syntax_mode
                    .advanced_query_for(&filter_text_for_render)
                    .is_some_and(|query| query.has_source_filters() || query.has_predicates());
                render_script_list_empty_state(
                    &filter_text_for_render,
                    has_active_filter,
                    empty_text_color,
                    empty_font_family,
                )
            }
        } else {
            // Use GPUI's list() component for variable-height items
            // Section headers render at 32px, regular items at 40px
            // This gives true visual compression for headers without the uniform_list hack

            // Clone grouped_items and flat_results for the closure
            let grouped_items_clone = grouped_items.clone();
            let flat_results_clone = flat_results.clone();

            let effective_section_header_height =
                crate::list_item::effective_section_header_height();
            let effective_list_item_height = crate::list_item::effective_list_item_height();

            // Capture entity handle for use in the render closure
            let entity = cx.entity();

            // theme_colors was pre-computed above to avoid borrow conflicts
            let current_design = self.current_design;

            // Track filter for closure logging and highlighting
            let filter_for_closure = filter_text_for_render.clone();
            let filter_for_highlight = filter_text_for_render.clone();

            // Capture selected index for render (may be overridden by storybook live spec)
            let selected_for_list_closure = selected_index_for_render;
            let footer_padding = main_list_footer_overlay_total_padding();
            let row_generation = self.main_list_row_generation;

            let variable_height_list =
                list(self.main_list_state.clone(), move |ix, _window, cx| {
                    let _item_render_start = std::time::Instant::now();

                    // Access entity state inside the closure
                    entity.update(cx, |this, cx| {
                        let current_selected = selected_for_list_closure;
                        let current_hovered = this.hovered_index;

                        if let Some(grouped_item) = grouped_items_clone.get(ix) {
                            match grouped_item {
                                GroupedListItem::SectionHeader(label, icon) => {
                                    // Section header at 32px height (8px grid) for clear visual separation,
                                    // or 20px if it is the first section header to pull it up closer to input.
                                    let is_first = ix == 0;
                                    let h_px = if is_first {
                                        crate::list_item::effective_first_section_header_height()
                                    } else {
                                        effective_section_header_height
                                    };
                                    div()
                                        .id(ElementId::NamedInteger(
                                            format!("section-header-gen-{row_generation}").into(),
                                            ix as u64,
                                        ))
                                        .h(px(h_px))
                                        .child(render_section_header(label, icon.as_deref(), theme_colors, is_first))
                                        .into_any_element()
                                }
                                GroupedListItem::Status(status) => {
                                    div()
                                        .id(ElementId::NamedInteger(
                                            format!("source-status-gen-{row_generation}").into(),
                                            ix as u64,
                                        ))
                                        .h(px(crate::list_item::effective_source_status_row_height()))
                                        .px_4()
                                        .flex()
                                        .items_center()
                                        .text_sm()
                                        .text_color(rgb(theme_colors.text_secondary))
                                        .child(status.label.clone())
                                        .into_any_element()
                                }
                                GroupedListItem::Item(result_idx) => {
                                    // Regular item at 40px height (LIST_ITEM_HEIGHT)
                                    let is_selected = ix == current_selected;
                                    // Hover gating is now handled by ListItem via GPUI input modality
                                    let is_hovered = current_hovered == Some(ix);

                                    // Create hover handler
                                    let hover_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              hovered: &bool,
                                              _window,
                                              cx| {
                                            if *hovered {
                                                this.input_mode = InputMode::Mouse;
                                                if this.hovered_index != Some(ix) {
                                                    this.hovered_index = Some(ix);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                this.hovered_index = None;
                                                cx.notify();
                                            }
                                        },
                                    );

                                    // Create click handler matching launcher click semantics
                                    let click_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              event: &gpui::ClickEvent,
                                              _window,
                                              cx| {
                                            let was_selected = this.selected_index == ix;
                                            // Always select the item on any click
                                            if !was_selected {
                                                this.selected_index = ix;
                                                cx.notify();
                                            }

                                            let click_count = event.click_count();
                                            if crate::ui_foundation::should_submit_selected_row_click(
                                                was_selected,
                                                click_count,
                                            ) {
                                                logging::log(
                                                    "UI",
                                                    &format!(
                                                        "Launcher row click submitting item {} (click_count={})",
                                                        ix, click_count
                                                    ),
                                                );
                                                this.execute_selected(cx);
                                            }
                                        },
                                    );

                                    // Dispatch to design-specific item renderer
                                    // Note: Confirmation for dangerous builtins is now handled
                                    // via modal dialog, not inline overlay
                                    let design_render_start = std::time::Instant::now();
                                    let inline_calculator =
                                        this.inline_calculator_for_result_index(*result_idx);
                                    let mut item_name = "inline-calculator";
                                    let item_element = if let Some(calculator) = inline_calculator
                                    {
                                        let _legacy_calculator_renderer = render_calculator_item;
                                        render_inline_calc_list_item(
                                            calculator,
                                            is_selected,
                                            &this.theme,
                                            this.current_design,
                                            color_resolver,
                                        )
                                    } else if let Some(result) = flat_results_clone.get(*result_idx)
                                    {
                                        item_name = result.name();
                                        render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
                                            &filter_for_highlight,
                                        )
                                    } else {
                                        item_name = "<missing-result>";
                                        div().h(px(effective_list_item_height)).into_any_element()
                                    };
                                    let design_elapsed = design_render_start.elapsed();

                                    // Log slow items (>1ms)
                                    if design_elapsed.as_micros() > 1000 {
                                        logging::log(
                                            "FILTER_PERF",
                                            &format!(
                                                "[SLOW_ITEM] ix={} name='{}' design_render={:.2}ms filter='{}'",
                                                ix,
                                                item_name,
                                                design_elapsed.as_secs_f64() * 1000.0,
                                                filter_for_closure
                                            ),
                                        );
                                    }

                                    div()
                                        .id(ElementId::NamedInteger(
                                            format!("script-item-gen-{row_generation}").into(),
                                            ix as u64,
                                        ))
                                        .h(px(effective_list_item_height))
                                        .on_hover(hover_handler)
                                        .on_click(click_handler)
                                        .child(item_element)
                                        .into_any_element()
                                }
                            }
                        } else {
                            // Fallback for out-of-bounds index
                            div().h(px(effective_list_item_height)).into_any_element()
                        }
                    })
                })
                // Enable proper scroll handling for mouse wheel/trackpad
                // ListSizingBehavior::Infer sets overflow.y = Overflow::Scroll internally
                // which is required for the list's hitbox to capture scroll wheel events
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .h_full()
                .pb(footer_padding);

            // Wrap list in a relative container with scrollbar overlay
            // CUSTOM SCROLL HANDLER: GPUI's list() component has issues measuring unmeasured items
            // (they appear as 0px height). This causes mouse scroll to fail to reach all items.
            // Solution: Intercept scroll wheel events and convert to index-based scrolling,
            // which works correctly like keyboard navigation does.
            //
            // Average item height for delta-to-index conversion:
            // Most items are LIST_ITEM_HEIGHT (40px), headers are SECTION_HEADER_HEIGHT (32px)
            // Use 44px as a reasonable average that feels natural for scrolling
            let avg_item_height = crate::list_item::effective_average_item_height_for_scroll();

            // Capture item count for scroll handler logging
            let scroll_item_count = item_count;
            self.sync_main_list_selection_to_visible_window("render");

            let scrollbar_overlay = {
                let footer_overlay_height = main_list_footer_overlay_total_padding();
                let viewport_height = self.main_list_state.viewport_bounds().size.height;
                let safe_viewport_height = (viewport_height - footer_overlay_height).max(px(0.0));
                let content_height = px(grouped_items
                    .iter()
                    .enumerate()
                    .map(|(ix, item)| match item {
                        GroupedListItem::SectionHeader(..) => {
                            if ix == 0 {
                                crate::list_item::effective_first_section_header_height()
                            } else {
                                crate::list_item::effective_section_header_height()
                            }
                        }
                        GroupedListItem::Status(..) => {
                            crate::list_item::effective_source_status_row_height()
                        }
                        GroupedListItem::Item(..) => crate::list_item::effective_list_item_height(),
                    })
                    .sum::<f32>());

                div()
                    .absolute()
                    .top_0()
                    .right_0()
                    .h(safe_viewport_height)
                    .w(px(16.0))
                    .child(
                        GpuiScrollbar::vertical(&self.main_list_state)
                            .id("launcher-main-scrollbar")
                            .scroll_size(size(px(0.0), content_height))
                            .into_any_element(),
                    )
                    .into_any_element()
            };

            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .h_full()
                .on_scroll_wheel(cx.listener(
                    move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                        if scroll_item_count == 0 {
                            return;
                        }
                        // The script list owns wheel-driven scrolling. Stop propagation
                        // before any selection math so GPUI's native list scroll path
                        // cannot drift the viewport away from the active row.
                        cx.stop_propagation();

                        let selected_before = this.selected_index;
                        let scroll_top_before = this.main_list_state.logical_scroll_top();
                        let wheel_accum_before = this.wheel_accum;

                        // Convert scroll delta to lines/items
                        // Lines: direct item count, Pixels: convert based on average item height
                        let delta_lines: f32 = match event.delta {
                            gpui::ScrollDelta::Lines(point) => point.y,
                            gpui::ScrollDelta::Pixels(point) => {
                                // Convert pixels to items using average item height
                                let pixels: f32 = point.y.into();
                                pixels / avg_item_height
                            }
                        };

                        // Accumulate smoothly for high-resolution trackpads
                        // Invert so scroll down (negative delta) moves selection down (positive)
                        this.wheel_accum += -delta_lines;

                        // Only apply integer steps when magnitude crosses 1.0
                        // This preserves smooth scrolling feel on trackpads
                        let steps = this.wheel_accum.trunc() as i32;
                        if steps != 0 {
                            // Subtract the applied steps from accumulator
                            this.wheel_accum -= steps as f32;

                            // Use the existing move_selection_by which handles section headers
                            // and properly updates scroll via scroll_to_selected_if_needed
                            this.move_selection_by(steps, cx);
                        }

                        let scroll_top_after = this.main_list_state.logical_scroll_top();
                        this.sync_main_list_selection_to_visible_window("wheel");
                        // doc-anchor-removed: [[design#Footer-safe list reveal]]
                        tracing::debug!(
                            target: "SCROLL_STATE",
                            delta_lines,
                            steps,
                            total_items = scroll_item_count,
                            selected_before,
                            selected_after = this.selected_index,
                            scroll_top_before = scroll_top_before.item_ix,
                            scroll_top_after = scroll_top_after.item_ix,
                            offset_before_px = scroll_top_before.offset_in_item.as_f32(),
                            offset_after_px = scroll_top_after.offset_in_item.as_f32(),
                            wheel_accum_before,
                            wheel_accum_after = this.wheel_accum,
                            propagation_stopped = true,
                            "script list wheel handled"
                        );
                    },
                ))
                .child(variable_height_list)
                .child(scrollbar_overlay)
                .into_any_element()
        };

        // Log panel - uses pre-extracted theme values to avoid borrow conflicts
        let log_panel = if self.show_logs {
            let logs = logging::get_last_logs(10);
            let mut log_container = div()
                .flex()
                .flex_col()
                .w_full()
                .bg(rgb(log_panel_bg))
                .border_t_1()
                .border_color(rgb(log_panel_border))
                .p(px(design_spacing.padding_md))
                .max_h(px(LOG_PANEL_MAX_HEIGHT))
                .font_family(FONT_MONO);

            for log_line in logs.iter().rev() {
                log_container = log_container.child(
                    div()
                        .text_color(rgb(log_panel_success))
                        .text_xs()
                        .child(log_line.clone()),
                );
            }
            Some(log_container)
        } else {
            None
        };

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                if event.keystroke.modifiers.platform {
                    tracing::debug!(
                        event = "script_list.key_down",
                        key = %event.keystroke.key,
                        cmd = true,
                        shift = event.keystroke.modifiers.shift,
                        mini_mode = (this.main_window_mode == MainWindowMode::Mini),
                        "script_list key_down: cmd+{}",
                        event.keystroke.key,
                    );
                }
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W only - ScriptList has special ESC handling below)
                if this.handle_global_shortcut_with_options(event, false, cx) {
                    return;
                }

                let key_str = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                if matches!(this.current_view, AppView::ScriptList)
                    && this.handle_menu_syntax_form_control_key_input(
                        key_str,
                        key_char,
                        &event.keystroke.modifiers,
                        window,
                        cx,
                    )
                {
                    cx.stop_propagation();
                    return;
                }
                if sk_is_key_tab(key_str)
                    && !has_cmd
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                    && this.menu_syntax_capture_form_owns_input()
                {
                    if event.keystroke.modifiers.shift {
                        this.focus_previous_menu_syntax_form_field(window, cx);
                    } else {
                        this.focus_next_menu_syntax_form_field(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                // This allows scripts to override default shortcuts via setActions()
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        shortcuts::keystroke_to_shortcut(key_str, &event.keystroke.modifiers);
                    if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                        logging::log(
                            "ACTIONS",
                            &format!(
                                "SDK action shortcut matched: '{}' -> '{}'",
                                key_combo, action_name
                            ),
                        );
                        if this.trigger_action_by_name(&action_name, cx) {
                            return;
                        }
                    }
                }

                // If actions popup is open, route all keyboard events through the shared router
                match this.route_key_to_actions_dialog(
                    key_str,
                    key_char,
                    &event.keystroke.modifiers,
                    ActionsDialogHost::MainList,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute {
                        action_id,
                        should_close,
                    } => {
                        this.execute_actions_route_action(
                            ActionsDialogHost::MainList,
                            action_id,
                            should_close,
                            window,
                            cx,
                        );
                        cx.notify();
                        return;
                    }
                    ActionsRoute::Handled => return,
                    ActionsRoute::NotHandled => {}
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str {
                        "v" if this.route_large_script_list_paste_to_acp(cx) => {
                            logging::log("KEY", "Shortcut Cmd+V -> route_large_script_list_paste_to_acp");
                            cx.stop_propagation();
                            return;
                        }
                        "l" => {
                            logging::log("KEY", "Shortcut Cmd+L -> toggle_logs");
                            this.toggle_logs(cx);
                            return;
                        }
                        // Cmd+1 cycles through all designs
                        "1" => {
                            logging::log("KEY", "Shortcut Cmd+1 -> cycle_design");
                            this.cycle_design(cx);
                            return;
                        }
                        // Script context shortcuts (require a selected script)
                        // Note: More specific patterns (with shift) must come BEFORE less specific ones
                        "k" if has_shift => {
                            // Cmd+Shift+K - Add/Update Keyboard Shortcut
                            logging::log("KEY", "Shortcut Cmd+Shift+K -> add_shortcut");
                            this.handle_action("add_shortcut".to_string(), window, cx);
                            return;
                        }
                        "k" => {
                            // Cmd+K - Toggle actions menu (routed through shared dispatcher)
                            logging::log("KEY", "Shortcut Cmd+K -> handle_cmd_k_actions_toggle");
                            this.handle_cmd_k_actions_toggle(window, cx);
                            return;
                        }
                        "i" => {
                            // Cmd+I - Toggle Info Panel
                            logging::log("KEY", "Shortcut Cmd+I -> toggle_info");
                            this.handle_action("toggle_info".to_string(), window, cx);
                            return;
                        }
                        "e" => {
                            // Cmd+E - Edit Script
                            logging::log("KEY", "Shortcut Cmd+E -> edit_script");
                            this.handle_action("edit_script".to_string(), window, cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            logging::log("KEY", "Shortcut Cmd+Shift+F -> reveal_in_finder");
                            this.handle_action("reveal_in_finder".to_string(), window, cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            logging::log("KEY", "Shortcut Cmd+Shift+C -> copy_path");
                            this.handle_action("copy_path".to_string(), window, cx);
                            return;
                        }
                        "d" if has_shift => {
                            // Cmd+Shift+D - Copy Deeplink
                            logging::log("KEY", "Shortcut Cmd+Shift+D -> copy_deeplink");
                            this.handle_action("copy_deeplink".to_string(), window, cx);
                            return;
                        }
                        "a" if has_shift => {
                            // Cmd+Shift+A - Add/Update Alias
                            logging::log("KEY", "Shortcut Cmd+Shift+A -> add_alias");
                            this.handle_action("add_alias".to_string(), window, cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            logging::log("KEY", "Shortcut Cmd+N -> create_script");
                            this.handle_action("create_script".to_string(), window, cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            logging::log("KEY", "Shortcut Cmd+R -> reload_scripts");
                            this.handle_action("reload_scripts".to_string(), window, cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            logging::log("KEY", "Shortcut Cmd+, -> settings");
                            this.handle_action("settings".to_string(), window, cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            logging::log("KEY", "Shortcut Cmd+Q -> quit");
                            this.handle_action("quit".to_string(), window, cx);
                            return;
                        }
                        _ => {}
                    }
                }

                // Actions popup keyboard routing is handled above via route_key_to_actions_dialog

                // LEGACY: Check if we're in fallback mode (no script matches, showing fallback commands)
                // Note: This is legacy code that handled a separate fallback rendering path.
                // Now fallbacks flow through GroupedListItem from grouping.rs, so this
                // branch should rarely (if ever) be triggered. The normal navigation below
                // handles fallback items in the unified list.
                if this.main_menu_fallback_state.is_active() {
                    match key_str {
                        key if sk_is_key_up(key) => {
                            if this.main_menu_fallback_state.move_up() {
                                cx.notify();
                            }
                        }
                        key if sk_is_key_down(key) => {
                            if this.main_menu_fallback_state.move_down() {
                                cx.notify();
                            }
                        }
                        key if sk_is_key_enter(key) => {
                            if !this.gpui_input_focused {
                                this.execute_selected_fallback(cx);
                            }
                        }
                        key if sk_is_key_escape(key) => {
                            // Clear filter to exit fallback mode
                            this.clear_filter(window, cx);
                        }
                        _ => {}
                    }
                    return;
                }

                // Run 12 Pass 13 — `ai-proposal-accept-dismiss`. When the
                // inline AI proposal hint card is up, Tab/Enter accepts and
                // Esc dismisses BEFORE the legacy navigation/clear handlers
                // run. Modifier-bearing keys (Cmd+Enter etc.) fall through
                // so the proposal-generation chord still works.
                if this.pending_menu_syntax_ai_proposal.is_some()
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if sk_is_key_tab(key_str)
                        || (sk_is_key_enter(key_str) && !event.keystroke.modifiers.shift)
                    {
                        if this.try_apply_pending_menu_syntax_ai_proposal(
                            crate::menu_syntax_ai_apply::ProposalApplyAction::Accept,
                            window,
                            cx,
                        ) {
                            cx.stop_propagation();
                            return;
                        }
                    } else if sk_is_key_escape(key_str) {
                        if this.try_apply_pending_menu_syntax_ai_proposal(
                            crate::menu_syntax_ai_apply::ProposalApplyAction::Dismiss,
                            window,
                            cx,
                        ) {
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                // Normal script list navigation
                // NOTE: Arrow keys are now handled by the arrow_interceptor in app_impl.rs
                // which fires before the Input component can consume them. This allows
                // input history navigation + list navigation to work correctly.
                match key_str {
                    key if sk_is_key_enter(key)
                        && event.keystroke.modifiers.platform
                        && !event.keystroke.modifiers.shift
                        && !event.keystroke.modifiers.alt
                        && !event.keystroke.modifiers.control =>
                    {
                        if this.try_route_global_cmd_enter_to_acp_context_capture(cx) {
                            cx.stop_propagation();
                        }
                    }
                    key if sk_is_key_enter(key) => {
                        if !this.gpui_input_focused {
                            this.execute_selected(cx);
                        }
                    }
                    key if sk_is_key_escape(key) => {
                        // Escape order on ScriptList:
                        //   1. menu-syntax trigger popup visible → close popup
                        //      only, leave filter text untouched. Second Escape
                        //      then falls through to the normal clear-filter
                        //      branch below.
                        //   2. filter non-empty → clear filter.
                        //   3. launcher-origin surface → go back to the main launcher.
                        //   4. filter empty → hide main window.
                        if crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open() {
                            if this.apply_menu_syntax_object_selector_intent(
                                crate::menu_syntax::InlinePickerKeyIntent::Close,
                                window,
                                cx,
                            ) {
                                return;
                            }
                        }
                        if crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open() {
                            if this.apply_menu_syntax_trigger_popup_intent(
                                crate::menu_syntax::InlinePickerKeyIntent::Close,
                                window,
                                cx,
                            ) {
                                return;
                            }
                        }
                        // Clear filter first if there's text, otherwise close window
                        if !this.filter_text.is_empty() {
                            this.clear_filter(window, cx);
                        } else if this.opened_from_main_menu {
                            this.go_back_or_close(window, cx);
                        } else {
                            // Filter is empty - close window
                            this.close_and_reset_window(cx);
                        }
                    }
                    _ => {}
                }
            },
        );

        let handle_key_up = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyUpEvent,
                  _window: &mut Window,
                  _cx: &mut Context<Self>| {
                let key = event.keystroke.key.as_str();
                if sk_is_key_up(key) || sk_is_key_down(key) {
                    tracing::info!(
                        target: "script_kit::input_history",
                        event = "script_list_arrow_key_up",
                        key = %key,
                        selected_index = this.selected_index,
                        history_index = ?this.input_history.current_index(),
                        filter_len = this.filter_text.len(),
                    );
                }
            },
        );

        // Main container with system font and transparency
        // NOTE: Shadow disabled for vibrancy - shadows on transparent elements cause gray fill

        // Use unified color resolver for text and fonts
        let text_primary = color_resolver.primary_text_color();
        let font_family = self.theme_font_family();

        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        let input_highlight_ranges = crate::menu_syntax::input_spans_for_input_with_targets(
            &filter_text_for_render,
            &capture_targets,
        )
        .into_iter()
        .filter(|span| span.role != crate::menu_syntax::MenuSyntaxFragmentRole::Subject)
        .map(|span| {
            (
                span.range,
                rgb(menu_syntax_input_span_color(&self.theme, span.role)).into(),
                menu_syntax_input_span_role_name(span.role).to_string(),
            )
        })
        .collect();
        self.gpui_input_state.update(cx, |state, _cx| {
            state.set_highlight_ranges_with_roles(input_highlight_ranges);
        });

        // NOTE: No .bg() here - Root provides vibrancy background for ALL content
        // This ensures main menu, AI chat, and all prompts have consistent styling

        let mut main_div = div()
            .flex()
            .flex_col()
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h_full()
            .text_color(rgb(text_primary))
            .font_family(font_family)
            .key_context("script_list")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .on_key_up(handle_key_up)
            // Header: Search Input + Run + Actions + Logo
            // Use shared header layout constants for consistency with all prompts
            .child({
                // Use shared header constants for default design, design tokens for others
                let header_padding_x = if is_default_design {
                    if is_mini {
                        crate::window_resize::mini_layout::HEADER_PADDING_X
                    } else {
                        HEADER_PADDING_X
                    }
                } else {
                    design_spacing.padding_lg
                };
                let header_padding_y = if is_default_design {
                    if is_mini {
                        crate::window_resize::mini_layout::HEADER_PADDING_Y
                    } else {
                        HEADER_PADDING_Y
                    }
                } else {
                    design_spacing.padding_sm
                };
                let header_gap = if is_default_design {
                    HEADER_GAP
                } else {
                    design_spacing.gap_md
                };

                div()
                    .w_full()
                    .px(px(header_padding_x))
                    .py(px(header_padding_y))
                    .min_h(px(crate::panel::HEADER_BUTTON_HEIGHT))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(header_gap))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            // Search input with cursor and selection support
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .child(self.render_search_input()),
                            )
                            .when(show_launcher_ask_ai_hint, |d| {
                                d.child(crate::components::render_launcher_ask_ai_hint(chrome))
                            }),
                    )
            })
            // Divider between header and list content
            // Use unified resolver for border color and spacing
            .child({
                let divider_margin = if is_default_design {
                    DIVIDER_MARGIN_DEFAULT
                } else {
                    spacing_resolver.margin_lg()
                };
                let border_width = if is_default_design {
                    DIVIDER_BORDER_WIDTH_DEFAULT
                } else {
                    design_visual.border_thin
                };

                div()
                    .mx(px(divider_margin))
                    .h(px(border_width))
                    .bg(rgba(chrome.divider_rgba))
            });

        if is_mini {
            // Mini mode: single column, toggle between list and info panel
            if self.show_info_panel {
                // Info panel replaces the list when toggled via Cmd+I
                let info_panel = self.render_preview_panel(cx);
                main_div = main_div.child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .w_full()
                        .overflow_hidden()
                        .child(div().w_full().h_full().min_h(px(0.)).child(info_panel)),
                );
            } else {
                main_div = main_div.child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .w_full()
                        .overflow_hidden()
                        .child(div().w_full().h_full().min_h(px(0.)).child(list_element)),
                );
            }

            if let Some(panel) = log_panel {
                main_div = main_div.child(panel);
            }

            // Hover blocker for the native footer zone. Uses deferred() so the
            // hitbox is appended LAST (checked FIRST in GPUI's reverse-order
            // hit test). block_mouse_except_scroll tells the hit test to exclude
            // elements behind this hitbox from hover while allowing scroll.
            // No background/border = nothing rendered into Metal layer = native
            // NSVisualEffectView blur shows through.
            main_div = main_div.child(gpui::deferred(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .w_full()
                    .h(px(
                        crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT,
                    ))
                    .block_mouse_except_scroll(),
            ));

            if state_changed && crate::logging::filter_perf_trace_enabled() {
                tracing::info!(
                    target: "script_kit::prompt_chrome",
                    event = "script_list_mini_ai_hint_rendered",
                    tab_hint = show_launcher_ask_ai_hint,
                    cmd_enter_hint = false,
                    "Mini ScriptList header rendered Ask AI keyboard hints"
                );
                let total_elapsed = render_list_start.elapsed();
                tracing::info!(
                    target: "RENDER_PERF",
                    category = "mini_render",
                    event = "render_script_list_end",
                    filter = %filter_for_log,
                    item_count = item_count_for_log,
                    selected_index = self.selected_index,
                    total_ms = format_args!("{:.2}", total_elapsed.as_secs_f64() * 1000.0),
                    mode = "mini",
                    "mini script list render complete"
                );
                self.main_menu_render_diagnostics.last_render_log_filter = self.filter_text.clone();
                self.main_menu_render_diagnostics.last_render_log_selection = self.selected_index;
                self.main_menu_render_diagnostics.last_render_log_item_count = item_count_for_log;
            }

            return main_div.into_any_element();
        }

        // Main content area: list takes full width unless info panel is toggled (Cmd+I)
        {
            let content_row = div()
                .flex()
                .flex_row()
                .flex_1()
                .min_h(px(0.)) // Critical: allows flex container to shrink properly
                .w_full()
                .overflow_hidden()
                // Left side: Script list — full width when info hidden, 50% when shown
                .child(
                    div()
                        .when(self.show_info_panel, |d| d.w_1_2())
                        .when(!self.show_info_panel, |d| d.w_full())
                        .h_full()
                        .min_h(px(0.))
                        .child(list_element),
                )
                // Right side: Info panel (50% width), only rendered when toggled
                .when(self.show_info_panel, |row| {
                    let preview_start = std::time::Instant::now();
                    let preview_panel = self.render_preview_panel(cx);
                    let preview_elapsed = preview_start.elapsed();
                    if state_changed {
                        logging::log(
                            "PREVIEW_PERF",
                            &format!(
                                "[PREVIEW_PANEL_DONE] filter='{}' took {:.2}ms",
                                filter_for_log,
                                preview_elapsed.as_secs_f64() * 1000.0
                            ),
                        );
                    }
                    row.child(
                        div()
                            .relative()
                            .flex()
                            .flex_col()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .when_some(
                                self.cached_main_window_preflight.clone(),
                                |d, receipt| {
                                    d.child(
                                        crate::main_window_preflight::render_main_window_preflight_receipt(
                                            self,
                                            &receipt,
                                        ),
                                    )
                                },
                            )
                            .child(div().flex_1().min_h(px(0.)).child(preview_panel)),
                    )
                });
            main_div = main_div.child(content_row);
        }

        // Footer: Universal three-key hint strip — ↵ Run · ⌘K Actions · ⌘↵ AI
        {
            let primary_label = self.main_window_primary_action_label();
            let hints =
                crate::components::universal_prompt_hints_with_primary_label(&primary_label);
            crate::components::emit_prompt_hint_audit("render_script_list::full", &hints);
            tracing::info!(
                target: "script_kit::prompt_chrome",
                event = "script_list_footer_unified",
                mode = "full",
                primary_label = %primary_label,
                "Script list footer rendered with selected enter-action label"
            );
            let gpui_footer =
                crate::components::render_universal_prompt_hint_strip_clickable_with_primary_label(
                    &primary_label,
                    cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                        this.dispatch_main_window_footer_action(
                            crate::footer_popup::FooterAction::Run,
                            window,
                            cx,
                            "gpui_footer_click",
                        );
                    }),
                    cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                        if this.has_actions()
                            || this.show_actions_popup
                            || crate::actions::is_actions_window_open()
                        {
                            this.toggle_actions(cx, window);
                        } else {
                            tracing::info!(
                                target: "script_kit::prompt_chrome",
                                event = "render_script_list_footer_actions_ignored_no_actions",
                                selected_index = this.selected_index,
                                "Ignored ScriptList footer actions click because the current selection has no actions"
                            );
                        }
                    }),
                    cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
                        this.open_tab_ai_acp_with_entry_intent(None, cx);
                    }),
                );
            if let Some(footer) = self.main_window_footer_slot(gpui_footer) {
                main_div = main_div.child(footer);
            }
        }

        if let Some(panel) = log_panel {
            main_div = main_div.child(panel);
        }

        // Note: Toast notifications are now handled by gpui-component's NotificationList
        // via the Root wrapper. Toasts are flushed in render() via flush_pending_toasts().

        // Note: HUD overlay is added at the top-level render() method for all views

        // Log total render_script_list time and update tracking state (only if state changed)
        if state_changed {
            let total_elapsed = render_list_start.elapsed();
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_SCRIPT_LIST_END] filter='{}' total={:.2}ms",
                    filter_for_log,
                    total_elapsed.as_secs_f64() * 1000.0
                ),
            );
            // Deferred state update: update after all logging (including preview panel) is done
            self.main_menu_render_diagnostics.last_render_log_filter = self.filter_text.clone();
            self.main_menu_render_diagnostics.last_render_log_selection = self.selected_index;
            self.main_menu_render_diagnostics.last_render_log_item_count = item_count_for_log;
        }

        main_div.into_any_element()
    }
}

#[cfg(test)]
mod render_script_list_footer_tests {
    use super::{
        app_shell_footer_colors, inline_calc_list_item_hint_text_color,
        inline_calc_list_item_result_text_color, inline_calc_list_item_selected_overlay_rgba,
        inline_calc_list_item_title, script_list_footer_info_label,
    };
    use crate::designs::DesignVariant;
    use crate::theme::ColorResolver;

    #[test]
    fn test_app_shell_footer_colors_use_theme_accent_tokens() {
        let theme = crate::theme::Theme::default();
        let colors = app_shell_footer_colors(&theme);

        assert_eq!(colors.accent, theme.colors.accent.selected);
        assert_eq!(colors.background, theme.colors.accent.selected_subtle);
        assert_eq!(colors.border, theme.colors.ui.border);
        assert_eq!(colors.text_muted, theme.colors.text.muted);
    }

    #[test]
    fn test_universal_prompt_hints_support_custom_primary_label() {
        let hints = crate::components::universal_prompt_hints_with_primary_label("Open App");
        assert_eq!(hints[0].as_ref(), "↵ Open App");
    }

    #[test]
    fn test_script_list_footer_info_label_hidden_when_window_tweaker_disabled() {
        assert_eq!(
            script_list_footer_info_label(false, false, 75, "acrylic", "light"),
            None
        );
    }

    #[test]
    fn test_script_list_footer_info_label_hidden_in_dark_mode() {
        assert_eq!(
            script_list_footer_info_label(true, true, 75, "acrylic", "dark"),
            None
        );
    }

    #[test]
    fn test_script_list_footer_info_label_formats_window_tweaker_metadata() {
        assert_eq!(
            script_list_footer_info_label(true, false, 75, "acrylic", "light"),
            Some("75% | acrylic | light | ⌘-/+ ⌘M ⌘⇧A".to_string())
        );
    }

    #[test]
    fn test_truncate_str_chars_returns_valid_utf8_boundary_when_filter_text_is_multibyte() {
        let input = "é".repeat(45);
        let truncated = crate::utils::truncate_str_chars(&input, 27);

        assert_eq!(truncated.chars().count(), 27);
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_inline_calc_list_item_title_prefixes_equals_sign() {
        assert_eq!(inline_calc_list_item_title("1500"), "= 1500");
    }

    #[test]
    fn test_inline_calc_result_text_color_does_use_resolver_accent_when_selected_non_default() {
        let mut theme = crate::theme::Theme::default();
        theme.colors.accent.selected = 0x112233;
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        let color = inline_calc_list_item_result_text_color(
            true,
            DesignVariant::NeonCyberpunk,
            &theme,
            color_resolver,
        );

        assert_eq!(color, color_resolver.primary_accent());
        assert_ne!(color, theme.colors.accent.selected);
    }

    #[test]
    fn test_inline_calc_hint_text_color_does_use_color_resolver_muted_token() {
        let theme = crate::theme::Theme::default();
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        assert_eq!(
            inline_calc_list_item_hint_text_color(color_resolver),
            color_resolver.empty_text_color()
        );
    }

    #[test]
    fn test_inline_calc_selected_overlay_does_use_resolver_accent_with_theme_alpha() {
        let mut theme = crate::theme::Theme::default();
        theme.colors.accent.selected_subtle = 0x010203;
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        let expected_alpha =
            ((theme.get_opacity().selected.clamp(0.0, 1.0) * 255.0).round() as u32).max(0x2E);
        let expected = (color_resolver.primary_accent() << 8) | expected_alpha;

        assert_eq!(
            inline_calc_list_item_selected_overlay_rgba(&theme, color_resolver),
            expected
        );
    }
}

#[cfg(test)]
mod render_script_list_click_contract_tests {
    use std::fs;

    #[test]
    fn launcher_list_uses_shared_selected_row_click_helper() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        assert!(
            source.contains("should_submit_selected_row_click"),
            "render_script_list should use the shared selected-row click helper"
        );
        assert!(
            source.contains("let was_selected = this.selected_index == ix;"),
            "render_script_list click handler should capture whether the row was already selected"
        );
        assert!(
            source.contains("this.execute_selected(cx);"),
            "render_script_list click handler should still execute the selected row"
        );
    }
}
