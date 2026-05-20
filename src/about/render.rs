#![allow(dead_code)]

use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use gpui::{
    div, prelude::*, px, rgb, rgba, svg, App, ClickEvent, Div, FocusHandle, FontWeight,
    IntoElement, KeyDownEvent, Window,
};
use gpui_component::scroll::ScrollableElement;

use crate::{about::AboutState, branding, theme, updates::UpdateState};

pub type AboutClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub type AboutKeyHandler = Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct AboutSurfaceActions {
    pub dismiss: AboutClickHandler,
    pub open_github: AboutClickHandler,
    pub open_discord: AboutClickHandler,
    pub follow_x: AboutClickHandler,
    pub check_updates: AboutClickHandler,
    pub open_release: AboutClickHandler,
    pub toggle_acknowledgements: AboutClickHandler,
    pub key_down: AboutKeyHandler,
}

fn is_about_activation_key(key: &str) -> bool {
    key == " "
        || key.eq_ignore_ascii_case("space")
        || key.eq_ignore_ascii_case("enter")
        || key.eq_ignore_ascii_case("return")
}

/// Render the launcher-native About surface opened from the tray menu.
pub(crate) fn render_about_surface(
    state: &AboutState,
    update_state: Arc<RwLock<UpdateState>>,
    focus: &FocusHandle,
    actions: AboutSurfaceActions,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    render_about_surface_inner(state, update_state, Some(focus), actions)
}

#[cfg(feature = "storybook")]
pub(crate) fn render_about_surface_preview(
    state: &AboutState,
    update_state: Arc<RwLock<UpdateState>>,
    actions: AboutSurfaceActions,
) -> gpui::AnyElement {
    render_about_surface_inner(state, update_state, None, actions).into_any_element()
}

fn render_about_surface_inner(
    state: &AboutState,
    update_state: Arc<RwLock<UpdateState>>,
    focus: Option<&FocusHandle>,
    actions: AboutSurfaceActions,
) -> impl IntoElement {
    let theme = theme::get_cached_theme();
    let chrome = theme::AppChromeColors::from_theme(&theme);
    let snapshot = update_state
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| UpdateState::Error("update state unavailable".into()));
    let key_down = actions.key_down.clone();

    div()
        .id("about-surface")
        .when_some(focus, |surface, focus| surface.track_focus(focus))
        .size_full()
        .flex()
        .flex_col()
        .bg(rgba(chrome.surface_rgba))
        .capture_key_down(move |event, window, cx| key_down(event, window, cx))
        .child(render_header(chrome, actions.dismiss.clone()))
        .child(
            div()
                .id("about-content-scroll")
                .flex_1()
                .min_h(px(0.0))
                .px(px(32.0))
                .py(px(14.0))
                .flex()
                .flex_col()
                .items_center()
                .overflow_y_scrollbar()
                .child(
                    div()
                        .w_full()
                        .max_w(px(560.0))
                        .flex()
                        .flex_col()
                        .items_center()
                        .child(render_logo_block(chrome))
                        .child(render_title_version(chrome))
                        .child(render_tagline(chrome))
                        .child(render_creator_row(chrome))
                        .child(div().h(px(4.0)))
                        .child(render_quick_actions(chrome, &actions))
                        .child(render_update_card(chrome, snapshot, &actions))
                        .child(render_acknowledgements(chrome, state, &actions))
                        .child(render_footer(chrome)),
                ),
        )
}

fn render_header(chrome: theme::AppChromeColors, dismiss: AboutClickHandler) -> Div {
    div()
        .h(px(52.0))
        .w_full()
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(rgba(chrome.border_rgba))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(chrome.text_primary_hex))
                .child("About Script Kit"),
        )
        .child(
            div()
                .id("about-close-button")
                .tab_index(0)
                .size(px(28.0))
                .rounded(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(16.0))
                .text_color(rgba(chrome.text_icon_rgba))
                .cursor_pointer()
                .hover(|style| style.bg(rgba(chrome.hover_rgba)))
                .focus_visible(|style| style.bg(rgba(chrome.hover_rgba)))
                .child("×")
                .on_click({
                    let dismiss = dismiss.clone();
                    move |event, window, cx| dismiss(event, window, cx)
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if is_about_activation_key(event.keystroke.key.as_str()) {
                        let click_event = ClickEvent::default();
                        dismiss(&click_event, window, cx);
                        cx.stop_propagation();
                    } else {
                        cx.propagate();
                    }
                }),
        )
}

