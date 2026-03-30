use gpui::SharedString;
use std::time::Duration;

use crate::dictation::types::{DictationLevel, DictationSessionPhase};
use crate::dictation::visualizer::bars_for_level;

// ---------------------------------------------------------------------------
// Overlay geometry & waveform contract constants
// ---------------------------------------------------------------------------

/// Compact pill width in pixels (matches vercel-voice 392px overlay).
pub(crate) const OVERLAY_WIDTH_PX: f32 = 392.0;
/// Compact pill height in pixels (matches vercel-voice 40px overlay).
pub(crate) const OVERLAY_HEIGHT_PX: f32 = 40.0;
/// Fully-rounded corner radius (half of height for pill shape).
pub(crate) const OVERLAY_RADIUS_PX: f32 = 20.0;
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

/// Bottom offset from the screen edge (dock clearance), matching vercel-voice.
const OVERLAY_BOTTOM_OFFSET_PX: f32 = 15.0;

// ---------------------------------------------------------------------------
// Glassmorphism constants (matching vercel-voice RecordingOverlay.css)
// ---------------------------------------------------------------------------

/// Overlay background: rgba(18,18,22,0.24).
const GLASSMORPHISM_BG: u32 = 0x121216;
const GLASSMORPHISM_BG_OPACITY: f32 = 0.24;
/// Border: rgba(255,255,255,0.18).
const GLASSMORPHISM_BORDER: u32 = 0xFFFFFF;
const GLASSMORPHISM_BORDER_OPACITY: f32 = 0.18;

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

/// Pulse cycle duration in seconds (matches vercel-voice 1.4s).
pub(crate) const TRANSCRIBING_PULSE_PERIOD_SECS: f64 = 1.4;
/// Stagger between consecutive dots in seconds (matches vercel-voice 0.2s).
pub(crate) const TRANSCRIBING_PULSE_STAGGER_SECS: f64 = 0.2;
/// Minimum dot opacity during pulse.
const PULSE_OPACITY_MIN: f32 = 0.3;
/// Maximum dot opacity during pulse.
const PULSE_OPACITY_MAX: f32 = 1.0;

/// Static opacities for reduced-motion fallback (no animation).
pub(crate) fn transcribing_dot_opacities_static() -> [f32; TRANSCRIBING_DOT_COUNT] {
    [OPACITY_SELECTED, OPACITY_ACTIVE, OPACITY_SELECTED]
}

/// Compute time-based staggered pulse opacities for the 3-dot transcribing animation.
///
/// Each dot follows a sine-wave pulse with a per-dot phase offset:
/// `opacity = min + (max - min) * (0.5 + 0.5 * sin(2π * (t - i * stagger) / period))`
///
/// Matches vercel-voice: 1.4s cycle, 0.2s stagger between dots.
pub(crate) fn transcribing_dot_opacities_at(elapsed_secs: f64) -> [f32; TRANSCRIBING_DOT_COUNT] {
    let mut opacities = [0.0_f32; TRANSCRIBING_DOT_COUNT];
    for (i, opacity) in opacities.iter_mut().enumerate() {
        let phase = elapsed_secs - (i as f64 * TRANSCRIBING_PULSE_STAGGER_SECS);
        let t = std::f64::consts::TAU * phase / TRANSCRIBING_PULSE_PERIOD_SECS;
        let wave = 0.5 + 0.5 * t.sin();
        *opacity = PULSE_OPACITY_MIN + (PULSE_OPACITY_MAX - PULSE_OPACITY_MIN) * wave as f32;
    }
    opacities
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
    div, prelude::*, px, rgb, App, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent,
    ParentElement, Render, Styled, Task, Window, WindowBounds, WindowOptions,
};

use crate::list_item::FONT_MONO;
use crate::theme::get_cached_theme;
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED, OPACITY_SUBTLE, OPACITY_TEXT_MUTED};
use crate::ui_foundation::HexColorExt;

use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// Monotonic generation counter for overlay sessions.
///
/// Incremented each time a new overlay window is opened. Async tasks
/// (pump, scheduled close) compare their captured generation against the
/// current value and bail when stale, preventing a delayed close from
/// killing a newly opened overlay.
static OVERLAY_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Global handle so we can reach the overlay from any callsite.
static DICTATION_OVERLAY_WINDOW: OnceLock<Mutex<Option<gpui::WindowHandle<DictationOverlay>>>> =
    OnceLock::new();

