// Shared design-contract resolver for the built-in main-input count label
// (`render_builtin_main_input_count_label` in `src/render_builtins/common.rs`).
//
// Every built-in browser (settings hub, clipboard history, app launcher, …)
// paints its trailing "N items" label through the same helper, so the
// typography, inset, and color are owned HERE — by the shared builtin
// main-input surface — not by any single browser. The renderer and the
// token exporter (`src/design_contract`) consume the SAME resolver so the
// two can never drift (2026-07-11 Oracle review, settings-hub slice).
//
// Physically lives under `src/render_builtins/` (pulled into the binary via
// the `render_builtins/mod.rs` include chain); the lib re-exports the same
// file (`#[path]` module in `src/lib.rs`, the `path_action` pattern) so the
// exporter and `cargo test --lib` reach it without linking the binary.

/// The count label renders with gpui `.text_sm()` — `rems(0.875)` at the
/// default 16px rem → 14px. This constant is the framework bridge; keep it
/// beside the resolver, never inline `0.875rem`/`14px` in the exporter.
pub const BUILTIN_MAIN_INPUT_COUNT_FONT_SIZE_PX: f32 = 14.0;

/// What `render_builtin_main_input_count_label` actually paints.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinMainInputCountLabelStyle {
    pub font_size_px: f32,
    /// gpui's implicit `phi()` line height, rounded like
    /// `line_height_in_pixels` (14 → 23). The label is vertically centered
    /// by the input shell's `items_center`, so this shapes the glyph box
    /// rather than the shell layout — but mockups must model it or the
    /// count baseline drifts.
    pub line_height_px: f32,
    /// The helper sets no weight: gpui's default `FontWeight::NORMAL` (400).
    /// The label must NOT inherit the search input's 430 — that weight
    /// belongs to the input body, not the shell's trailing children.
    pub font_weight: gpui::FontWeight,
    /// Right inset — the SAME `MainMenuSearchTokens.text_inset_x` authority
    /// already exported as `--sk-main-menu-search-text-inset-x`. Reused, not
    /// aliased.
    pub inset_right: f32,
    /// The SAME `AppChromeColors.text_hint_rgba` resolution already exported
    /// as `--sk-text-hint`. Reused, not aliased.
    pub text_rgba: u32,
}

// pub(crate): the signature carries the pub(crate) `AppChromeColors`, and
// both consumers (the bin renderer and the lib exporter) are in-crate.
pub(crate) fn resolved_builtin_main_input_count_label_style(
    def: crate::designs::MainMenuThemeDef,
    chrome: &crate::theme::AppChromeColors,
) -> BuiltinMainInputCountLabelStyle {
    BuiltinMainInputCountLabelStyle {
        font_size_px: BUILTIN_MAIN_INPUT_COUNT_FONT_SIZE_PX,
        // Reuse the shared GPUI default-line-height bridge extracted for the
        // confirm surface instead of duplicating the phi formula.
        line_height_px: crate::confirm::confirm_prompt_line_height_px(
            BUILTIN_MAIN_INPUT_COUNT_FONT_SIZE_PX,
        ),
        font_weight: gpui::FontWeight::NORMAL,
        inset_right: def.search.text_inset_x,
        text_rgba: chrome.text_hint_rgba,
    }
}

#[cfg(test)]
mod builtin_main_input_count_label_contract {
    use super::*;

    fn stock_inputs() -> (
        crate::designs::MainMenuThemeDef,
        crate::theme::AppChromeColors,
    ) {
        let theme: crate::theme::Theme = crate::theme::presets::all_presets()
            .into_iter()
            .find(|preset| preset.id == "script-kit-dark")
            .expect("stock script-kit-dark preset")
            .create_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let def = crate::designs::MainMenuThemeVariant::InfoBarBase.base_def();
        (def, chrome)
    }

    #[test]
    fn count_label_typography_is_text_sm_with_default_line_height_and_weight() {
        let (def, chrome) = stock_inputs();
        let style = resolved_builtin_main_input_count_label_style(def, &chrome);
        assert_eq!(style.font_size_px, 14.0);
        assert_eq!(
            style.line_height_px,
            crate::confirm::confirm_prompt_line_height_px(14.0)
        );
        assert_eq!(style.line_height_px, 23.0);
        // Normal/default weight — never the search body's 430.
        assert_eq!(style.font_weight.0, gpui::FontWeight::NORMAL.0);
        assert_eq!(style.font_weight.0, 400.0);
        assert_ne!(style.font_weight.0, def.search.font_weight.0);
    }

    #[test]
    fn count_label_reuses_canonical_inset_and_hint_color_authorities() {
        let (def, chrome) = stock_inputs();
        let style = resolved_builtin_main_input_count_label_style(def, &chrome);
        assert_eq!(style.inset_right, def.search.text_inset_x);
        assert_eq!(style.inset_right, 16.0);
        assert_eq!(style.text_rgba, chrome.text_hint_rgba);
    }
}
