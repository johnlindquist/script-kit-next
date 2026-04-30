//! Confirm popup playground — integrated surface scenes for compare mode.
//!
//! Nine stable destructive-warning variants rendered via
//! `IntegratedSurfaceShell` with a real `PromptFooter` and themed confirm
//! overlay panel. No production confirm code is touched.

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::components::prompt_footer::{PromptFooter, PromptFooterColors};
use crate::list_item::FONT_MONO;
use crate::storybook::{
    config_from_storybook_footer_selection_value,
    playground_overlay_metrics::confirm_playground_overlay_metrics, FooterVariationId,
    IntegratedSurfaceShell, IntegratedSurfaceShellConfig, StoryVariant,
};
use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

// ---------------------------------------------------------------------------
// Variant IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmPopupPlaygroundId {
    Current,
    QuietWarning,
    RedStripe,
    AlertBadge,
    SolidAction,
    SplitActions,
    Compact,
    RichBorder,
    HighContrast,
    /// Live shipping route: AppView::ConfirmPrompt — title + body fill the
    /// main content area, footer reuses the native AppKit Apply/Close slots.
    InWindow,
}

impl ConfirmPopupPlaygroundId {
    pub const ALL: [Self; 10] = [
        Self::InWindow,
        Self::Current,
        Self::QuietWarning,
        Self::RedStripe,
        Self::AlertBadge,
        Self::SolidAction,
        Self::SplitActions,
        Self::Compact,
        Self::RichBorder,
        Self::HighContrast,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::QuietWarning => "quiet-warning",
            Self::RedStripe => "red-stripe",
            Self::AlertBadge => "alert-badge",
            Self::SolidAction => "solid-action",
            Self::SplitActions => "split-actions",
            Self::Compact => "compact",
            Self::RichBorder => "rich-border",
            Self::HighContrast => "high-contrast",
            Self::InWindow => "in-window",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Current => "Current Bottom Sheet",
            Self::QuietWarning => "Quiet Warning",
            Self::RedStripe => "Red Stripe",
            Self::AlertBadge => "Alert Badge",
            Self::SolidAction => "Solid Action",
            Self::SplitActions => "Split Actions",
            Self::Compact => "Compact",
            Self::RichBorder => "Rich Border",
            Self::HighContrast => "High Contrast",
            Self::InWindow => "In-Window State",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Current => "Current hierarchy with the warning copy from the destructive prompt.",
            Self::QuietWarning => "Low-chrome warning with only text and a faint keycap treatment.",
            Self::RedStripe => "Danger is carried by a top stripe and restrained action color.",
            Self::AlertBadge => "Warning icon and label sit in a small badge above the body.",
            Self::SolidAction => "Primary destructive action reads as a filled button.",
            Self::SplitActions => "Cancel and destructive actions get separated visual groups.",
            Self::Compact => {
                "Short, tight confirmation sheet for high-frequency destructive actions."
            }
            Self::RichBorder => {
                "Border and surface tint frame the warning without a filled button."
            }
            Self::HighContrast => {
                "Maximum contrast option for the clearest destructive affordance."
            }
            Self::InWindow => {
                "Live shipping route: confirm fills the main window with title + body and the native footer reuses Apply/Close slots labeled per ParentConfirmOptions."
            }
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "quiet-warning" => Some(Self::QuietWarning),
            "red-stripe" => Some(Self::RedStripe),
            "alert-badge" => Some(Self::AlertBadge),
            "solid-action" => Some(Self::SolidAction),
            "split-actions" => Some(Self::SplitActions),
            "compact" => Some(Self::Compact),
            "rich-border" => Some(Self::RichBorder),
            "high-contrast" => Some(Self::HighContrast),
            "in-window" => Some(Self::InWindow),
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
    pub footer_variant: FooterVariationId,
    pub style: ConfirmPopupVisualStyle,
    pub border_opacity_tenths: u8,
    pub confirm_fill_opacity_tenths: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmPopupVisualStyle {
    Current,
    Quiet,
    Stripe,
    Badge,
    FilledAction,
    SplitActions,
    Compact,
    RichBorder,
    HighContrast,
    /// Full-window state matching the live `AppView::ConfirmPrompt` route:
    /// title + body centered in the main content area, no overlay panel.
    InWindow,
}

const TITLE: &str = "Empty Trash";
const BODY: &str = "Empty Trash now? This cannot be undone.";
const CONFIRM_TEXT: &str = "Empty Trash";
const CANCEL_TEXT: &str = "Cancel";

const SPECS: [ConfirmPopupPlaygroundSpec; 10] = [
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::InWindow,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::InWindow,
        border_opacity_tenths: 0,
        confirm_fill_opacity_tenths: 0,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::Current,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::Current,
        border_opacity_tenths: 3,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::QuietWarning,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::Quiet,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 0,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::RedStripe,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::Stripe,
        border_opacity_tenths: 3,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::AlertBadge,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::Badge,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::SolidAction,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::FilledAction,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 9,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::SplitActions,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::SplitActions,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 2,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::Compact,
        title: TITLE,
        body: "This cannot be undone.",
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::Compact,
        border_opacity_tenths: 2,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::RichBorder,
        title: TITLE,
        body: BODY,
        confirm_text: CONFIRM_TEXT,
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::RichBorder,
        border_opacity_tenths: 5,
        confirm_fill_opacity_tenths: 1,
    },
    ConfirmPopupPlaygroundSpec {
        id: ConfirmPopupPlaygroundId::HighContrast,
        title: "Delete Script",
        body: "Delete this script permanently? This cannot be undone.",
        confirm_text: "Delete",
        cancel_text: CANCEL_TEXT,
        footer_variant: FooterVariationId::Minimal,
        style: ConfirmPopupVisualStyle::HighContrast,
        border_opacity_tenths: 6,
        confirm_fill_opacity_tenths: 9,
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
    render_confirm_popup_playground(stable_id, false)
}

pub fn render_confirm_popup_playground_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_confirm_popup_playground(stable_id, true)
}

fn render_confirm_popup_playground(stable_id: &str, compact: bool) -> AnyElement {
    let spec = SPECS
        .iter()
        .find(|s| s.id.as_str() == stable_id)
        .copied()
        .unwrap_or(SPECS[0]);

    let shell = IntegratedSurfaceShellConfig {
        width: if compact { 430.0 } else { 560.0 },
        height: if compact { 260.0 } else { 320.0 },
        ..Default::default()
    };

    tracing::info!(
        event = "confirm_popup_playground_state_built",
        variant_id = spec.id.as_str(),
        "Built confirm popup playground state"
    );

    // InWindow mirrors the live AppView::ConfirmPrompt route: the confirm
    // content fills the main launcher body; no overlay panel is drawn. The
    // native footer slot reuses the same Apply/Close keycap labels the
    // shipping route emits via FooterButtonConfig.
    if matches!(spec.style, ConfirmPopupVisualStyle::InWindow) {
        return IntegratedSurfaceShell::new(shell, render_in_window_body(spec, compact))
            .footer(render_in_window_footer(spec))
            .into_any_element();
    }

    let metrics = confirm_playground_overlay_metrics(shell);

    tracing::info!(
        event = "confirm_popup_playground_overlay_wired",
        variant_id = spec.id.as_str(),
        overlay_left = metrics.placement.left,
        overlay_top = metrics.placement.top,
        overlay_width = metrics.placement.width,
        "Wired confirm playground overlay through shared metrics"
    );

    IntegratedSurfaceShell::new(shell, render_launcher_body(compact))
        .footer(render_footer(spec.footer_variant))
        .overlay(metrics.placement, render_confirm_panel(spec, compact))
        .into_any_element()
}

fn render_in_window_body(spec: ConfirmPopupPlaygroundSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let title_color = theme.colors.ui.error.to_rgb();
    let body_color = theme.colors.text.secondary.to_rgb();
    let title_size = if compact { 16.0 } else { 20.0 };
    let body_size = if compact { 12.0 } else { 14.0 };

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(if compact { 6.0 } else { 12.0 }))
        .child(
            div()
                .text_size(px(title_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(spec.title),
        )
        .child(
            div()
                .max_w(px(if compact { 360.0 } else { 480.0 }))
                .text_size(px(body_size))
                .text_color(body_color)
                .child(spec.body),
        )
        .into_any_element()
}

fn render_in_window_footer(spec: ConfirmPopupPlaygroundSpec) -> AnyElement {
    let theme = get_cached_theme();
    let accent = theme.colors.accent.selected;
    let muted_keycap_bg = theme.colors.ui.border.with_opacity(0.06);
    let muted_label_color = theme.colors.text.secondary.to_rgb();
    let danger_label_color = theme.colors.ui.error.to_rgb();

    div()
        .w_full()
        .px(px(16.0))
        .py(px(8.0))
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .gap(px(12.0))
        .child(render_keycap_action(
            "Esc",
            spec.cancel_text,
            false,
            muted_keycap_bg,
            muted_label_color,
            muted_label_color,
        ))
        .child(render_keycap_action(
            "↵",
            spec.confirm_text,
            true,
            accent.with_opacity(0.06),
            accent.to_rgb(),
            danger_label_color,
        ))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn render_launcher_body(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(if compact { 6.0 } else { 8.0 }))
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

fn render_confirm_panel(spec: ConfirmPopupPlaygroundSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    let accent = theme.colors.ui.error;
    let border_opacity = spec.border_opacity_tenths as f32 / 10.0;
    let confirm_fill_opacity = spec.confirm_fill_opacity_tenths as f32 / 10.0;
    let show_badge = matches!(
        spec.style,
        ConfirmPopupVisualStyle::Badge | ConfirmPopupVisualStyle::HighContrast
    );
    let show_title_icon = !matches!(
        spec.style,
        ConfirmPopupVisualStyle::Quiet | ConfirmPopupVisualStyle::Badge
    );
    let filled_action = matches!(
        spec.style,
        ConfirmPopupVisualStyle::FilledAction | ConfirmPopupVisualStyle::HighContrast
    );
    let split_actions = matches!(spec.style, ConfirmPopupVisualStyle::SplitActions);
    let top_stripe_opacity = match spec.style {
        ConfirmPopupVisualStyle::Quiet => 0.0,
        ConfirmPopupVisualStyle::Current => 0.10,
        ConfirmPopupVisualStyle::Stripe => 0.38,
        ConfirmPopupVisualStyle::RichBorder => 0.26,
        ConfirmPopupVisualStyle::HighContrast => 0.52,
        _ => 0.18,
    };

    // Title row — optional warning icon for danger variant
    let mut title_row = div().flex().flex_row().items_center().gap(px(6.0));

    if show_title_icon {
        title_row = title_row.child(
            div()
                .text_xs()
                .text_color(theme.colors.ui.error.to_rgb())
                .child("!"),
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
        .rounded(px(match spec.style {
            ConfirmPopupVisualStyle::Compact => 6.0,
            ConfirmPopupVisualStyle::HighContrast => 8.0,
            _ => 10.0,
        }))
        .overflow_hidden()
        .border_1()
        .border_color(
            if matches!(
                spec.style,
                ConfirmPopupVisualStyle::RichBorder | ConfirmPopupVisualStyle::HighContrast
            ) {
                accent.with_opacity(border_opacity)
            } else {
                theme.colors.ui.border.with_opacity(border_opacity)
            },
        )
        .bg(gpui::rgba(chrome.popup_surface_rgba))
        // Top accent stripe
        .child(
            div()
                .h(px(
                    if matches!(spec.style, ConfirmPopupVisualStyle::Stripe) {
                        3.0
                    } else {
                        1.0
                    },
                ))
                .w_full()
                .bg(accent.with_opacity(top_stripe_opacity)),
        )
        // Content
        .child(
            div()
                .px(px(if compact { 10.0 } else { 12.0 }))
                .py(px(
                    if matches!(spec.style, ConfirmPopupVisualStyle::Compact) {
                        9.0
                    } else {
                        12.0
                    },
                ))
                .flex()
                .flex_col()
                .gap(px(
                    if matches!(spec.style, ConfirmPopupVisualStyle::Compact) {
                        7.0
                    } else {
                        10.0
                    },
                ))
                .when(show_badge, |d| {
                    d.child(
                        div()
                            .self_start()
                            .rounded(px(999.0))
                            .px(px(7.0))
                            .py(px(2.0))
                            .bg(accent.with_opacity(0.10))
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(accent.to_rgb())
                            .child("Warning"),
                    )
                })
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
                        .justify_between()
                        .gap(px(8.0))
                        .when(!split_actions, |d| d.justify_end())
                        .child(render_keycap_action(
                            "Esc",
                            spec.cancel_text,
                            split_actions,
                            theme.colors.ui.border.with_opacity(0.06),
                            theme.colors.text.secondary.to_rgb(),
                            theme.colors.text.secondary.to_rgb(),
                        ))
                        .child(render_keycap_action(
                            "↵",
                            spec.confirm_text,
                            true,
                            if filled_action {
                                accent.with_opacity(confirm_fill_opacity)
                            } else {
                                accent.with_opacity(confirm_fill_opacity.max(0.04))
                            },
                            if filled_action {
                                theme.colors.background.main.to_rgb()
                            } else {
                                accent.to_rgb()
                            },
                            if filled_action {
                                theme.colors.background.main.to_rgb()
                            } else {
                                accent.to_rgb()
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