fn render_logo_block(chrome: theme::AppChromeColors) -> Div {
    div()
        .size(px(56.0))
        .mb(px(10.0))
        .rounded(px(16.0))
        .border_1()
        .border_color(rgba(chrome.border_rgba))
        .bg(rgba(chrome.panel_surface_rgba))
        .flex()
        .items_center()
        .justify_center()
        .shadow(vec![gpui::BoxShadow {
            color: rgba(0x00000033).into(),
            offset: gpui::point(px(0.), px(4.)),
            blur_radius: px(12.),
            spread_radius: px(0.),
        }])
        .child(
            svg()
                .external_path(crate::utils::get_logo_path())
                .size(px(36.0))
                .text_color(rgb(chrome.accent_hex)),
        )
}

fn render_title_version(chrome: theme::AppChromeColors) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .text_size(px(28.0))
                .line_height(px(34.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(chrome.text_primary_hex))
                .child(branding::APP_NAME),
        )
        .child(
            div()
                .h(px(20.0))
                .mt(px(6.0))
                .px(px(10.0))
                .rounded(px(12.0))
                .border_1()
                .border_color(rgba(chrome.accent_badge_border_rgba))
                .bg(rgba(chrome.accent_badge_bg_rgba))
                .flex()
                .items_center()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(chrome.accent_badge_text_hex))
                .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
        )
}

fn render_tagline(chrome: theme::AppChromeColors) -> Div {
    div()
        .w(px(440.0))
        .max_w_full()
        .mt(px(6.0))
        .text_size(px(13.0))
        .line_height(px(18.0))
        .text_center()
        .text_color(rgba(chrome.text_muted_rgba))
        .child(branding::TAGLINE)
}

fn render_creator_row(chrome: theme::AppChromeColors) -> Div {
    div()
        .mt(px(8.0))
        .h(px(30.0))
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .size(px(32.0))
                .rounded(px(999.0))
                .border_1()
                .border_color(rgba(chrome.border_rgba))
                .bg(rgba(chrome.panel_surface_rgba))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(chrome.accent_hex))
                .child("JL"),
        )
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba(chrome.text_muted_rgba))
                .child("Created by John Lindquist"),
        )
}

fn render_quick_actions(chrome: theme::AppChromeColors, actions: &AboutSurfaceActions) -> Div {
    div()
        .mt(px(10.0))
        .w_full()
        .max_w(px(500.0))
        .min_w(px(0.0))
        .flex()
        .flex_wrap()
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .child(action_button_with_min_width(
            "about-open-github",
            "Open GitHub repo",
            chrome,
            actions.open_github.clone(),
            true,
            128.0,
        ))
        .child(action_button_with_min_width(
            "about-open-discord",
            "Open Discord",
            chrome,
            actions.open_discord.clone(),
            true,
            128.0,
        ))
        .child(action_button_with_min_width(
            "about-follow-x",
            "Follow on X",
            chrome,
            actions.follow_x.clone(),
            true,
            128.0,
        ))
}

fn render_update_card(
    chrome: theme::AppChromeColors,
    update_state: UpdateState,
    actions: &AboutSurfaceActions,
) -> Div {
    let (status, label, enabled, handler) = match update_state {
        UpdateState::Idle => (
            format!("Version v{}", env!("CARGO_PKG_VERSION")),
            "Check for Updates",
            true,
            actions.check_updates.clone(),
        ),
        UpdateState::Checking => (
            "Checking…".to_string(),
            "Checking…",
            false,
            actions.check_updates.clone(),
        ),
        UpdateState::UpToDate => (
            "Up to date".to_string(),
            "Check for Updates",
            true,
            actions.check_updates.clone(),
        ),
        UpdateState::Available { version, .. } => (
            format!("Update Available: v{version}"),
            "Download",
            true,
            actions.open_release.clone(),
        ),
        // TODO(branding): add semantic success token
        UpdateState::Error(_) => (
            "Check failed".to_string(),
            "Check for Updates",
            true,
            actions.check_updates.clone(),
        ),
    };

    div()
        .mt(px(12.0))
        .w_full()
        .max_w(px(500.0))
        .min_h(px(60.0))
        .p(px(12.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(rgba(chrome.border_rgba))
        .bg(rgba(chrome.panel_surface_rgba))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .shadow(vec![gpui::BoxShadow {
            color: rgba(0x0000001a).into(),
            offset: gpui::point(px(0.), px(2.)),
            blur_radius: px(6.),
            spread_radius: px(0.),
        }])
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(chrome.text_primary_hex))
                        .child("Updates"),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(18.0))
                        .min_w(px(0.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .text_color(rgba(chrome.text_muted_rgba))
                        .child(status),
                ),
        )
        .child(action_button(
            "about-update-button",
            label,
            chrome,
            handler,
            enabled,
        ))
}

