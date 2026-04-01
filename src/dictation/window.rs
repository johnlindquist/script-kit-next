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
/// Confirming phase uses the same pill height — content swaps inline like
/// vercel-voice (no vertical expansion that would push off-screen).
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
    MouseButton, MouseDownEvent, ParentElement, Render, Styled, Task, Window, WindowBounds,
    WindowOptions,
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

/// Register a callback to be invoked when the user confirms stop via
/// Enter or the Stop button in the overlay.
pub fn set_overlay_abort_callback(callback: impl Fn(&mut App) + Send + Sync + 'static) {
    *OVERLAY_ABORT_CALLBACK.lock() = Some(Box::new(callback));
}

// ---------------------------------------------------------------------------
// Confirming-phase copy constants (single source of truth)
// ---------------------------------------------------------------------------

/// Button label for the Stop action in the confirming overlay.
const CONFIRM_STOP_LABEL: &str = "Stop \u{21b5}";
/// Button label for the Continue action in the confirming overlay.
const CONFIRM_CONTINUE_LABEL: &str = "Continue \u{238b}";
/// Hint text shown in the confirming overlay footer.
const CONFIRM_HINT: &str = "\u{21b5} Stop \u{00b7} \u{238b} Continue";

/// Interval between animation ticks for the transcribing dot pulse (ms).
const TRANSCRIBING_TICK_MS: u64 = 50;

/// What the overlay should do when Escape is pressed in a given phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverlayEscapeAction {
    /// First Escape during recording — show confirmation UI, don't abort yet.
    TransitionToConfirming,
    /// Escape during Confirming — dismiss confirmation and resume recording.
    ResumeRecording,
    /// Enter during Confirming — actually abort the session.
    AbortSession,
    CloseOverlay,
    Propagate,
}

/// Return phase-appropriate (headline, hint) copy for the dictation overlay.
///
/// The headline is the primary status text (e.g. "Listening…", "Stop dictation?").
/// The hint is the footer/shortcut hint (e.g. "Esc Cancel", "↵ Stop · ⎋ Continue").
pub(crate) fn overlay_phase_copy(phase: &DictationSessionPhase) -> (&'static str, &'static str) {
    match phase {
        DictationSessionPhase::Recording => ("Listening\u{2026}", "Esc Cancel"),
        DictationSessionPhase::Confirming => ("Stop dictation?", CONFIRM_HINT),
        DictationSessionPhase::Transcribing => ("Transcribing\u{2026}", "Esc Close"),
        DictationSessionPhase::Delivering => ("Delivering\u{2026}", "Esc Close"),
        DictationSessionPhase::Finished => ("Done", "Esc Close"),
        DictationSessionPhase::Failed(_) => ("Dictation failed", "Esc Close"),
        DictationSessionPhase::Idle => ("", ""),
    }
}

/// Recordings shorter than this abort immediately on Escape; longer ones
/// enter a confirmation state where Escape means "continue" and Enter means
/// "stop."  Mirrors the vercel-voice 5-second threshold.
const ESCAPE_CONFIRM_THRESHOLD: Duration = Duration::from_secs(5);

/// Map a dictation session phase + elapsed time to the appropriate Escape behavior.
///
/// Follows the vercel-voice confirm-first pattern:
/// - Recording (< 5 s) → immediate abort
/// - Recording (≥ 5 s) → show confirmation (first Escape)
/// - Confirming → dismiss confirmation and resume recording
pub(crate) fn overlay_escape_action(
    phase: &DictationSessionPhase,
    elapsed: Duration,
) -> OverlayEscapeAction {
    match phase {
        DictationSessionPhase::Recording if elapsed < ESCAPE_CONFIRM_THRESHOLD => {
            OverlayEscapeAction::AbortSession
        }
        DictationSessionPhase::Recording => OverlayEscapeAction::TransitionToConfirming,
        DictationSessionPhase::Confirming => OverlayEscapeAction::ResumeRecording,
        DictationSessionPhase::Transcribing
        | DictationSessionPhase::Delivering
        | DictationSessionPhase::Finished
        | DictationSessionPhase::Failed(_) => OverlayEscapeAction::CloseOverlay,
        DictationSessionPhase::Idle => OverlayEscapeAction::Propagate,
    }
}

