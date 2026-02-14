fn calculator_copy_hint_label(is_selected: bool) -> &'static str {
    if is_selected {
        "↵ Copy"
    } else {
        "Enter to Copy"
    }
}

fn calculator_selected_overlay_alpha(theme: &crate::theme::Theme) -> u32 {
    let opacity = theme.get_opacity().selected.clamp(0.0, 1.0);
    ((opacity * 255.0).round() as u32).max(0x2E)
}

fn render_calculator_item(
    calculator: &crate::calculator::CalculatorInlineResult,
    is_selected: bool,
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
) -> AnyElement {
    let tokens = get_tokens(design_variant);
    let spacing = tokens.spacing();
    let typography = tokens.typography();

    let expression = if calculator.raw_input.trim().is_empty() {
        calculator.normalized_expr.as_str()
    } else {
        calculator.raw_input.as_str()
    };

    let line_one_color = if is_selected {
        theme.colors.accent.selected
    } else {
        theme.colors.text.secondary
    };
    let line_two_color = if is_selected {
        theme.colors.accent.selected
    } else {
        theme.colors.text.primary
    };

    let hint_alpha = if is_selected { 0xE6 } else { 0x99 };
    let hint_label = calculator_copy_hint_label(is_selected);

    div()
        .w_full()
        .h_full()
        .px(px(spacing.item_padding_x))
        .py(px(spacing.padding_xs))
        .when(is_selected, |div| {
            div.bg(rgba(
                (theme.colors.accent.selected_subtle << 8)
                    | calculator_selected_overlay_alpha(theme),
            ))
        })
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(spacing.gap_md))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(spacing.gap_sm))
                .overflow_x_hidden()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(spacing.gap_sm))
                        .items_baseline()
                        .overflow_x_hidden()
                        .child(
                            div()
                                .font_family(typography.font_family_mono)
                                .text_size(px(typography.font_size_xs))
                                .text_color(rgba((line_one_color << 8) | 0xCC))
                                .child(expression.to_string()),
                        )
                        .child(
                            div()
                                .text_size(px(typography.font_size_xs))
                                .text_color(rgba((theme.colors.text.muted << 8) | 0xA6))
                                .child(calculator.operation_name.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(spacing.gap_sm))
                        .items_baseline()
                        .overflow_x_hidden()
                        .child(
                            div()
                                .text_size(px(typography.font_size_md))
                                .font_weight(typography.font_weight_semibold)
                                .text_color(rgb(line_two_color))
                                .child(calculator.formatted.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(typography.font_size_xs))
                                .text_color(rgba((theme.colors.text.muted << 8) | 0x99))
                                .child(calculator.words.clone()),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(typography.font_size_xs))
                .text_color(rgba((theme.colors.text.muted << 8) | hint_alpha))
                .child(hint_label),
        )
        .into_any_element()
}

#[cfg(test)]
mod calculator_item_tests {
    use super::*;

    #[test]
    fn test_calculator_copy_hint_label_prefers_short_selected_hint() {
        assert_eq!(calculator_copy_hint_label(true), "↵ Copy");
        assert_eq!(calculator_copy_hint_label(false), "Enter to Copy");
    }
}
