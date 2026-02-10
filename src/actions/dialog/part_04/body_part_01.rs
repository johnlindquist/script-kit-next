// Get design tokens for the current design variant
let tokens = get_tokens(self.design_variant);
let colors = tokens.colors();
let spacing = tokens.spacing();
let visual = tokens.visual();

// NOTE: Key handling is done by the parent (ScriptListApp in main.rs)
// which routes all keyboard events to this dialog's methods.
// We do NOT attach our own on_key_down handler to avoid double-processing.

// Render search input - compact version
let search_display = if self.search_text.is_empty() {
    SharedString::from("Search actions...")
} else {
    SharedString::from(self.search_text.clone())
};

// Use helper method for design/theme color extraction
let (_search_box_bg, border_color, _muted_text, dimmed_text, _secondary_text) =
    self.get_search_colors(&colors);

// Get primary text color for cursor (matches main list styling)
let primary_text = if self.design_variant == DesignVariant::Default {
    rgb(self.theme.colors.text.primary)
} else {
    rgb(colors.text_primary)
};

// Get accent color for the search input focus indicator
let accent_color_hex = if self.design_variant == DesignVariant::Default {
    self.theme.colors.accent.selected
} else {
    colors.accent
};
let accent_color = rgb(accent_color_hex);

// Focus border color (accent with theme-aware transparency)
// Use border_active opacity for focused state, scaled for visibility
let opacity = self.theme.get_opacity();
let focus_border_alpha = ((opacity.border_active * 1.5).min(1.0) * 255.0) as u8;
let _focus_border_color = rgba(hex_with_alpha(accent_color_hex, focus_border_alpha));

// Raycast-style footer search input: minimal styling, full-width, top separator line
// No boxed input field - just text on a clean background with a thin top border
// Use theme colors for both light and dark mode
// Light mode derives from the same theme tokens as dark mode
let separator_color = border_color;
let hint_text_color = dimmed_text;
let input_text_color = primary_text;
let action_row_vertical_padding = 2.0;

let input_container = div()
    .w(px(POPUP_WIDTH)) // Match parent width exactly
    .min_w(px(POPUP_WIDTH))
    .max_w(px(POPUP_WIDTH))
    .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
    .min_h(px(SEARCH_INPUT_HEIGHT))
    .max_h(px(SEARCH_INPUT_HEIGHT))
    .overflow_hidden() // Prevent any content from causing shifts
    .px(px(spacing.item_padding_x))
    .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
    // No background - clean/transparent to match Raycast
    .border_t_1() // Top separator line only
    .border_color(separator_color)
    .flex()
    .flex_row()
    .items_center()
    .child(
        // Full-width search input - no box styling, just text
        div()
            .flex_1() // Take full width
            .h(px(28.0))
            .flex()
            .flex_row()
            .items_center()
            .text_sm()
            // Placeholder or input text color
            .text_color(if self.search_text.is_empty() {
                hint_text_color
            } else {
                input_text_color
            })
            // Cursor at start when empty
            .when(self.search_text.is_empty(), |d| {
                d.child(
                    div()
                        .w(px(2.))
                        .h(px(16.))
                        .mr(px(2.))
                        .rounded(px(1.))
                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                )
            })
            .child(search_display.clone())
            // Cursor at end when has text
            .when(!self.search_text.is_empty(), |d| {
                d.child(
                    div()
                        .w(px(2.))
                        .h(px(16.))
                        .ml(px(2.))
                        .rounded(px(1.))
                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                )
            }),
    );
