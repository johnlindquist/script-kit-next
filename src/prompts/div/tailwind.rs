use super::*;

/// Apply Tailwind styles to a div based on a class string
pub(super) fn apply_tailwind_styles(mut element: Div, class_string: &str) -> Div {
    let styles = TailwindStyles::parse(class_string);

    // Layout
    if styles.flex {
        element = element.flex();
    }
    if styles.flex_col {
        element = element.flex_col();
    }
    if styles.flex_row {
        element = element.flex_row();
    }
    if styles.flex_1 {
        element = element.flex_1();
    }
    if styles.items_center {
        element = element.items_center();
    }
    if styles.items_start {
        element = element.items_start();
    }
    if styles.items_end {
        element = element.items_end();
    }
    if styles.justify_center {
        element = element.justify_center();
    }
    if styles.justify_between {
        element = element.justify_between();
    }
    if styles.justify_start {
        element = element.justify_start();
    }
    if styles.justify_end {
        element = element.justify_end();
    }

    // Sizing
    if styles.w_full {
        element = element.w_full();
    }
    if styles.h_full {
        element = element.h_full();
    }
    if styles.min_w_0 {
        element = element.min_w(px(0.));
    }
    if styles.min_h_0 {
        element = element.min_h(px(0.));
    }

    // Spacing - padding
    if let Some(p) = styles.padding {
        element = element.p(px(p));
    }
    if let Some(px_val) = styles.padding_x {
        element = element.px(px(px_val));
    }
    if let Some(py_val) = styles.padding_y {
        element = element.py(px(py_val));
    }
    if let Some(pt) = styles.padding_top {
        element = element.pt(px(pt));
    }
    if let Some(pb) = styles.padding_bottom {
        element = element.pb(px(pb));
    }
    if let Some(pl) = styles.padding_left {
        element = element.pl(px(pl));
    }
    if let Some(pr) = styles.padding_right {
        element = element.pr(px(pr));
    }

    // Spacing - margin
    if let Some(m) = styles.margin {
        element = element.m(px(m));
    }
    if let Some(mx_val) = styles.margin_x {
        element = element.mx(px(mx_val));
    }
    if let Some(my_val) = styles.margin_y {
        element = element.my(px(my_val));
    }
    if let Some(mt) = styles.margin_top {
        element = element.mt(px(mt));
    }
    if let Some(mb) = styles.margin_bottom {
        element = element.mb(px(mb));
    }
    if let Some(ml) = styles.margin_left {
        element = element.ml(px(ml));
    }
    if let Some(mr) = styles.margin_right {
        element = element.mr(px(mr));
    }

    // Gap
    if let Some(gap_val) = styles.gap {
        element = element.gap(px(gap_val));
    }

    // Colors
    if let Some(color) = styles.bg_color {
        element = element.bg(rgb(color));
    }
    if let Some(color) = styles.text_color {
        element = element.text_color(rgb(color));
    }
    if let Some(color) = styles.border_color {
        element = element.border_color(rgb(color));
    }

    // Typography
    // User-specified pixel size - not converted to rem
    if let Some(size) = styles.font_size {
        element = element.text_size(px(size));
    }
    if styles.font_bold {
        element = element.font_weight(FontWeight::BOLD);
    }
    if styles.font_medium {
        element = element.font_weight(FontWeight::MEDIUM);
    }
    if styles.font_normal {
        element = element.font_weight(FontWeight::NORMAL);
    }

    // Border radius
    if let Some(r) = styles.rounded {
        element = element.rounded(px(r));
    }

    // Border
    if styles.border {
        element = element.border_1();
    }
    if let Some(width) = styles.border_width {
        if width == 0.0 {
            // No border
        } else if width == 2.0 {
            element = element.border_2();
        } else if width == 4.0 {
            element = element.border_4();
        } else if width == 8.0 {
            element = element.border_8();
        }
    }

    element
}
