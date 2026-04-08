//! Confirm popup playground — integrated surface scenes for compare mode.
//!
//! Three stable variants (`current`, `whisper`, `danger`) rendered via
//! `IntegratedSurfaceShell` with a real `PromptFooter` and themed confirm
//! overlay panel. No production confirm code is touched.

use gpui::*;

use crate::components::prompt_footer::{PromptFooter, PromptFooterColors};
use crate::list_item::FONT_MONO;
use crate::storybook::{
    config_from_storybook_footer_selection_value, FooterVariationId, IntegratedOverlayAnchor,
    IntegratedOverlayPlacement, IntegratedSurfaceShell, IntegratedSurfaceShellConfig, StoryVariant,
};
use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

// ---------------------------------------------------------------------------
// Variant IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmPopupPlaygroundId {
    Current,
    Whisper,
    Danger,
}

impl ConfirmPopupPlaygroundId {
    pub const ALL: [Self; 3] = [Self::Current, Self::Whisper, Self::Danger];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Whisper => "whisper",
            Self::Danger => "danger",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Whisper => "Whisper",
            Self::Danger => "Danger",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Current => "Match current production hierarchy and footer-attached placement.",
            Self::Whisper => "Same placement with quieter chrome and less visual weight.",
            Self::Danger => "Destructive confirm with red-tinted emphasis and warning icon.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "whisper" => Some(Self::Whisper),
            "danger" => Some(Self::Danger),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmPopupPlaygroundSpec {
    pub id: ConfirmPopupPlaygroundId,
    pub title: &'static str,
    pub body: &'static str,
    pub confirm_text: &'static str,
    pub cancel_text: &'static str,
    pub is_danger: bool,
    pub footer_variant: FooterVariationId,
    pub border_opacity_tenths: u8,
    pub confirm_fill_opacity_tenths: u8,
}

const SPECS: [ConfirmPopupPlaygroundSpec; 3] = [
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::Current,
        title: "Clear Conversation",
        body: "This will remove all messages. You can't undo this.",
        confirm_text: "Clear",
        cancel_text: "Cancel",
        is_danger: false,
        footer_variant: FooterVariationId::Minimal,
        border_opacity_tenths: 3,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::Whisper,
        title: "Clear Conversation",
        body: "This will remove all messages. You can't undo this.",
        confirm_text: "Clear",
        cancel_text: "Cancel",
        is_danger: false,
        footer_variant: FooterVariationId::Minimal,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 0,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::Danger,
        title: "Delete Script",
        body: "This will permanently delete the script and its data.",
        confirm_text: "Delete",
        cancel_text: "Cancel",
        is_danger: true,
        footer_variant: FooterVariationId::Minimal,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 1,
    },
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn confirm_popup_playground_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "confirm-popup-playground")
                .with_prop("variantId", spec.id.as_str())
        })
        .collect()
}

pub fn render_confirm_popup_playground_story_preview(stable_id: &str) -> AnyElement {
    let spec = SPECS
        .iter()
        .find(|s| s.id.as_str() == stable_id)
        .copied()
        .unwrap_or(SPECS[0]);

    tracing::info!(
        event = "confirm_popup_playground_rendered",
        variant_id = spec.id.as_str(),
        danger = spec.is_danger,
        "Rendered confirm popup playground preview"
    );

    IntegratedSurfaceShell::new(
        IntegratedSurfaceShellConfig {
            width: 560.0,
            height: 320.0,
            ..Default::default()
        },
        render_launcher_body(),
    )
    .footer(render_footer(spec.footer_variant))
    .overlay(
        IntegratedOverlayPlacement::new(IntegratedOverlayAnchor::Footer, 120.0, 150.0, 320.0),
        render_confirm_panel(spec),
    )
    .into_any_element()
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn render_launcher_body() -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child("Launcher scene"),
        )
        .child(launcher_row(
            &theme,
            "Clipboard History",
            theme.colors.background.title_bar.to_rgb(),
            theme.colors.text.secondary.to_rgb(),
        ))
        .child(launcher_row(
            &theme,
            "Theme Chooser",
            theme.colors.background.title_bar.with_opacity(0.75),
            theme.colors.text.primary.to_rgb(),
        ))
        .child(launcher_row(
            &theme,
            "Window Switcher",
            theme.colors.background.title_bar.to_rgb(),
            theme.colors.text.secondary.to_rgb(),
        ))
        .into_any_element()
}