/// Callback type for overlay escape actions (abort dictation).
type OverlayAbortCallback = Box<dyn Fn(&mut App) + Send + Sync + 'static>;

/// Global abort callback set by the dictation runtime.
static OVERLAY_ABORT_CALLBACK: Mutex<Option<OverlayAbortCallback>> = Mutex::new(None);

/// Register a callback to be invoked when the user confirms abort via
/// double-Escape in the overlay.
pub fn set_overlay_abort_callback(callback: impl Fn(&mut App) + Send + Sync + 'static) {
    *OVERLAY_ABORT_CALLBACK.lock() = Some(Box::new(callback));
}

/// Interval between animation ticks for the transcribing dot pulse (ms).
const TRANSCRIBING_TICK_MS: u64 = 50;

/// The GPUI entity that renders the compact dictation pill.
pub struct DictationOverlay {
    state: DictationOverlayState,
    focus_handle: FocusHandle,
    /// When the transcribing animation started (for pulse phase computation).
    transcribing_started_at: Option<Instant>,
    /// Whether the user has "Reduce motion" enabled in system accessibility.
    reduced_motion: bool,
    /// Keeps the transcribing tick loop alive; dropped when phase changes.
    _animation_task: Option<Task<()>>,
}

impl DictationOverlay {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: DictationOverlayState::default(),
            focus_handle: cx.focus_handle(),
            transcribing_started_at: None,
            reduced_motion: crate::platform::prefers_reduced_motion(),
            _animation_task: None,
        }
    }

    /// Replace the visual state snapshot (called from the dictation runtime).
    pub fn set_state(&mut self, state: DictationOverlayState, cx: &mut Context<Self>) {
        let entering_transcribing = state.phase == DictationSessionPhase::Transcribing
            && self.state.phase != DictationSessionPhase::Transcribing;
        let leaving_transcribing = state.phase != DictationSessionPhase::Transcribing
            && self.state.phase == DictationSessionPhase::Transcribing;

        self.state = state;

        if entering_transcribing && !self.reduced_motion {
            self.transcribing_started_at = Some(Instant::now());
            // Spawn a tick loop that re-renders every TRANSCRIBING_TICK_MS so
            // the sine-wave pulse progresses smoothly.
            self._animation_task = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(TRANSCRIBING_TICK_MS))
                        .await;
                    let should_stop = this
                        .update(cx, |view, cx| {
                            if view.state.phase != DictationSessionPhase::Transcribing {
                                return true;
                            }
                            cx.notify();
                            false
                        })
                        .unwrap_or(true);
                    if should_stop {
                        break;
                    }
                }
            }));
        } else if leaving_transcribing {
            self.transcribing_started_at = None;
            self._animation_task = None;
        }

        cx.notify();
    }

    /// Handle key-down events for the overlay.
    ///
    /// State machine:
    /// - Recording + Escape → Confirming (show Abort/Resume)
    /// - Confirming + Escape → Abort (invoke callback, close overlay)
    /// - Confirming + any other key → Resume Recording
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let is_escape = crate::ui_foundation::is_key_escape(key);

        match &self.state.phase {
            DictationSessionPhase::Recording if is_escape => {
                tracing::info!(
                    category = "DICTATION",
                    "Escape pressed during recording, entering confirming state"
                );
                let phase = DictationSessionPhase::Confirming;
                let _ = crate::dictation::set_overlay_phase(phase.clone());
                self.state.phase = phase;
                cx.notify();
                cx.stop_propagation();
            }
            DictationSessionPhase::Confirming if is_escape => {
                tracing::info!(
                    category = "DICTATION",
                    "Second Escape pressed, aborting dictation"
                );
                let callback = OVERLAY_ABORT_CALLBACK.lock().take();
                if let Some(cb) = callback {
                    cb(cx);
                }
                cx.stop_propagation();
            }
            DictationSessionPhase::Confirming => {
                tracing::info!(
                    category = "DICTATION",
                    key,
                    "Non-Escape key pressed during confirming, resuming recording"
                );
                let phase = DictationSessionPhase::Recording;
                let _ = crate::dictation::set_overlay_phase(phase.clone());
                self.state.phase = phase;
                cx.notify();
                cx.stop_propagation();
            }
            _ => {
                cx.propagate();
            }
        }
    }
}

impl Focusable for DictationOverlay {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DictationOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = get_cached_theme();

