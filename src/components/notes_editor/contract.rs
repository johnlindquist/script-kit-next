//! Resolved editor metrics for the shared markdown notes editor.
//!
//! The editable Notes surface is a vendored gpui-component `Input` in
//! code-editor mode. Its painted typography is NOT app-authored: the font
//! comes from the synchronized gpui-component theme
//! (`sync_gpui_component_theme` sets `theme.mono_font_size =
//! px(sk_theme.get_fonts().mono_size)`), the line box and caret come from
//! `Input` internals. This module is the component-side owner of that
//! composition so the design-contract exporter and any Input-facing code
//! resolve ONE set of numbers instead of re-deriving vendor arithmetic.
//!
//! Vendored sources mirrored here (the constants are function-local /
//! `pub(super)` upstream, so they cannot be imported without editing the
//! vendor; this module is the app-side single mirror and its tests are the
//! tripwire):
//! - line box: `Input::render` sets `line_height(Rems(1.25))`
//!   (vendor/gpui-component/crates/ui/src/input/input.rs:243,361), resolved
//!   against GPUI's default 16px window rem size.
//! - caret width: `blink_cursor::CURSOR_WIDTH = px(2.)`
//!   (vendor/gpui-component/crates/ui/src/input/blink_cursor.rs:32).
//! - caret height: `Size::Medium` factor `0.85 × line_height`, vertically
//!   centered (vendor/gpui-component/crates/ui/src/input/element.rs:283-294).
//!
//! Inner Input padding is NOT mirrored: `gpui_component::Size` exposes the
//! real `input_px()`/`input_py()` accessors, so this module reads the actual
//! vendored `Size::Medium` decision instead of copying `12`/`8`.
//!
//! Colors resolve through `map_scriptkit_to_gpui_theme` — the SAME Script
//! Kit theme → gpui-component theme bridge `sync_gpui_component_theme`
//! installs — so text, caret, and markdown-link colors are the renderer's
//! resolution, never `FontConfig::default()` or hand-entered bytes.
//!
//! There is NO capture-specific font size: `gpui::HighlightStyle` is
//! "uniformly sized" (vendor/gpui/src/style.rs) — markdown headings paint at
//! the same nominal size, heavier weight, clipped by the fixed line box.

use gpui::Hsla;

use super::component::{
    markdown_link_destination_rest_color, MARKDOWN_LINK_DESTINATION_COMPACT_OPACITY,
};

/// `Input`'s multi-line line height in rems (vendored, see module docs).
const INPUT_LINE_HEIGHT_REMS: f32 = 1.25;
/// GPUI default window rem size in px; the Notes window never overrides it.
const WINDOW_REM_SIZE_PX: f32 = 16.0;
/// Vendored `blink_cursor::CURSOR_WIDTH`.
const INPUT_CARET_WIDTH_PX: f32 = 2.0;
/// Vendored caret height factor for the default `Size::Medium` input.
const INPUT_CARET_MEDIUM_HEIGHT_FACTOR: f32 = 0.85;

/// What the editable Notes `Input` actually paints for typography, caret
/// geometry, and colors, resolved from the same Script Kit theme →
/// gpui-component theme conversion the renderer uses (NOT
/// `FontConfig::default()`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedNotesEditorMetrics {
    /// Editor font family (`theme.get_fonts().mono_family`, applied via
    /// `Input::font_family(cx.theme().mono_font_family)`).
    pub base_font_family: String,
    /// Editor font size (`theme.get_fonts().mono_size`, applied via
    /// `Input::text_size(cx.theme().mono_font_size)`).
    pub base_font_size: f32,
    /// Painted line box: `Rems(1.25) × 16px rem` = 20px. Every styled run,
    /// including bold markdown titles, is clipped into this box.
    pub line_box_height: f32,
    /// Inner horizontal Input padding — the REAL vendored
    /// `Size::Medium.input_px()` accessor, not a copied literal.
    pub input_padding_x: f32,
    /// Inner vertical Input padding — `Size::Medium.input_py()`.
    pub input_padding_y: f32,
    pub caret_width: f32,
    /// `0.85 × line_box_height` = 17px, vertically centered in the line.
    pub caret_height: f32,
    /// Base text run color: the window text style the host root installs
    /// (`.text_color(theme.colors.text.primary)` in both Notes and Day Page
    /// hosts; the Input shapes runs from `window.text_style()`).
    pub text_color: Hsla,
    /// Caret color from the theme bridge (`theme_color.caret` —
    /// `colors.text.primary` when no focused-cursor override is configured).
    pub caret_color: Hsla,
    /// Markdown link LABEL color: the bridge accent
    /// (`theme_color.accent` ← `colors.accent.selected`). Distinct authority
    /// from the markdown-title highlight color, even though both are amber
    /// in the stock theme.
    pub link_label_color: Hsla,
    /// Markdown link DESTINATION color at rest (no selection overlap):
    /// `markdown_link_destination_rest_color(accent)` — the same resolver
    /// the highlighter paints with.
    pub link_destination_rest_color: Hsla,
    /// Authored compact opacity behind the rest color (JSON-only source
    /// leaf; the exported color above is the resolved product).
    pub link_destination_compact_opacity: f32,
}

