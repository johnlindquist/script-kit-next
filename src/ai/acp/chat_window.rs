//! Detachable AI chat window.
//!
//! Creates a separate PopUp window for the ACP chat that persists
//! independently from the main Script Kit panel.

use std::sync::{Mutex, OnceLock};

use gpui::{px, AnyWindowHandle, App, AppContext as _, WindowBounds, WindowKind, WindowOptions};

/// Global handle to the detached AI chat window.
static CHAT_WINDOW: OnceLock<Mutex<Option<AnyWindowHandle>>> = OnceLock::new();

/// Check if the detached AI chat window is open.
pub fn is_chat_window_open() -> bool {
    let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}

/// Open (or focus) the detached AI chat window.
pub fn open_chat_window(cx: &mut App) -> anyhow::Result<()> {
    // If already open, just focus it
    let existing = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = existing {
        let _ = handle.update(cx, |_root, window, _cx| {
            window.activate_window();
        });
        return Ok(());
    }

    // Restore saved window bounds, or use defaults
    let window_bounds =
        crate::window_state::load_window_bounds(crate::window_state::WindowRole::AcpChat)
            .map(|persisted| persisted.to_gpui())
            .unwrap_or_else(|| {
                WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point {
                        x: px(100.0),
                        y: px(100.0),
                    },
                    size: gpui::Size {
                        width: px(520.0),
                        height: px(600.0),
                    },
                })
            });

    let window_options = WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("AI Chat".into()),
            appears_transparent: true,
            traffic_light_position: Some(gpui::Point {
                x: px(8.),
                y: px(7.),
            }),
        }),
        window_background: gpui::WindowBackgroundAppearance::Blurred,
        focus: true,
        show: true,
        kind: WindowKind::PopUp,
        ..Default::default()
    };

    let handle = cx.open_window(window_options, |window, cx| {
        let theme = crate::theme::get_cached_theme();
        let view = cx.new(|_cx| ChatWindowPlaceholder);

        // Set background color
        window.set_background_appearance(gpui::WindowBackgroundAppearance::Blurred);

        view
    })?;

    // Store the handle
    {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(handle.into());
        }
    }

    tracing::info!("acp_chat_window_opened");
    Ok(())
}

/// Close the detached AI chat window.
#[allow(dead_code)]
pub fn close_chat_window(cx: &mut App) {
    let existing = {
        let slot = CHAT_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(handle) = existing {
        let _ = handle.update(cx, |_root, window, _cx| {
            // Save window bounds before closing
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(
                crate::window_state::WindowRole::AcpChat,
                wb,
            );
            window.remove_window();
        });
    }
}

/// Minimal placeholder view for the detached chat window.
struct ChatWindowPlaceholder;

impl gpui::Render for ChatWindowPlaceholder {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::{div, prelude::*, rgb};
        let theme = crate::theme::get_cached_theme();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(div().text_base().opacity(0.7).child("AI Chat Window"))
            .child(
                div()
                    .pt(px(8.0))
                    .text_sm()
                    .opacity(0.45)
                    .child("Detached chat \u{2014} full implementation coming soon"),
            )
            .child(
                div()
                    .pt(px(4.0))
                    .text_xs()
                    .opacity(0.35)
                    .text_color(rgb(theme.colors.accent.selected))
                    .child("\u{2318}W to close"),
            )
    }
}
