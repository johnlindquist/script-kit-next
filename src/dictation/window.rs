use gpui::SharedString;
use std::time::Duration;

use crate::dictation::types::{DictationLevel, DictationSessionPhase};
use crate::dictation::visualizer::bars_for_level;

/// Threshold: if any bar exceeds this, we treat audio as "active" (green).
const SOUND_THRESHOLD: f32 = 0.15;

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

use crate::list_item::FONT_MONO;
use crate::theme::get_cached_theme;
use crate::theme::opacity::{
    OPACITY_ACTIVE, OPACITY_SELECTED, OPACITY_SUBTLE, OPACITY_TEXT_MUTED,
};
use crate::ui_foundation::HexColorExt;

use parking_lot::Mutex;
use std::sync::OnceLock;

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

        // Translucent pill surface derived from theme background.
        let surface_bg = if theme.is_vibrancy_enabled() {
            theme.colors.background.main.with_opacity(OPACITY_SUBTLE)
        } else {
            theme.colors.background.main.with_opacity(OPACITY_SELECTED)
        };
        let border_color = theme.colors.ui.border.with_opacity(OPACITY_SUBTLE);

        let timer_color = theme.colors.text.muted.with_opacity(OPACITY_SELECTED);
        let muted_text = theme.colors.text.muted.with_opacity(OPACITY_TEXT_MUTED);
        let text_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);

        let phase = &self.state.phase;
        let bars = &self.state.bars;
        let elapsed = &self.state.elapsed;

        // 3-column inner content matching vercel-voice grid layout:
        //   left (timer) | center (bars/dots/status) | right (spacer)
        let inner = match phase {
            DictationSessionPhase::Recording => {
                let elapsed_secs = elapsed.as_secs();
                let timer_text: SharedString =
                    format!("{}:{:02}", elapsed_secs / 60, elapsed_secs % 60).into();
                let has_sound = bars.iter().any(|&b| b > SOUND_THRESHOLD);

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    // Left: timer
                    .child(
                        div().flex().flex_row().items_center().gap(px(8.)).child(
                            div()
                                .text_size(px(11.5))
                                .font_family(FONT_MONO)
                                .text_color(timer_color)
                                .child(timer_text),
                        ),
                    )
                    // Center: waveform bars (flex-grow to fill)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .child(render_waveform_bars(bars, has_sound)),
                    )
                    // Right: spacer to balance the timer width
                    .child(div().w(px(32.)))
            }
            DictationSessionPhase::Transcribing => {
                // 3 green dots matching vercel-voice .transcribing-dots
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .child(render_transcribing_dots())
            }
            DictationSessionPhase::Finished => div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .w_full()
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_family(FONT_MONO)
                        .text_color(text_color)
                        .overflow_hidden()
                        .child(finished_label(&self.state.transcript)),
                ),
            DictationSessionPhase::Failed(ref msg) => {
                let err_text: SharedString = format!("Error: {msg}").into();
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_family(FONT_MONO)
                            .text_color(muted_text)
                            .overflow_hidden()
                            .child(err_text),
                    )
            }
            // Idle / Delivering — render nothing meaningful
            _ => div(),
        };

        // Compact pill: 36px tall, fully-rounded (18px), border stroke.
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .h(px(36.))
            .px(px(16.))
            .gap(px(12.))
            .bg(surface_bg)
            .rounded(px(18.))
            .border_1()
            .border_color(border_color)
            .child(inner)
    }
}

/// Format a human-readable label for the finished overlay state.
///
/// - Empty/whitespace transcript → `"Done"`
/// - Short transcript (≤28 chars) → `"Done · <transcript>"`
/// - Long transcript (>28 chars) → `"Done · <first 28 chars>…"`
pub(crate) fn finished_label(transcript: &SharedString) -> SharedString {
    let owned = transcript.to_string();
    let trimmed = owned.trim();
    if trimmed.is_empty() {
        return "Done".into();
    }
    const MAX_CHARS: usize = 28;
    let mut preview = String::new();
    let mut chars = trimmed.chars();
    for _ in 0..MAX_CHARS {
        let Some(ch) = chars.next() else {
            break;
        };
        preview.push(ch);
    }
    if chars.next().is_some() {
        format!("Done · {preview}…").into()
    } else {
        format!("Done · {preview}").into()
    }
}

/// Render 9 waveform bars matching vercel-voice `.bars-container` styling.
///
/// Bars are 4px wide, 4px gap, 4px min-height, 20px max-height, fully rounded.
/// Uses theme success color when sound is detected, error color when silent.
fn render_waveform_bars(bars: &[f32; 9], has_sound: bool) -> impl IntoElement {
    let theme = get_cached_theme();
    let bar_hex = if has_sound {
        theme.colors.ui.success
    } else {
        theme.colors.ui.error
    };

    let bar_elements: Vec<_> = bars
        .iter()
        .map(|&height| {
            // Opacity: clamp(0.3, value * 1.5, 1.0) — matching vercel-voice JS
            let bar_opacity = (height * 1.5).clamp(0.3, 1.0);
            let bar_color = bar_hex.with_opacity(bar_opacity);
            // Height: 4 + pow(v, 0.7) * 16 — matching vercel-voice JS, clamped to 20px
            let bar_h = (4.0 + height.powf(0.7) * 16.0).min(20.0);
            div()
                .w(px(4.))
                .h(px(bar_h))
                .min_h(px(4.))
                .bg(bar_color)
                .rounded(px(4.))
        })
        .collect();

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .h(px(20.));

    for bar in bar_elements {
        container = container.child(bar);
    }

    container
}

/// Render 3 dots for the transcribing state.
///
/// Uses theme success color at staggered opacities to suggest pulsing motion.
fn render_transcribing_dots() -> impl IntoElement {
    let theme = get_cached_theme();
    let opacities = [OPACITY_SELECTED, OPACITY_ACTIVE, OPACITY_SELECTED];

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .h(px(20.));

    for &opacity in &opacities {
        let dot_color = theme.colors.ui.success.with_opacity(opacity);
        container = container.child(div().w(px(4.)).h(px(4.)).rounded(px(2.)).bg(dot_color));
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
pub fn open_dictation_overlay(
    cx: &mut App,
) -> anyhow::Result<gpui::WindowHandle<DictationOverlay>> {
    use anyhow::Context as _;

    // If already open, return the existing handle.
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    {
        let guard = slot.lock();
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
        .open_window(window_options, |_window, cx| cx.new(DictationOverlay::new))
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
                        let ns_window: cocoa::base::id = msg_send![ns_view, window];
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
        let mut guard = slot.lock();
        *guard = Some(handle);
    }

    tracing::info!(category = "DICTATION", "Dictation overlay window opened");
    Ok(handle)
}

/// Push a new state snapshot into the overlay (no-op if overlay is not open).
pub fn update_dictation_overlay(state: DictationOverlayState, cx: &mut App) -> anyhow::Result<()> {
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let handle = {
        let guard = slot.lock();
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
        let mut guard = slot.lock();
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
    let guard = slot.lock();
    guard.is_some()
}