/// Return the overlay height for a given session phase.
///
/// All phases use the same pill height — confirming swaps content inline
/// rather than expanding vertically (which would push off the bottom edge).
fn overlay_height_for_phase(_phase: &DictationSessionPhase) -> f32 {
    OVERLAY_HEIGHT_PX
}

/// Resize and reposition the overlay window to match the given phase height.
///
/// The pill stays bottom-anchored: when height grows, the top edge moves up.
/// When height shrinks, the top edge moves down.
#[cfg(target_os = "macos")]
fn resize_overlay_for_phase(window: &mut Window, target_height: f32) {
    if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
            use cocoa::base::id;
            use cocoa::foundation::NSRect;
            use objc::{msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as id;
            // SAFETY: ns_view is from the live overlay window on the main thread.
            // We read the current frame, adjust origin.y to keep the bottom edge
            // fixed, set the new height, and apply via setFrame:display:.
            unsafe {
                let ns_window: id = msg_send![ns_view, window];
                if ns_window.is_null() {
                    return;
                }
                let current_frame: NSRect = msg_send![ns_window, frame];
                let old_height = current_frame.size.height;
                let new_height = target_height as f64;
                let delta = new_height - old_height;
                let new_frame = NSRect::new(
                    cocoa::foundation::NSPoint::new(
                        current_frame.origin.x,
                        current_frame.origin.y - delta,
                    ),
                    cocoa::foundation::NSSize::new(current_frame.size.width, new_height),
                );
                let () = msg_send![ns_window, setFrame: new_frame display: cocoa::base::YES];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn resize_overlay_for_phase(_window: &mut Window, _target_height: f32) {}

/// The GPUI entity that renders the compact dictation pill.
pub struct DictationOverlay {
    state: DictationOverlayState,
    display_bars: [f32; WAVEFORM_BAR_COUNT],
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
        let bars = bars_for_level(DictationLevel {
            rms: 0.0,
            peak: 0.0,
        });
        Self {
            state: DictationOverlayState::default(),
            display_bars: bars,
            focus_handle: cx.focus_handle(),
            transcribing_started_at: None,
            reduced_motion: crate::platform::prefers_reduced_motion(),
            _animation_task: None,
        }
    }

    /// Enter the confirming state: update phase, notify.
    ///
    /// No resize needed — confirming swaps content inline at the same pill height.
    fn enter_confirming(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.phase = DictationSessionPhase::Confirming;
        crate::dictation::set_overlay_phase(DictationSessionPhase::Confirming);
        cx.notify();
    }

    /// Dismiss the confirmation state and resume recording.
    fn resume_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.phase = DictationSessionPhase::Recording;
        crate::dictation::set_overlay_phase(DictationSessionPhase::Recording);
        resize_overlay_for_phase(window, OVERLAY_HEIGHT_PX);
        cx.notify();
    }

    /// Abort the dictation session via the registered callback and close
    /// the overlay window directly.
    ///
    /// This must NOT call `close_dictation_overlay()` because we are
    /// already inside `&mut self` (via `handle_key_down` or a mouse
    /// handler).  Calling `handle.update()` on the same entity would be a
    /// reentrant borrow that silently fails, leaving the window alive but
    /// the global slot empty — causing stacked overlay windows on the
    /// next toggle.
    fn abort_overlay_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let callback = OVERLAY_ABORT_CALLBACK.lock().take();
        // Pre-clear the global slot so if the callback calls
        // close_dictation_overlay, the handle is already gone and that
        // call becomes a harmless no-op.
        let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().take();
        if let Some(cb) = callback {
            cb(cx);
        }
        prepare_overlay_window_for_close(window);
        window.remove_window();
        tracing::info!(
            category = "DICTATION",
            "Overlay closed from within entity (abort)"
        );
    }

    /// Close the overlay window directly from within the entity.
    ///
    /// Same reentrant-borrow avoidance as `abort_overlay_session`, but
    /// without invoking the abort callback (used for non-recording phases).
    fn close_overlay_from_within(&mut self, window: &mut Window) {
        let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().take();
        *OVERLAY_ABORT_CALLBACK.lock() = None;
        prepare_overlay_window_for_close(window);
        window.remove_window();
        tracing::info!(category = "DICTATION", "Overlay closed from within entity");
    }

    /// Replace the visual state snapshot (called from the dictation runtime).
    pub fn set_state(
        &mut self,
        state: DictationOverlayState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let entering_transcribing = state.phase == DictationSessionPhase::Transcribing
            && self.state.phase != DictationSessionPhase::Transcribing;
        let leaving_transcribing = state.phase != DictationSessionPhase::Transcribing
            && self.state.phase == DictationSessionPhase::Transcribing;

        // Resize the overlay when the phase changes height class.
        let old_height = overlay_height_for_phase(&self.state.phase);
        let new_height = overlay_height_for_phase(&state.phase);
        if (new_height - old_height).abs() > f32::EPSILON {
            resize_overlay_for_phase(window, new_height);
        }

        // Smoothly animate bars during recording; snap for other phases.
        self.display_bars =
            if state.phase == DictationSessionPhase::Recording && !self.reduced_motion {
                crate::dictation::visualizer::animate_bars(
                    self.display_bars,
                    state.bars,
                    Duration::from_millis(16),
                )
            } else {
                state.bars
            };

        self.state = state;

        if entering_transcribing && !self.reduced_motion {
            self.transcribing_started_at = Some(Instant::now());
            // Spawn a tick loop that re-renders every TRANSCRIBING_TICK_MS so
            // the sine-wave pulse progresses smoothly.
            self._animation_task = Some(cx.spawn(async move |this, cx| loop {
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
            }));
        } else if leaving_transcribing {
            self.transcribing_started_at = None;
            self._animation_task = None;
        }

        cx.notify();
    }

    /// Handle key-down events for the overlay.
    ///
    /// Escape semantics (vercel-voice 5-second threshold pattern):
    /// - Recording (< 5 s) + Escape → immediate abort
    /// - Recording (≥ 5 s) + Escape → transition to Confirming
    /// - Confirming + Enter → abort the session
    /// - Confirming + Escape → resume Recording
    /// - Confirming + any other key → swallowed (no state change)
    /// - Transcribing / Delivering / Finished / Failed → dismiss overlay only
    /// - Idle → propagate (no-op)
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        tracing::debug!(
            category = "DICTATION",
            key,
            phase = ?self.state.phase,
            "Overlay received key_down"
        );

        // In Confirming state, only Enter and Escape have meaning.
        // All other keys are swallowed without changing state.
        if self.state.phase == DictationSessionPhase::Confirming {
            if crate::ui_foundation::is_key_enter(key) {
                tracing::info!(
                    category = "DICTATION",
                    "Enter pressed during confirmation, aborting dictation session"
                );
                self.abort_overlay_session(window, cx);
                cx.stop_propagation();
                return;
            }
            if !crate::ui_foundation::is_key_escape(key) {
                // Swallow unrelated keys — no resume, no propagation.
                cx.stop_propagation();
                return;
            }
        }

        if !crate::ui_foundation::is_key_escape(key) {
            cx.propagate();
            return;
        }

        // Use the authoritative runtime elapsed time for threshold decisions,
        // falling back to the pump-snapshot elapsed when no session is active.
        let elapsed = crate::dictation::dictation_elapsed().unwrap_or(self.state.elapsed);

        match overlay_escape_action(&self.state.phase, elapsed) {
            OverlayEscapeAction::TransitionToConfirming => {
                tracing::info!(
                    category = "DICTATION",
                    elapsed_ms = elapsed.as_millis() as u64,
                    "Escape pressed after threshold, showing confirmation"
                );
                self.enter_confirming(window, cx);
                cx.stop_propagation();
            }
            OverlayEscapeAction::ResumeRecording => {
                tracing::info!(
                    category = "DICTATION",
                    elapsed_ms = elapsed.as_millis() as u64,
                    "Escape pressed in confirmation, resuming recording"
                );
                self.resume_recording(window, cx);
                cx.stop_propagation();
            }
            OverlayEscapeAction::AbortSession => {
                tracing::info!(
                    category = "DICTATION",
                    elapsed_ms = elapsed.as_millis() as u64,
                    "Escape pressed before threshold, aborting dictation session"
                );
                self.abort_overlay_session(window, cx);
                cx.stop_propagation();
            }
            OverlayEscapeAction::CloseOverlay => {
                tracing::info!(
                    category = "DICTATION",
                    phase = ?self.state.phase,
                    "Escape pressed, dismissing dictation overlay"
                );
                self.close_overlay_from_within(window);
                cx.stop_propagation();
            }
            OverlayEscapeAction::Propagate => {
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
        let bars = &self.display_bars;
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
                // Same 3-column horizontal layout as recording — content swaps
                // inline at the same pill height (no vertical expansion).
                //   left: timer  |  center: Stop · Continue buttons  |  right: spacer
                let timer_text = format_elapsed(*elapsed);
                let stop_color = theme.colors.ui.error.with_opacity(OPACITY_ACTIVE);
                let continue_color = theme.colors.ui.success.with_opacity(OPACITY_ACTIVE);
                let stop_bg = theme.colors.ui.error.with_opacity(0.14);
                let continue_bg = theme.colors.ui.success.with_opacity(0.08);

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    // Left: timer (same position as recording)
                    .child(
                        div().flex().flex_row().items_center().gap(px(8.)).child(
                            div()
                                .text_size(px(STATUS_TEXT_SIZE_PX))
                                .font_family(FONT_MONO)
                                .text_color(timer_color)
                                .child(timer_text),
                        ),
                    )
                    // Center: Stop / Continue buttons (replaces waveform)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .gap(px(10.))
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .bg(stop_bg)
                                    .rounded(px(999.))
                                    .border_1()
                                    .border_color(theme.colors.ui.error.with_opacity(0.45))
                                    .text_size(px(STATUS_TEXT_SIZE_PX))
                                    .font_family(FONT_MONO)
                                    .text_color(stop_color)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                                            this.abort_overlay_session(window, cx);
                                        }),
                                    )
                                    .child(CONFIRM_STOP_LABEL),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .bg(continue_bg)
                                    .rounded(px(999.))
                                    .border_1()
                                    .border_color(theme.colors.ui.success.with_opacity(0.35))
                                    .text_size(px(STATUS_TEXT_SIZE_PX))
                                    .font_family(FONT_MONO)
                                    .text_color(continue_color)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                                            this.resume_recording(window, cx);
                                        }),
                                    )
                                    .child(CONFIRM_CONTINUE_LABEL),
                            ),
                    )
                    // Right: spacer to balance the timer width
                    .child(div().w(px(TIMER_SPACER_WIDTH_PX)))
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

        // Same pill radius for all phases — confirming swaps content inline.
        let radius = OVERLAY_RADIUS_PX;

        // Inner pill surface — glassmorphism styling, padding, rounded corners.
        let surface = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .overflow_hidden()
            .px(px(OVERLAY_HORIZONTAL_PADDING_PX))
            .gap(px(OVERLAY_CONTENT_GAP_PX))
            .bg(surface_bg)
            .rounded(px(radius))
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
            .overflow_hidden()
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

