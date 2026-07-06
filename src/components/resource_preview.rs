//! Shared read-only `kit://` resource preview surface.
//!
//! The Day Page (main window) and the Notes window both open deeplinked
//! `kit://` resources in a read-only inspector. This component owns the
//! preview chrome — header, action row, scrollable body, and keycap hint
//! footer — so padding, typography, and visual language cannot drift
//! between windows.
//!
//! Horizontal inset contract: callers pass the host editor's own text inset
//! (`NotesEditorLayout.padding_x`) as `inset_x` so preview text aligns with
//! prose rendered in the same window.

use gpui::{div, prelude::*, px, AnyElement, App, FontWeight, SharedString};
use gpui_component::theme::ActiveTheme;

use crate::components::hint_strip::{render_hint_icons_clickable, ClickableHint, HintClickHandler};
use crate::list_item::FONT_MONO;
use crate::ui::chrome::{alpha_from_opacity, HINT_TEXT_OPACITY};

/// Muted opacity for secondary metadata rows (mime line).
/// Matches the Notes window `OPACITY_MUTED` token.
const MUTED_OPACITY: f32 = 0.7;
/// Border opacity for the scrollable body frame.
/// Matches the Notes window `OPACITY_SECTION_BORDER` token.
const BORDER_OPACITY: f32 = 0.2;
/// Corner radius for the scrollable body frame.
const BODY_RADIUS: f32 = 6.0;
/// Vertical padding around the whole preview surface.
const INSET_Y: f32 = 8.0;
/// Internal padding of the scrollable body frame.
const BODY_PADDING: f32 = 12.0;

/// One header action link ("Copy URI", "Close Preview", …).
pub(crate) struct ResourcePreviewAction {
    /// Full element id, e.g. "day-page-kit-resource-preview-copy-uri".
    /// Runtime probes address these ids — keep them stable per surface.
    pub id: SharedString,
    pub label: SharedString,
    /// Muted actions (Close) render in muted foreground; the rest in accent.
    pub muted: bool,
    pub on_click: HintClickHandler,
}

pub(crate) struct ResourcePreviewSurface {
    /// Element id prefix, e.g. "day-page-kit-resource-preview".
    pub id_prefix: &'static str,
    pub title: SharedString,
    pub uri: SharedString,
    pub mime_type: SharedString,
    pub text: SharedString,
    pub truncated: bool,
    /// Horizontal content inset; pass the host editor's text inset so the
    /// preview aligns with prose in the same window.
    pub inset_x: f32,
    pub actions: Vec<ResourcePreviewAction>,
    /// Clickable keycap hints for the footer line, in hint-strip syntax
    /// (e.g. "Esc Close", "⌘C Copy URI", "↵ Open Source"). These teach the
    /// preview keyboard contract and double as buttons.
    pub footer_hints: Vec<ClickableHint>,
}

fn preview_id(prefix: &str, suffix: &str) -> SharedString {
    SharedString::from(format!("{prefix}-{suffix}"))
}

pub(crate) fn render_resource_preview(surface: ResourcePreviewSurface, cx: &App) -> AnyElement {
    let ResourcePreviewSurface {
        id_prefix,
        title,
        uri,
        mime_type,
        text,
        truncated,
        inset_x,
        actions,
        footer_hints,
    } = surface;

    let mut action_row = div().flex().items_center().gap_2();
    for action in actions {
        let color = if action.muted {
            cx.theme().muted_foreground
        } else {
            cx.theme().accent
        };
        let on_click = action.on_click;
        action_row = action_row.child(
            div()
                .id(action.id)
                .text_xs()
                .text_color(color)
                .cursor_pointer()
                .hover(|s| s.text_color(cx.theme().foreground))
                .on_click(move |event, window, cx| on_click(event, window, cx))
                .child(action.label),
        );
    }

    let hint_text_rgba = {
        let theme = crate::theme::get_cached_theme();
        ((theme.colors.text.primary & 0x00FF_FFFF) << 8) | alpha_from_opacity(HINT_TEXT_OPACITY)
    };

    div()
        .id(SharedString::from(id_prefix))
        .flex_1()
        .min_h(px(0.))
        .flex()
        .flex_col()
        .gap_3()
        .px(px(inset_x))
        .py(px(INSET_Y))
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .id(preview_id(id_prefix, "title"))
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(title),
                        )
                        .child(
                            div()
                                .id(preview_id(id_prefix, "uri"))
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(uri),
                        )
                        .child(
                            div()
                                .id(preview_id(id_prefix, "meta"))
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(MUTED_OPACITY))
                                .child(format!(
                                    "{mime_type} · read-only{}",
                                    if truncated { " · truncated" } else { "" }
                                )),
                        ),
                )
                .child(action_row),
        )
        .child(
            div()
                .id(preview_id(id_prefix, "body"))
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .rounded(px(BODY_RADIUS))
                .border_1()
                .border_color(cx.theme().border.opacity(BORDER_OPACITY))
                .p(px(BODY_PADDING))
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(cx.theme().foreground)
                .child(text),
        )
        .child(
            div()
                .id(preview_id(id_prefix, "hints"))
                .flex()
                .items_center()
                .child(render_hint_icons_clickable(footer_hints, hint_text_rgba)),
        )
        .into_any_element()
}