fn launcher_row(_theme: &crate::theme::Theme, label: &str, bg: Hsla, fg: Hsla) -> gpui::Div {
    div()
        .rounded(px(8.0))
        .bg(bg)
        .px(px(12.0))
        .py(px(10.0))
        .text_sm()
        .text_color(fg)
        .child(label.to_string())
}

fn render_footer(footer_variant: FooterVariationId) -> AnyElement {
    let theme = get_cached_theme();
    let colors = PromptFooterColors::from_theme(&theme);
    let config = config_from_storybook_footer_selection_value(Some(footer_variant.as_str()));

    PromptFooter::new(config, colors).into_any_element()
}

fn render_confirm_panel(spec: ConfirmPopupPlaygroundSpec) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    let accent = if spec.is_danger {
        theme.colors.ui.error
    } else {
        theme.colors.accent.selected
    };

    let border_opacity = spec.border_opacity_tenths as f32 / 10.0;
    let confirm_fill_opacity = spec.confirm_fill_opacity_tenths as f32 / 10.0;

    // Title row — optional warning icon for danger variant
    let mut title_row = div().flex().flex_row().items_center().gap(px(6.0));

    if spec.is_danger {
        title_row = title_row.child(
            div()
                .text_xs()
                .text_color(theme.colors.ui.error.to_rgb())
                .child("⚠"),
        );
    }

    title_row = title_row.child(
        div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.colors.text.primary.to_rgb())
            .child(spec.title),
    );

    div()
        .w_full()
        .rounded(px(10.0))
        .overflow_hidden()
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(border_opacity))
        .bg(gpui::rgba(chrome.popup_surface_rgba))
        // Top accent stripe
        .child(
            div()
                .h(px(1.0))
                .w_full()
                .bg(accent.with_opacity(if spec.is_danger { 0.18 } else { 0.10 })),
        )
        // Content
        .child(
            div()
                .px(px(12.0))
                .py(px(12.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(title_row)
                .child(
                    div()
                        .text_xs()
                        .line_height(px(18.0))
                        .text_color(theme.colors.text.secondary.to_rgb())
                        .child(spec.body),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.0))
                        .child(render_keycap_action(
                            "Esc",
                            spec.cancel_text,
                            false,
                            theme.colors.ui.border.with_opacity(0.06),
                            theme.colors.text.secondary.to_rgb(),
                            theme.colors.text.secondary.to_rgb(),
                        ))
                        .child(render_keycap_action(
                            "↵",
                            spec.confirm_text,
                            true,
                            accent.with_opacity(confirm_fill_opacity.max(0.04)),
                            accent.to_rgb(),
                            if spec.is_danger {
                                accent.to_rgb()
                            } else {
                                theme.colors.text.primary.to_rgb()
                            },
                        )),
                ),
        )
        .into_any_element()
}

fn render_keycap_action(
    keycap: &str,
    label: &str,
    active: bool,
    keycap_bg: Hsla,
    keycap_fg: Hsla,
    label_fg: Hsla,
) -> AnyElement {
    let mut row = div().flex().flex_row().items_center().gap(px(6.0));

    if active {
        row = row.bg(keycap_bg).rounded(px(6.0)).px(px(6.0)).py(px(4.0));
    }

    row.child(
        div()
            .px(px(5.0))
            .py(px(2.0))
            .rounded(px(4.0))
            .bg(keycap_bg)
            .text_xs()
            .font_family(FONT_MONO)
            .text_color(keycap_fg)
            .child(keycap.to_string()),
    )
    .child(
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(label_fg)
            .child(label.to_string()),
    )
    .into_any_element()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{confirm_popup_playground_story_variants, ConfirmPopupPlaygroundId, SPECS};
    use std::collections::HashSet;

    #[test]
    fn confirm_popup_playground_variant_ids_are_unique() {
        let ids: HashSet<_> = confirm_popup_playground_story_variants()
            .into_iter()
            .map(|v| v.stable_id())
            .collect();
        assert_eq!(ids.len(), SPECS.len());
    }

    #[test]
    fn confirm_popup_playground_stable_ids_round_trip() {
        for id in ConfirmPopupPlaygroundId::ALL {
            assert_eq!(
                ConfirmPopupPlaygroundId::from_stable_id(id.as_str()),
                Some(id)
            );
        }
    }
}
