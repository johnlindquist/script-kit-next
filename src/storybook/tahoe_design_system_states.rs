//! Tahoe/Liquid Glass design-system preview matrix.
//!
//! Each preview is backed by the shared Tahoe chrome tokens and the runtime
//! native material resolver used by live windows.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};

use crate::storybook::StoryVariant;
use crate::theme::{get_cached_theme, AppChromeColors};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TahoeDesignSystemSurfaceId {
    MainMenu,
    ActionsPopup,
    Footer,
    AcpChat,
    ThemeDesigner,
    FormPrompt,
}

impl TahoeDesignSystemSurfaceId {
    pub const ALL: [Self; 6] = [
        Self::MainMenu,
        Self::ActionsPopup,
        Self::Footer,
        Self::AcpChat,
        Self::ThemeDesigner,
        Self::FormPrompt,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MainMenu => "tahoe-main-menu",
            Self::ActionsPopup => "tahoe-actions-popup",
            Self::Footer => "tahoe-footer",
            Self::AcpChat => "tahoe-acp-chat",
            Self::ThemeDesigner => "tahoe-theme-designer",
            Self::FormPrompt => "tahoe-form-prompt",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MainMenu => "Main Menu",
            Self::ActionsPopup => "Actions Popup",
            Self::Footer => "Footer",
            Self::AcpChat => "Agent Chat",
            Self::ThemeDesigner => "Theme Designer",
            Self::FormPrompt => "Form Prompt",
        }
    }

    pub fn radius_label(self) -> &'static str {
        match self {
            Self::MainMenu | Self::Footer => "panel",
            Self::ActionsPopup => "popup shell",
            Self::AcpChat => "control large",
            Self::ThemeDesigner | Self::FormPrompt => "prompt surface",
        }
    }

    pub fn radius_value(self) -> f32 {
        let metrics = crate::ui::chrome::TAHOE_CHROME_METRICS;
        match self {
            Self::MainMenu | Self::Footer => metrics.panel_radius,
            Self::ActionsPopup => metrics.popup_shell_radius,
            Self::AcpChat => metrics.control_lg_radius,
            Self::ThemeDesigner | Self::FormPrompt => metrics.prompt_surface_radius,
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "tahoe-main-menu" => Some(Self::MainMenu),
            "tahoe-actions-popup" => Some(Self::ActionsPopup),
            "tahoe-footer" => Some(Self::Footer),
            "tahoe-acp-chat" => Some(Self::AcpChat),
            "tahoe-theme-designer" => Some(Self::ThemeDesigner),
            "tahoe-form-prompt" => Some(Self::FormPrompt),
            _ => None,
        }
    }
}

pub fn tahoe_design_system_story_variants() -> Vec<StoryVariant> {
    TahoeDesignSystemSurfaceId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(format!(
                    "{} Tahoe material/radius preview using shared chrome tokens.",
                    id.name()
                ))
                .with_prop("surface", id.as_str())
                .with_prop("representation", "tahoeTokenPreview")
                .with_prop("materialSource", "native_material_selection")
                .with_prop("radiusSource", "TAHOE_CHROME_METRICS")
                .with_prop("radiusToken", id.radius_label())
        })
        .collect()
}

pub fn render_tahoe_design_system_preview(stable_id: &str) -> AnyElement {
    let id = TahoeDesignSystemSurfaceId::from_stable_id(stable_id)
        .unwrap_or(TahoeDesignSystemSurfaceId::MainMenu);
    render_tahoe_surface(id, false)
}

pub fn render_tahoe_design_system_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = TahoeDesignSystemSurfaceId::from_stable_id(stable_id)
        .unwrap_or(TahoeDesignSystemSurfaceId::MainMenu);
    render_tahoe_surface(id, true)
}