/// Resolve the editor metrics for a Script Kit theme via the same
/// `get_fonts()` bridge and `map_scriptkit_to_gpui_theme` conversion
/// `sync_gpui_component_theme` feeds into gpui-component. Pure: safe for the
/// checked-in design-contract exporter.
pub(crate) fn resolved_notes_editor_metrics(
    sk_theme: &crate::theme::Theme,
) -> ResolvedNotesEditorMetrics {
    let fonts = sk_theme.get_fonts();
    let line_box_height = INPUT_LINE_HEIGHT_REMS * WINDOW_REM_SIZE_PX;
    let bridge = crate::theme::gpui_integration::map_scriptkit_to_gpui_theme(
        sk_theme,
        sk_theme.is_dark_mode(),
    );
    let medium = gpui_component::Size::Medium;
    ResolvedNotesEditorMetrics {
        base_font_family: fonts.mono_family,
        base_font_size: fonts.mono_size,
        line_box_height,
        input_padding_x: f32::from(medium.input_px()),
        input_padding_y: f32::from(medium.input_py()),
        caret_width: INPUT_CARET_WIDTH_PX,
        caret_height: INPUT_CARET_MEDIUM_HEIGHT_FACTOR * line_box_height,
        text_color: crate::theme::gpui_integration::hex_to_hsla(sk_theme.colors.text.primary),
        caret_color: bridge.caret,
        link_label_color: bridge.accent,
        link_destination_rest_color: markdown_link_destination_rest_color(bridge.accent),
        link_destination_compact_opacity: MARKDOWN_LINK_DESTINATION_COMPACT_OPACITY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stock_dark_theme() -> crate::theme::Theme {
        crate::theme::presets::all_presets()
            .into_iter()
            .find(|preset| preset.id == "script-kit-dark")
            .expect("script-kit-dark preset")
            .create_theme()
    }

    #[test]
    fn stock_theme_resolves_input_paint_metrics() {
        let metrics = resolved_notes_editor_metrics(&stock_dark_theme());
        assert_eq!(metrics.base_font_family, "JetBrains Mono");
        assert_eq!(metrics.base_font_size, 16.0);
        assert_eq!(metrics.line_box_height, 20.0);
        assert_eq!(metrics.caret_width, 2.0);
        assert_eq!(metrics.caret_height, 17.0);
        // Real vendored Size::Medium accessors, not mirrored literals.
        assert_eq!(metrics.input_padding_x, 12.0);
        assert_eq!(metrics.input_padding_y, 8.0);
    }

    #[test]
    fn line_box_matches_autosize_assumption_today() {
        // Two authorities happen to agree (Input line box vs
        // NotesLayoutMetrics.auto_resize_line_height); this test is the
        // tripwire if the vendored Input line height ever changes.
        let metrics = resolved_notes_editor_metrics(&stock_dark_theme());
        assert_eq!(metrics.line_box_height, 20.0);
    }

    #[test]
    fn stock_theme_resolves_editor_and_link_colors_through_the_bridge() {
        let theme = stock_dark_theme();
        let metrics = resolved_notes_editor_metrics(&theme);

        // Text and caret both resolve to text.primary in the stock dark
        // theme (no focused-cursor override configured).
        assert_eq!(
            metrics.text_color,
            crate::theme::gpui_integration::hex_to_hsla(theme.colors.text.primary)
        );
        assert_eq!(metrics.caret_color, metrics.text_color);

        // Link label = bridge accent; rest destination = the highlighter's
        // own rest resolver over that accent (alpha 0.45), never a
        // re-derived pair.
        assert_eq!(
            metrics.link_label_color,
            crate::theme::gpui_integration::hex_to_hsla(theme.colors.accent.selected)
        );
        assert_eq!(
            metrics.link_destination_rest_color,
            markdown_link_destination_rest_color(metrics.link_label_color)
        );
        assert_eq!(
            metrics.link_destination_rest_color.a,
            MARKDOWN_LINK_DESTINATION_COMPACT_OPACITY
        );
        assert_eq!(metrics.link_destination_compact_opacity, 0.45);
    }
}
