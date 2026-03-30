use gpui::SharedString;
use std::time::Duration;

use crate::dictation::types::{DictationLevel, DictationSessionPhase};
use crate::dictation::visualizer::bars_for_level;

// ---------------------------------------------------------------------------
// Overlay geometry & waveform contract constants
// ---------------------------------------------------------------------------

/// Compact pill width in pixels.
pub(crate) const OVERLAY_WIDTH_PX: f32 = 220.0;
/// Compact pill height in pixels (matches vercel-voice 36px overlay).
pub(crate) const OVERLAY_HEIGHT_PX: f32 = 36.0;
/// Fully-rounded corner radius (half of height for pill shape).
pub(crate) const OVERLAY_RADIUS_PX: f32 = 18.0;
/// Horizontal padding inside the pill.
pub(crate) const OVERLAY_HORIZONTAL_PADDING_PX: f32 = 16.0;
/// Gap between inner content columns.
pub(crate) const OVERLAY_CONTENT_GAP_PX: f32 = 12.0;
/// Font size for timer, status, and transcript text.
pub(crate) const STATUS_TEXT_SIZE_PX: f32 = 11.5;
/// Right-side spacer width to balance the timer column.
pub(crate) const TIMER_SPACER_WIDTH_PX: f32 = 32.0;

/// Number of waveform bars (matches vercel-voice 9-bar visualizer).
pub(crate) const WAVEFORM_BAR_COUNT: usize = 9;
/// Width of each waveform bar in pixels.
pub(crate) const WAVEFORM_BAR_WIDTH_PX: f32 = 4.0;
/// Gap between waveform bars in pixels.
pub(crate) const WAVEFORM_BAR_GAP_PX: f32 = 4.0;
/// Minimum waveform bar height (silent level).
pub(crate) const WAVEFORM_BAR_MIN_HEIGHT_PX: f32 = 4.0;
/// Maximum waveform bar height (peak level).
pub(crate) const WAVEFORM_BAR_MAX_HEIGHT_PX: f32 = 20.0;

/// Number of transcribing-state dots.
pub(crate) const TRANSCRIBING_DOT_COUNT: usize = 3;
/// Diameter of each transcribing dot.
pub(crate) const TRANSCRIBING_DOT_SIZE_PX: f32 = 4.0;
/// Gap between transcribing dots.
pub(crate) const TRANSCRIBING_DOT_GAP_PX: f32 = 4.0;

/// Threshold: if any bar exceeds this, we treat audio as "active" (green).
const SOUND_THRESHOLD: f32 = 0.15;

// ---------------------------------------------------------------------------
// Overlay helper functions
// ---------------------------------------------------------------------------

/// Format elapsed duration as `M:SS` for the compact timer display.
pub(crate) fn format_elapsed(elapsed: Duration) -> SharedString {
    let elapsed_secs = elapsed.as_secs();
    format!("{}:{:02}", elapsed_secs / 60, elapsed_secs % 60).into()
}

/// Compute waveform bar opacity from a 0.0–1.0 audio level.
///
/// Matches vercel-voice JS: `clamp(0.3, value * 1.5, 1.0)`.
pub(crate) fn waveform_bar_opacity(level: f32) -> f32 {
    (level.clamp(0.0, 1.0) * 1.5).clamp(0.3, 1.0)
}

/// Compute waveform bar height from a 0.0–1.0 audio level.
///
/// Matches vercel-voice JS: `4 + pow(v, 0.7) * 16`, clamped to max height.
pub(crate) fn waveform_bar_height(level: f32) -> f32 {
    (WAVEFORM_BAR_MIN_HEIGHT_PX
        + level.clamp(0.0, 1.0).powf(0.7)
            * (WAVEFORM_BAR_MAX_HEIGHT_PX - WAVEFORM_BAR_MIN_HEIGHT_PX))
        .min(WAVEFORM_BAR_MAX_HEIGHT_PX)
}

/// Returns true if any bar exceeds the sound threshold.
pub(crate) fn has_sound(bars: &[f32; WAVEFORM_BAR_COUNT]) -> bool {
    bars.iter().any(|&bar| bar > SOUND_THRESHOLD)
}