fn render_tahoe_surface(id: TahoeDesignSystemSurfaceId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let accessibility = crate::platform::system_appearance_accessibility();
    let native_material =
        crate::platform::native_material_selection(theme.is_vibrancy_enabled(), accessibility);
    let metrics = crate::ui::chrome::TAHOE_CHROME_METRICS;
    let width = if compact { 360.0 } else { 560.0 };
    let shell_radius = id.radius_value();

    div()
        .w_full()
        .min_h(px(if compact { 260.0 } else { 380.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .rounded(px(shell_radius))
                .overflow_hidden()
                .border_1()
                .border_color(rgba(chrome.border_rgba))
                .bg(rgba(chrome.popup_surface_rgba))
                .shadow(vec![gpui::BoxShadow {
                    color: rgba(0x00000044),
                    offset: gpui::point(px(0.0), px(10.0)),
                    blur_radius: px(metrics.footer_shadow_blur),
                    spread_radius: px(0.0),
                }])
                .flex()
                .flex_col()
                .child(header(id, native_material.label(), &chrome))
                .child(body(id, compact, &chrome))
                .child(footer(id, &chrome)),
        )
        .into_any_element()
}

fn header(
    id: TahoeDesignSystemSurfaceId,
    material_label: &'static str,
    chrome: &AppChromeColors,
) -> AnyElement {
    div()
        .px(px(16.0))
        .py(px(12.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .bg(rgba(chrome.panel_surface_rgba))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(chrome.text_primary_hex))
                .child(id.name()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba(chrome.text_hint_rgba))
                .child(material_label),
        )
        .into_any_element()
}

fn body(id: TahoeDesignSystemSurfaceId, compact: bool, chrome: &AppChromeColors) -> AnyElement {
    let metrics = crate::ui::chrome::TAHOE_CHROME_METRICS;
    let row_radius = match id {
        TahoeDesignSystemSurfaceId::ActionsPopup => metrics.action_row_radius,
        TahoeDesignSystemSurfaceId::AcpChat => metrics.control_md_radius,
        _ => metrics.control_sm_radius,
    };

    div()
        .p(px(if compact { 12.0 } else { 16.0 }))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .children((0..3).map(|index| {
            div()
                .h(px(if id == TahoeDesignSystemSurfaceId::AcpChat {
                    metrics.acp_composer_min_height
                } else {
                    metrics.button_height
                }))
                .rounded(px(row_radius))
                .bg(if index == 0 {
                    rgba(chrome.selection_rgba)
                } else {
                    rgba(chrome.input_surface_rgba)
                })
                .border_1()
                .border_color(rgba(chrome.badge_border_rgba))
                .px(px(10.0))
                .flex()
                .items_center()
                .text_xs()
                .text_color(rgb(chrome.text_primary_hex))
                .child(match (id, index) {
                    (TahoeDesignSystemSurfaceId::MainMenu, 0) => "Selected launcher row",
                    (TahoeDesignSystemSurfaceId::ActionsPopup, 0) => "Run action",
                    (TahoeDesignSystemSurfaceId::Footer, 0) => "Inset native footer rail",
                    (TahoeDesignSystemSurfaceId::AcpChat, 0) => "Composer glass candidate",
                    (TahoeDesignSystemSurfaceId::ThemeDesigner, 0) => "Material controls",
                    (TahoeDesignSystemSurfaceId::FormPrompt, 0) => "Focused input field",
                    _ => "Token-aligned chrome",
                })
        }))
        .into_any_element()
}

fn footer(id: TahoeDesignSystemSurfaceId, chrome: &AppChromeColors) -> AnyElement {
    div()
        .px(px(16.0))
        .py(px(8.0))
        .border_t_1()
        .border_color(rgba(chrome.divider_rgba))
        .flex()
        .justify_between()
        .text_xs()
        .text_color(rgba(chrome.text_hint_rgba))
        .child(format!(
            "{} radius {:.0}px",
            id.radius_label(),
            id.radius_value()
        ))
        .child("TAHOE_CHROME_METRICS")
        .into_any_element()
}
