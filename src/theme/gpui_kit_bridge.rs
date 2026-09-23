//! Bridge Script Kit's theme into gpui-component 0.6 (gpui-kit), the theme
//! layer every gpui-ai component reads (`cx.theme()` / `semantic_tokens()`).
//!
//! Script Kit already resolves its theme into the forked gpui-component 0.5
//! `ThemeColor` (see `gpui_integration.rs`). Both crates keep the same
//! shadcn-derived field names, so the bridge copies every field the two
//! `ThemeColor` shapes share through their serde representation and keeps
//! 0.6-only fields at gpui-kit's own light/dark defaults. That keeps one source
//! of truth (the Script Kit theme) without hand-maintaining a second
//! 140-field mapping that would silently drift.

use gpui::App;
use gpui_component::theme::{
    Theme as ForkTheme, ThemeColor as ForkThemeColor, ThemeMode as ForkThemeMode,
};
use gpui_kit_component::{
    Theme as KitTheme, ThemeColor as KitThemeColor, ThemeMode as KitThemeMode,
    ThemeTokens as KitThemeTokens,
};
use serde_json::Value;

/// Result of one bridge pass, returned for logging and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KitThemeBridgeReceipt {
    /// Fields copied from the Script Kit-resolved palette.
    pub copied_fields: usize,
    /// gpui-kit fields with no Script Kit counterpart (kept at kit defaults).
    pub kit_only_fields: usize,
}

/// Overlay every shared `ThemeColor` field from `source` onto `base`.
pub(crate) fn merge_theme_colors(
    source: &ForkThemeColor,
    base: &KitThemeColor,
) -> (KitThemeColor, KitThemeBridgeReceipt) {
    let (Ok(Value::Object(source)), Ok(Value::Object(mut merged))) =
        (serde_json::to_value(source), serde_json::to_value(base))
    else {
        return (
            *base,
            KitThemeBridgeReceipt {
                copied_fields: 0,
                kit_only_fields: 0,
            },
        );
    };
    let mut copied_fields = 0;
    let mut kit_only_fields = 0;
    for (key, slot) in merged.iter_mut() {
        match source.get(key) {
            Some(value) => {
                *slot = value.clone();
                copied_fields += 1;
            }
            None => kit_only_fields += 1,
        }
    }
    let receipt = KitThemeBridgeReceipt {
        copied_fields,
        kit_only_fields,
    };
    match serde_json::from_value::<KitThemeColor>(Value::Object(merged)) {
        Ok(colors) => (colors, receipt),
        Err(error) => {
            tracing::warn!(%error, "gpui_kit_theme_bridge_decode_failed");
            (
                *base,
                KitThemeBridgeReceipt {
                    copied_fields: 0,
                    ..receipt
                },
            )
        }
    }
}

/// Mirror the already-synced fork theme into gpui-kit's global theme.
///
/// Call after the fork theme has been updated (see
/// `PreparedComponentTheme::apply`), so both component layers — and every
/// gpui-ai component — render the same Script Kit palette, fonts and radii.
pub(crate) fn sync_kit_theme_from_fork(cx: &mut App) -> KitThemeBridgeReceipt {
    if !cx.has_global::<KitTheme>() || !cx.has_global::<ForkTheme>() {
        return KitThemeBridgeReceipt {
            copied_fields: 0,
            kit_only_fields: 0,
        };
    }
    let fork = ForkTheme::global(cx).clone();
    let is_dark = matches!(fork.mode, ForkThemeMode::Dark);
    let base = if is_dark {
        *KitThemeColor::dark()
    } else {
        *KitThemeColor::light()
    };
    let (colors, receipt) = merge_theme_colors(&fork.colors, &base);

    let kit = KitTheme::global_mut(cx);
    kit.colors = colors;
    kit.tokens = KitThemeTokens::from(&colors);
    kit.mode = if is_dark {
        KitThemeMode::Dark
    } else {
        KitThemeMode::Light
    };
    kit.font_family = fork.font_family.clone();
    kit.font_size = fork.font_size;
    kit.mono_font_family = fork.mono_font_family.clone();
    kit.mono_font_size = fork.mono_font_size;
    kit.radius = fork.radius;
    kit.radius_lg = fork.radius_lg;
    kit.shadow = fork.shadow;
    kit.transparent = fork.transparent;
    kit.highlight_theme = bridge_highlight_theme(&fork, is_dark);
    // gpui-kit mirrors colors/fonts/radius into the headless gpui-base theme
    // (TextView, scrollbars, resize handles) only through `sync_base`.
    KitTheme::sync_base(cx);
    tracing::debug!(
        copied_fields = receipt.copied_fields,
        kit_only_fields = receipt.kit_only_fields,
        is_dark,
        "gpui_kit_theme_bridged"
    );
    receipt
}

/// Carry Script Kit's markdown/code highlight palette into gpui-kit through
/// the shared serde shape; fall back to gpui-kit's own light/dark default.
fn bridge_highlight_theme(
    fork: &ForkTheme,
    is_dark: bool,
) -> std::sync::Arc<gpui_kit_component::highlighter::HighlightTheme> {
    use gpui_kit_component::highlighter::HighlightTheme as KitHighlightTheme;
    serde_json::to_value(&*fork.highlight_theme)
        .ok()
        .and_then(|value| serde_json::from_value::<KitHighlightTheme>(value).ok())
        .map(std::sync::Arc::new)
        .unwrap_or_else(|| {
            if is_dark {
                KitHighlightTheme::default_dark()
            } else {
                KitHighlightTheme::default_light()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_fields_follow_script_kit_palette() {
        let mut source = *ForkThemeColor::dark();
        source.accent = gpui::hsla(0.33, 0.9, 0.5, 1.0);
        source.background = gpui::hsla(0.6, 0.2, 0.1, 1.0);
        let (merged, receipt) = merge_theme_colors(&source, &KitThemeColor::dark());
        // Colors round-trip through gpui's hex serde form (8-bit channels).
        let close = |a: gpui::Hsla, b: gpui::Hsla| {
            let (a, b) = (a.to_rgb(), b.to_rgb());
            (a.r - b.r).abs() < 1.0 / 255.0
                && (a.g - b.g).abs() < 1.0 / 255.0
                && (a.b - b.b).abs() < 1.0 / 255.0
        };
        assert!(close(merged.accent, source.accent));
        assert!(close(merged.background, source.background));
        assert_ne!(merged.accent, KitThemeColor::dark().accent);
        assert!(receipt.copied_fields > 40, "{receipt:?}");
    }
}

/// Initialize every GPUI component layer Script Kit renders with: the forked
/// gpui-component 0.5 (legacy surfaces), gpui-kit 0.6 and gpui-ai (Agent
/// Chat and migrated surfaces). Call instead of `gpui_component::init`.
pub fn init_component_libraries(cx: &mut App) {
    gpui_component::init(cx);
    // gpui_ai::init also initializes gpui-kit (gpui-component 0.6).
    gpui_ai::init(cx);
    sync_kit_theme_from_fork(cx);
    // gpui-kit re-applies its registry theme (`Theme::change`) whenever the
    // registry global changes (async theme loading, hot reload). This observer
    // is registered after gpui-kit's own, so it runs after that reset and puts
    // the Script Kit palette back before the next frame.
    cx.observe_global::<gpui_kit_component::ThemeRegistry>(|cx| {
        sync_kit_theme_from_fork(cx);
    })
    .detach();
}
