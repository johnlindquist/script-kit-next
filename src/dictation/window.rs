use gpui::SharedString;
use std::time::Duration;

use crate::dictation::types::{DictationLevel, DictationSessionPhase};
use crate::dictation::visualizer::bars_for_level;

/// Snapshot of the dictation overlay's visual state.
///
/// Updated on every level/phase change and consumed by the overlay renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct DictationOverlayState {
    pub phase: DictationSessionPhase,
    pub elapsed: Duration,
    pub bars: [f32; 9],
    pub transcript: SharedString,
}

impl Default for DictationOverlayState {
    fn default() -> Self {
        Self {
            phase: DictationSessionPhase::Idle,
            elapsed: Duration::ZERO,
            bars: bars_for_level(DictationLevel {
                rms: 0.0,
                peak: 0.0,
            }),
            transcript: SharedString::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// GPUI overlay entity + window lifecycle
// ---------------------------------------------------------------------------

use gpui::{
    div, prelude::*, px, App, Context, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions,
};

use crate::theme::get_cached_theme;
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_GHOST, OPACITY_MUTED, OPACITY_TEXT_MUTED};
use crate::ui_foundation::HexColorExt;


use std::sync::{Mutex, OnceLock};

/// Global handle so we can reach the overlay from any callsite.
static DICTATION_OVERLAY_WINDOW: OnceLock<Mutex<Option<gpui::WindowHandle<DictationOverlay>>>> =
    OnceLock::new();

/// The GPUI entity that renders the compact dictation pill.
pub struct DictationOverlay {
    state: DictationOverlayState,
    focus_handle: FocusHandle,
}

impl DictationOverlay {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: DictationOverlayState::default(),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Replace the visual state snapshot (called from the dictation runtime).
    pub fn set_state(&mut self, state: DictationOverlayState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
    }
}

impl Focusable for DictationOverlay {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DictationOverlay {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = get_cached_theme();

        // Surface: ghost-opacity background, compact pill (36px tall).
        let surface_bg = theme.colors.background.main.with_opacity(OPACITY_GHOST);
        let text_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);
        let muted_text = theme.colors.text.primary.with_opacity(OPACITY_TEXT_MUTED);
        let bar_hex = theme.colors.text.primary;

        let phase = &self.state.phase;
        let bars = &self.state.bars;
        let elapsed = &self.state.elapsed;

        // Build the inner content based on current phase.
        let inner = match phase {
            DictationSessionPhase::Recording => {
                let elapsed_secs = elapsed.as_secs();
                let timer_text: SharedString =
                    format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60).into();

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(render_waveform_bars(bars, bar_hex))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(text_color)
                            .child(timer_text),
                    )
            }
            DictationSessionPhase::Transcribing => div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(muted_text)
                        .child(SharedString::from("Transcribing…")),
                ),
            DictationSessionPhase::Finished => div()
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(text_color)
                        .child(self.state.transcript.clone()),
                ),
            DictationSessionPhase::Failed(ref msg) => {
                let err_text: SharedString = format!("Error: {msg}").into();
                div().flex().flex_row().items_center().child(
                    div()
                        .text_size(px(13.))
                        .text_color(muted_text)
                        .child(err_text),
                )
            }
            // Idle / Delivering — render nothing meaningful
            _ => div(),
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .h(px(36.))
            .px(px(12.))
            .bg(surface_bg)
            .rounded(px(8.))
            .child(inner)
    }
}

/// Render 9 waveform bars as a tiny inline element.
fn render_waveform_bars(bars: &[f32; 9], text_hex: u32) -> impl IntoElement {
    let bar_elements: Vec<_> = bars
        .iter()
        .map(|&height| {
            let bar_opacity = OPACITY_MUTED + height * (OPACITY_ACTIVE - OPACITY_MUTED);
            let bar_color = text_hex.with_opacity(bar_opacity);
            div()
                .w(px(3.))
                .h(px(height * 20.0))
                .min_h(px(2.))
                .bg(bar_color)
                .rounded(px(1.))
        })
        .collect();

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .h(px(24.));

    for bar in bar_elements {
        container = container.child(bar);
    }

    container
}

// ---------------------------------------------------------------------------
// Public window lifecycle API
// ---------------------------------------------------------------------------

/// Open the dictation overlay as a compact floating pill.
///
/// Creates a `WindowKind::PopUp` window with blurred background and vibrancy.
/// Returns a handle that can be used to update or close the overlay.
pub fn open_dictation_overlay(cx: &mut App) -> anyhow::Result<gpui::WindowHandle<DictationOverlay>> {
    use anyhow::Context as _;

    // If already open, return the existing handle.
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    {
        let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(handle) = *guard {
            return Ok(handle);
        }
    }

    let theme = get_cached_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Compact pill: 220x36, centered near top of screen.
    let bounds = gpui::Bounds {
        origin: gpui::Point {
            x: px(0.),
            y: px(80.),
        },
        size: gpui::Size {
            width: px(220.),
            height: px(36.),
        },
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false,
        show: true,
        kind: gpui::WindowKind::PopUp,
        ..Default::default()
    };

    let handle = cx
        .open_window(window_options, |_window, cx| {
            cx.new(DictationOverlay::new)
        })
        .context("Failed to open dictation overlay window")?;

    // Configure vibrancy on macOS (never call setLevel on PopUp windows).
    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_view, window, _cx| {
            let is_dark = theme.should_use_dark_vibrancy();
            // Obtain the NSView from the raw window handle, then get its parent NSWindow.
            if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
                if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
                    use objc::{msg_send, sel, sel_impl};
                    let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
                    // SAFETY: ns_view is a valid NSView from a just-created GPUI window.
                    // We obtain the parent NSWindow via the standard -[NSView window] message.
                    // Called on the main thread as required by AppKit.
                    unsafe {
                        let ns_window: cocoa::base::id =
                            msg_send![ns_view, window];
                        crate::platform::configure_secondary_window_vibrancy(
                            ns_window,
                            "Dictation",
                            is_dark,
                        );
                    }
                }
            }
        });
    }

    // Store handle.
    {
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(handle);
    }

    tracing::info!(category = "DICTATION", "Dictation overlay window opened");
    Ok(handle)
}

/// Push a new state snapshot into the overlay (no-op if overlay is not open).
pub fn update_dictation_overlay(state: DictationOverlayState, cx: &mut App) -> anyhow::Result<()> {
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let handle = {
        let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        match *guard {
            Some(h) => h,
            None => return Ok(()), // Not open — nothing to update.
        }
    };

    let _ = handle.update(cx, |view, _window, cx| {
        view.set_state(state, cx);
    });

    Ok(())
}

/// Close the dictation overlay window.
pub fn close_dictation_overlay(cx: &mut App) -> anyhow::Result<()> {
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let handle = {
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        guard.take()
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_view, window, _cx| {
            window.remove_window();
        });
        tracing::info!(category = "DICTATION", "Dictation overlay window closed");
    }

    Ok(())
}

/// Check whether the dictation overlay window is currently open.
pub fn is_dictation_overlay_open() -> bool {
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}