/// Staggered opacities for the 3-dot transcribing animation.
pub(crate) fn transcribing_dot_opacities() -> [f32; TRANSCRIBING_DOT_COUNT] {
    [OPACITY_SELECTED, OPACITY_ACTIVE, OPACITY_SELECTED]
}

/// Snapshot of the dictation overlay's visual state.
///
/// Updated on every level/phase change and consumed by the overlay renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct DictationOverlayState {
    pub phase: DictationSessionPhase,
    pub elapsed: Duration,
    pub bars: [f32; WAVEFORM_BAR_COUNT],
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
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED, OPACITY_SUBTLE, OPACITY_TEXT_MUTED};
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
                let timer_text = format_elapsed(*elapsed);
                let active = has_sound(bars);

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    // Left: timer
                    .child(
                        div().flex().flex_row().items_center().gap(px(8.)).child(
                            div()
                                .text_size(px(STATUS_TEXT_SIZE_PX))
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
                            .child(render_waveform_bars(bars, active)),
                    )
                    // Right: spacer to balance the timer width
                    .child(div().w(px(TIMER_SPACER_WIDTH_PX)))
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
                        .text_size(px(STATUS_TEXT_SIZE_PX))
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
                            .text_size(px(STATUS_TEXT_SIZE_PX))
                            .font_family(FONT_MONO)
                            .text_color(muted_text)
                            .overflow_hidden()
                            .child(err_text),
                    )
            }
            // Idle / Delivering — render nothing meaningful
            _ => div(),
        };

        // Compact pill: OVERLAY_HEIGHT_PX tall, fully-rounded, border stroke.
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .h(px(OVERLAY_HEIGHT_PX))
            .px(px(OVERLAY_HORIZONTAL_PADDING_PX))
            .gap(px(OVERLAY_CONTENT_GAP_PX))
            .bg(surface_bg)
            .rounded(px(OVERLAY_RADIUS_PX))
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

/// Render waveform bars matching vercel-voice `.bars-container` styling.
///
/// Uses theme success color when sound is detected, error color when silent.
fn render_waveform_bars(bars: &[f32; WAVEFORM_BAR_COUNT], active: bool) -> impl IntoElement {
    let theme = get_cached_theme();
    let bar_hex = if active {
        theme.colors.ui.success
    } else {
        theme.colors.ui.error
    };

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(WAVEFORM_BAR_GAP_PX))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX));

    for &level in bars {
        let bar_color = bar_hex.with_opacity(waveform_bar_opacity(level));
        container = container.child(
            div()
                .w(px(WAVEFORM_BAR_WIDTH_PX))
                .h(px(waveform_bar_height(level)))
                .min_h(px(WAVEFORM_BAR_MIN_HEIGHT_PX))
                .bg(bar_color)
                .rounded(px(WAVEFORM_BAR_WIDTH_PX)),
        );
    }

    container
}

/// Render dots for the transcribing state.
///
/// Uses theme success color at staggered opacities to suggest pulsing motion.
fn render_transcribing_dots() -> impl IntoElement {
    let theme = get_cached_theme();

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(TRANSCRIBING_DOT_GAP_PX))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX));

    for &opacity in &transcribing_dot_opacities() {
        let dot_color = theme.colors.ui.success.with_opacity(opacity);
        container = container.child(
            div()
                .w(px(TRANSCRIBING_DOT_SIZE_PX))
                .h(px(TRANSCRIBING_DOT_SIZE_PX))
                .rounded(px(TRANSCRIBING_DOT_SIZE_PX / 2.0))
                .bg(dot_color),
        );
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

    // Compact pill: OVERLAY_WIDTH_PX x OVERLAY_HEIGHT_PX, centered near top of screen.
    let bounds = gpui::Bounds {
        origin: gpui::Point {
            x: px(0.),
            y: px(80.),
        },
        size: gpui::Size {
            width: px(OVERLAY_WIDTH_PX),
            height: px(OVERLAY_HEIGHT_PX),
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
        handle
            .update(cx, |_view, window, _cx| {
                window.remove_window();
            })
            .map_err(|error| {
                anyhow::anyhow!("failed to close dictation overlay window: {error}")
            })?;
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