/// Configure the native window surface for the dictation overlay:
/// clear background, corner radius mask, and masksToBounds to prevent
/// white sliver artifacts at the pill's rounded corners.
#[cfg(target_os = "macos")]
fn configure_overlay_window_surface(window: &mut Window) {
    if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
            use cocoa::base::{id, NO, YES};
            use objc::{class, msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as id;
            // SAFETY: ns_view belongs to the live dictation overlay window on
            // the main thread.  We set the window to non-opaque with a clear
            // background and apply a corner mask to the content view's layer
            // so the pill shape clips cleanly.
            unsafe {
                let ns_window: id = msg_send![ns_view, window];
                if ns_window.is_null() {
                    return;
                }
                let clear: id = msg_send![class!(NSColor), clearColor];
                let () = msg_send![ns_window, setOpaque: NO];
                let () = msg_send![ns_window, setBackgroundColor: clear];

                let content_view: id = msg_send![ns_window, contentView];
                if content_view.is_null() {
                    return;
                }
                let () = msg_send![content_view, setWantsLayer: YES];
                let layer: id = msg_send![content_view, layer];
                if layer.is_null() {
                    return;
                }
                let () = msg_send![layer, setCornerRadius: OVERLAY_RADIUS_PX as f64];
                let () = msg_send![layer, setMasksToBounds: YES];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_overlay_window_surface(_window: &mut Window) {}

/// Prepare the overlay window for a clean close: set alpha to 0 so the
/// NSWindow backing store clear doesn't flash white.
#[cfg(target_os = "macos")]
fn prepare_overlay_window_for_close(window: &mut Window) {
    configure_overlay_window_surface(window);
    if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
            use cocoa::base::id;
            use objc::{class, msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as id;
            // SAFETY: ns_view belongs to the live dictation overlay window on
            // the main thread.  We set alpha to 0 before remove_window so the
            // backing store clear is invisible.
            unsafe {
                let ns_window: id = msg_send![ns_view, window];
                if ns_window.is_null() {
                    return;
                }
                let clear: id = msg_send![class!(NSColor), clearColor];
                let () = msg_send![ns_window, setBackgroundColor: clear];
                let () = msg_send![ns_window, setAlphaValue: 0.0_f64];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn prepare_overlay_window_for_close(_window: &mut Window) {}

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
            let alive = handle.update(cx, |_view, _window, _cx| {}).is_ok();
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

    // Snapshot main-window visibility BEFORE any native window operations.
    // If it was hidden, we must ensure it stays hidden after creating the
    // overlay — macOS may surface sibling panels at the same level.
    let main_was_visible = crate::is_main_window_visible();

    // focus: false + show: false — the overlay must not activate the app or
    // surface the main window.  Creating a PopUp with show:true causes macOS
    // to surface sibling panels at the same window level, which briefly
    // flashes the main window.  We create hidden, then bring to front via
    // orderFrontRegardless below (which only surfaces THIS window).
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false,
        show: false,
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

            // Apply clear background + corner mask after vibrancy.
            configure_overlay_window_surface(window);
        });
    }

    // Always make the overlay key window so it receives Escape/Enter key
    // events in both visible-app and hidden-app dictation flows.  The
    // overlay is a NonactivatingPanel (PopUp) and the app uses ACCESSORY
    // activation policy, so makeKeyWindow does not surface the main window
    // or activate the app.  When main is hidden, orderOut: is sent first
    // (below) to prevent sibling-panel flash.
    let should_key_overlay = true;

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_view, window, _cx| {
            if let Ok(wh) = raw_window_handle::HasWindowHandle::window_handle(window) {
                if let raw_window_handle::RawWindowHandle::AppKit(appkit) = wh.as_raw() {
                    use objc::{msg_send, sel, sel_impl};
                    let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
                    // SAFETY: ns_view is a valid NSView from a just-created GPUI window.
                    // orderFrontRegardless brings the window to front without activating the app.
                    // makeKeyWindow is always sent so the overlay receives Escape/Enter key
                    // events in both visible-app and hidden-app dictation flows.
                    unsafe {
                        let ns_window: cocoa::base::id = msg_send![ns_view, window];

                        // Hide the main window BEFORE surfacing the overlay.
                        // orderFrontRegardless on a PopUp can cause macOS to
                        // also surface sibling panels at the same level.  By
                        // sending orderOut: to the main panel first (in the
                        // same synchronous call), it cannot flash on screen.
                        // SAFETY: get_main_window returns the registered main
                        // NSPanel; orderOut: with nil sender is safe.
                        if !main_was_visible {
                            if let Some(main_window) = crate::window_manager::get_main_window() {
                                let () = msg_send![main_window, orderOut: cocoa::base::nil];
                            }
                        }

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

    let _ = handle.update(cx, |view, window, cx| {
        view.set_state(state, window, cx);
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
        // Fade to transparent before removing so the backing store clear
        // doesn't flash white.
        let result = handle.update(cx, |_view, window, _cx| {
            prepare_overlay_window_for_close(window);
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