        // Glassmorphism surface matching vercel-voice: rgba(18,18,22,0.24)
        // with backdrop blur and rgba(255,255,255,0.18) border.
        let mut surface_bg = rgb(GLASSMORPHISM_BG);
        surface_bg.a = GLASSMORPHISM_BG_OPACITY;
        let mut border_color = rgb(GLASSMORPHISM_BORDER);
        border_color.a = GLASSMORPHISM_BORDER_OPACITY;

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
            DictationSessionPhase::Confirming => {
                let abort_color = theme.colors.ui.error.with_opacity(OPACITY_ACTIVE);
                let resume_color = theme.colors.text.muted.with_opacity(OPACITY_TEXT_MUTED);
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(px(16.))
                    .w_full()
                    .child(
                        div()
                            .text_size(px(STATUS_TEXT_SIZE_PX))
                            .font_family(FONT_MONO)
                            .text_color(abort_color)
                            .child("Esc Abort"),
                    )
                    .child(
                        div()
                            .text_size(px(STATUS_TEXT_SIZE_PX))
                            .font_family(FONT_MONO)
                            .text_color(resume_color)
                            .child("Any key Resume"),
                    )
            }
            DictationSessionPhase::Transcribing => {
                // 3 green dots matching vercel-voice .transcribing-dots
                let dot_opacities = if self.reduced_motion {
                    transcribing_dot_opacities_static()
                } else if let Some(started) = self.transcribing_started_at {
                    transcribing_dot_opacities_at(started.elapsed().as_secs_f64())
                } else {
                    transcribing_dot_opacities_static()
                };
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .child(render_transcribing_dots(&dot_opacities))
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

        // Inner pill surface — glassmorphism styling, padding, rounded corners.
        let surface = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .px(px(OVERLAY_HORIZONTAL_PADDING_PX))
            .gap(px(OVERLAY_CONTENT_GAP_PX))
            .bg(surface_bg)
            .rounded(px(OVERLAY_RADIUS_PX))
            .border_1()
            .border_color(border_color)
            .child(inner);

        // Outer root claims the full popup content bounds so no GPUI inset
        // gap remains between the pill and the native window frame.
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .w_full()
            .h_full()
            .child(surface)
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
/// Uses theme success color at the given per-dot opacities.
/// When animated, opacities come from `transcribing_dot_opacities_at()`;
/// under reduced-motion, from `transcribing_dot_opacities_static()`.
fn render_transcribing_dots(opacities: &[f32; TRANSCRIBING_DOT_COUNT]) -> impl IntoElement {
    let theme = get_cached_theme();

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(TRANSCRIBING_DOT_GAP_PX))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX));

    for &opacity in opacities {
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

/// Calculate bottom-center bounds for the overlay on the active display.
///
/// Resolves the active display via `get_active_display()`, which returns the
/// screen containing the currently focused window (key window). This ensures the
/// overlay appears on the display the user is actively working on, not wherever
/// the mouse cursor happens to be parked.
/// Falls back to the first visible display, then to a hardcoded 1920×1080 default.
/// Positions the pill centered horizontally and `OVERLAY_BOTTOM_OFFSET_PX` above
/// the bottom edge of the chosen display's visible area.
fn calculate_overlay_bottom_center_bounds() -> gpui::Bounds<gpui::Pixels> {
    // Prefer the display with the key window (active/focused display).
    // Fall back to the first visible display (primary) if unavailable.
    let active_display = crate::platform::get_active_display().or_else(|| {
        let displays = crate::platform::get_macos_visible_displays();
        displays.into_iter().next()
    });

    let (vis_x, vis_y, vis_w, vis_h) = if let Some(display) = active_display {
        let v = &display.visible_area;
        (
            v.origin_x as f32,
            v.origin_y as f32,
            v.width as f32,
            v.height as f32,
        )
    } else {
        // Fallback: assume 1920x1080 with 24px menu bar.
        (0.0_f32, 24.0_f32, 1920.0_f32, 1056.0_f32)
    };

    let x = vis_x + (vis_w - OVERLAY_WIDTH_PX) / 2.0;
    let y = vis_y + vis_h - OVERLAY_BOTTOM_OFFSET_PX - OVERLAY_HEIGHT_PX;

    tracing::debug!(
        category = "DICTATION",
        x,
        y,
        vis_x,
        vis_y,
        vis_w,
        vis_h,
        "Calculated overlay bottom-center position"
    );

    gpui::Bounds {
        origin: gpui::Point { x: px(x), y: px(y) },
        size: gpui::Size {
            width: px(OVERLAY_WIDTH_PX),
            height: px(OVERLAY_HEIGHT_PX),
        },
    }
}

/// Open the dictation overlay as a compact floating pill.
///
/// Creates a `WindowKind::PopUp` window with blurred background and vibrancy.
/// Returns a handle that can be used to update or close the overlay.
pub fn open_dictation_overlay(
    cx: &mut App,
) -> anyhow::Result<gpui::WindowHandle<DictationOverlay>> {
    use anyhow::Context as _;

    // If already open AND the native window is still alive, return the
    // existing handle.  If the handle is stale (window was closed natively),
    // clear the slot and fall through to create a fresh one.
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    {
        let mut guard = slot.lock();
        if let Some(handle) = *guard {
            let alive = handle
                .update(cx, |_view, _window, _cx| {})
                .is_ok();
            if alive {
                return Ok(handle);
            }
            // Stale handle — clear and recreate.
            tracing::warn!(
                category = "DICTATION",
                "Overlay handle was stale, clearing slot"
            );
            *guard = None;
        }
    }

    let theme = get_cached_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Bottom-center positioning matching vercel-voice: centered horizontally,
    // 15px above the bottom of the active display's visible area.
    let bounds = calculate_overlay_bottom_center_bounds();

    // focus: false — the overlay must not activate the app or surface the
    // main window.  We bring the overlay to front via orderFrontRegardless
    // and makeKeyWindow below.
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

    // Only make the overlay key window when the main window is already
    // visible.  When Script Kit is hidden, keying the overlay would pull
    // the entire app forward — the dictation hotkey must never surface
    // the main window.  orderFrontRegardless alone is enough to show the
    // popup without activating the app.
    let should_key_overlay = crate::is_main_window_visible();

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_view, window, _cx| {
            if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
                if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
                    use objc::{msg_send, sel, sel_impl};
                    let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
                    // SAFETY: ns_view is a valid NSView from a just-created GPUI window.
                    // orderFrontRegardless brings the window to front without activating the app.
                    // makeKeyWindow is only sent when the main window is already visible,
                    // to deliver Escape key events without pulling a hidden app forward.
                    unsafe {
                        let ns_window: cocoa::base::id = msg_send![ns_view, window];
                        let () = msg_send![ns_window, orderFrontRegardless];
                        if should_key_overlay {
                            let () = msg_send![ns_window, makeKeyWindow];
                        }
                    }
                }
            }
        });
    }

    // Focus the GPUI focus handle so key events (Escape) are delivered —
    // but only when we also made the window key, otherwise this would
    // activate the app.
    if should_key_overlay {
        let _ = handle.update(cx, |view, window, cx| {
            view.focus_handle.focus(window, cx);
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
    *OVERLAY_ABORT_CALLBACK.lock() = None;

    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let handle = {
        let mut guard = slot.lock();
        guard.take()
    };

    if let Some(handle) = handle {
        // Attempt to remove — ignore errors from already-dead windows.
        let result = handle.update(cx, |_view, window, _cx| {
            window.remove_window();
        });
        if result.is_ok() {
            tracing::info!(category = "DICTATION", "Dictation overlay window closed");
        } else {
            tracing::warn!(
                category = "DICTATION",
                "Overlay window already gone when close was called"
            );
        }
    }

    Ok(())
}

/// Check whether the dictation overlay window is currently open.
///
/// Note: this only checks whether the slot holds a handle.  For true
/// liveness validation (which requires `&mut App`), use
/// `open_dictation_overlay` which probes the handle before reuse.
pub fn is_dictation_overlay_open() -> bool {
    let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    let guard = slot.lock();
    guard.is_some()
}

/// Bump the logical overlay session generation.
///
/// Call this on every `DictationToggleOutcome::Started`, even when reusing
/// the same overlay window handle.  This ensures that stale delayed closes
/// from a prior session see a generation mismatch and bail.
pub fn begin_overlay_session() -> u64 {
    OVERLAY_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}

/// Return the current overlay generation counter.
///
/// Async tasks capture this value at spawn time and compare on each tick.
/// When the live value differs, the task is stale and must stop.
pub fn overlay_generation() -> u64 {
    OVERLAY_GENERATION.load(Ordering::SeqCst)
}