fn render_acknowledgements(
    chrome: theme::AppChromeColors,
    state: &AboutState,
    actions: &AboutSurfaceActions,
) -> Div {
    div()
        .mt(px(8.0))
        .w_full()
        .max_w(px(500.0))
        .rounded(px(9.0))
        .border_1()
        .border_color(rgba(chrome.border_rgba))
        .child(
            div()
                .id("about-acknowledgements-toggle")
                .tab_index(0)
                .h(px(34.0))
                .px(px(12.0))
                .rounded(px(9.0))
                .flex()
                .items_center()
                .justify_between()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(chrome.text_primary_hex))
                .cursor_pointer()
                .hover(|style| style.bg(rgba(chrome.hover_rgba)))
                .focus_visible(|style| style.bg(rgba(chrome.hover_rgba)))
                .child("Acknowledgements")
                .child(if state.acks_open { "−" } else { "+" })
                .on_click({
                    let toggle = actions.toggle_acknowledgements.clone();
                    move |event, window, cx| toggle(event, window, cx)
                })
                .on_key_down({
                    let toggle = actions.toggle_acknowledgements.clone();
                    move |event: &KeyDownEvent, window, cx| {
                        if is_about_activation_key(event.keystroke.key.as_str()) {
                            let click_event = ClickEvent::default();
                            toggle(&click_event, window, cx);
                            cx.stop_propagation();
                        } else {
                            cx.propagate();
                        }
                    }
                }),
        )
        .when(state.acks_open, |container| {
            container.child(
                div()
                    .pt(px(8.0))
                    .px(px(12.0))
                    .pb(px(10.0))
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(rgba(chrome.text_muted_rgba))
                    .child("Powered by GPUI, ureq, tray-icon, resvg, and the Rust ecosystem."),
            )
        })
}

fn render_footer(chrome: theme::AppChromeColors) -> Div {
    div()
        .mt(px(20.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .max_w_full()
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .text_size(px(11.0))
        .text_color(rgba(chrome.text_muted_rgba))
        .child("© John Lindquist · Built with GPUI")
}

fn action_button(
    id: &'static str,
    label: &'static str,
    chrome: theme::AppChromeColors,
    handler: AboutClickHandler,
    enabled: bool,
) -> impl IntoElement {
    action_button_with_min_width(id, label, chrome, handler, enabled, 142.0)
}

fn action_button_with_min_width(
    id: &'static str,
    label: &'static str,
    chrome: theme::AppChromeColors,
    handler: AboutClickHandler,
    enabled: bool,
    min_width: f32,
) -> impl IntoElement {
    let mut label_element = div()
        .min_w(px(0.0))
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .child(label);
    if enabled {
        label_element = label_element.cursor_pointer();
    }
    let mut button = div()
        .id(id)
        .h(px(34.0))
        .min_w(px(min_width))
        .px(px(12.0))
        .rounded(px(9.0))
        .border_1()
        .border_color(rgba(chrome.border_rgba))
        .bg(rgba(chrome.input_surface_rgba))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(if enabled {
            rgb(chrome.text_primary_hex)
        } else {
            rgba(chrome.text_hint_rgba)
        })
        .child(label_element);

    if enabled {
        button = button
            .tab_index(0)
            .cursor_pointer()
            .hover(|style| style.bg(rgba(chrome.hover_rgba)))
            .focus_visible(|style| style.bg(rgba(chrome.hover_rgba)))
            .on_click({
                let handler = handler.clone();
                move |event, window, cx| handler(event, window, cx)
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if is_about_activation_key(event.keystroke.key.as_str()) {
                    let click_event = ClickEvent::default();
                    handler(&click_event, window, cx);
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            });
    } else {
        button = button.cursor_default();
    }

    button
}
