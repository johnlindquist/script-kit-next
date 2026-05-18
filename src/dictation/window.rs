use gpui::SharedString;
use std::time::Duration;

use crate::dictation::types::DictationSessionPhase;
use crate::dictation::visualizer::silent_bars;

// ---------------------------------------------------------------------------
// Overlay geometry & waveform contract constants
// ---------------------------------------------------------------------------

/// Glass bar width in pixels.
pub(crate) const OVERLAY_WIDTH_PX: f32 = 520.0;
/// Glass bar height in pixels.
pub(crate) const OVERLAY_HEIGHT_PX: f32 = 72.0;
/// Confirming phase uses the same bar height so content swaps inline.
/// Rounded corner radius for the standalone glass bar.
pub(crate) const OVERLAY_RADIUS_PX: f32 = 22.0;
/// Horizontal padding inside the glass bar.
pub(crate) const OVERLAY_HORIZONTAL_PADDING_PX: f32 = 11.0;
/// Gap between inner content columns.
pub(crate) const OVERLAY_CONTENT_GAP_PX: f32 = 8.0;
/// Font size for timer, status, and transcript text.
pub(crate) const STATUS_TEXT_SIZE_PX: f32 = 11.5;
/// Right-side spacer width to balance the timer column.
pub(crate) const TIMER_SPACER_WIDTH_PX: f32 = 32.0;
/// Width of the right-hand target badge slot (replaces spacer when target is shown).
pub(crate) const TARGET_BADGE_SLOT_WIDTH_PX: f32 = 108.0;

/// Number of waveform bars in the compact capsule visualizer.
pub(crate) const WAVEFORM_BAR_COUNT: usize = 9;
/// Width of each waveform bar in pixels.
pub(crate) const WAVEFORM_BAR_WIDTH_PX: f32 = 3.0;
/// Gap between waveform bars in pixels.
pub(crate) const WAVEFORM_BAR_GAP_PX: f32 = 3.0;
/// Minimum waveform bar height (silent level).
pub(crate) const WAVEFORM_BAR_MIN_HEIGHT_PX: f32 = 4.0;
/// Maximum waveform bar height (peak level).
pub(crate) const WAVEFORM_BAR_MAX_HEIGHT_PX: f32 = 18.0;

/// Number of transcribing-state dots.
pub(crate) const TRANSCRIBING_DOT_COUNT: usize = 3;
/// Diameter of each transcribing dot.
pub(crate) const TRANSCRIBING_DOT_SIZE_PX: f32 = 4.0;
/// Gap between transcribing dots.
pub(crate) const TRANSCRIBING_DOT_GAP_PX: f32 = 4.0;

/// Threshold: if any bar exceeds this, we treat audio as "active" (green).
const SOUND_THRESHOLD: f32 = 0.10;

/// Bottom offset from the screen edge for dock clearance.
const OVERLAY_BOTTOM_OFFSET_PX: f32 = 15.0;

// ---------------------------------------------------------------------------
// Glass bar surface constants
// ---------------------------------------------------------------------------

/// Subtle inner border opacity for the selected signal band.
const GLASS_SIGNAL_BORDER_OPACITY: f32 = 0.10;
/// Neutral rim opacity, similar to the main menu window edge.
const GLASS_BAR_RIM_OPACITY: f32 = 0.16;
/// Glass bar shadow opacity.
const GLASS_BAR_SHADOW_OPACITY: f32 = 0.22;

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
/// Compact capsule curve: `min + pow(v, 0.7) * (max - min)`.
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

/// Build the overlay badge label for the current delivery target.
///
/// External-app dictation names the tracked frontmost app, matching the
/// clipboard flow's "Paste to <app>" hint while keeping internal targets
/// explicit.
pub(crate) fn target_badge_label(target: crate::dictation::DictationTarget) -> SharedString {
    if matches!(target, crate::dictation::DictationTarget::ExternalApp) {
        if let Some(name) = crate::frontmost_app_tracker::get_last_real_app()
            .map(|app| app.name.trim().to_string())
            .filter(|name| !name.is_empty())
        {
            return name.into();
        }
    }

    target.overlay_label().into()
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
    pub target: crate::dictation::DictationTarget,
}

impl Default for DictationOverlayState {
    fn default() -> Self {
        Self {
            phase: DictationSessionPhase::Idle,
            elapsed: Duration::ZERO,
            bars: silent_bars(),
            transcript: SharedString::default(),
            target: crate::dictation::DictationTarget::ExternalApp,
        }
    }
}

// ---------------------------------------------------------------------------
// GPUI overlay entity + window lifecycle
// ---------------------------------------------------------------------------

use gpui::{
    div, prelude::*, px, rgba, AnyElement, App, Context, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, ParentElement, Render, StatefulInteractiveElement,
    Styled, Task, Window, WindowBounds, WindowOptions,
};

use crate::list_item::FONT_SYSTEM_UI;
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED, OPACITY_SUBTLE, OPACITY_TEXT_MUTED};
use crate::theme::{get_cached_theme, AppChromeColors};
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
/// Callback type for overlay submit actions (stop and transcribe dictation).
type OverlaySubmitCallback = Box<dyn Fn(&mut App) + Send + Sync + 'static>;

/// Global abort callback set by the dictation runtime.
static OVERLAY_ABORT_CALLBACK: Mutex<Option<OverlayAbortCallback>> = Mutex::new(None);
/// Global submit callback set by the dictation runtime.
static OVERLAY_SUBMIT_CALLBACK: Mutex<Option<OverlaySubmitCallback>> = Mutex::new(None);

/// Register a callback to be invoked when the user confirms stop via
/// Enter or the Stop button in the overlay.
pub fn set_overlay_abort_callback(callback: impl Fn(&mut App) + Send + Sync + 'static) {
    *OVERLAY_ABORT_CALLBACK.lock() = Some(Box::new(callback));
}

/// Register a callback to be invoked when the user clicks Stop on the
/// recording overlay.
pub fn set_overlay_submit_callback(callback: impl Fn(&mut App) + Send + Sync + 'static) {
    *OVERLAY_SUBMIT_CALLBACK.lock() = Some(Box::new(callback));
}

// ---------------------------------------------------------------------------
// Global escape monitor (fires even when overlay is not focused)
// ---------------------------------------------------------------------------

/// Wrapper for NSEvent monitor ID to make it Send+Sync.
///
/// The monitor ID is created and removed on the main thread; the static only
/// provides cross-thread visibility for the cleanup path.
#[cfg(target_os = "macos")]
struct SendableId(cocoa::base::id);
// SAFETY: The NSEvent monitor ID is created on the main thread and only
// accessed behind a Mutex. The cleanup call (removeMonitor:) also runs on
// the main thread. The raw pointer never escapes to a non-main thread.
#[cfg(target_os = "macos")]
unsafe impl Send for SendableId {}
#[cfg(target_os = "macos")]
unsafe impl Sync for SendableId {}

/// Holds the NSEvent global monitor ID so we can remove it on close.
#[cfg(target_os = "macos")]
static GLOBAL_ESCAPE_MONITOR: Mutex<Option<SendableId>> = Mutex::new(None);

/// Install a global key-down monitor that catches Escape pressed in any app.
///
/// `NSEvent addGlobalMonitorForEventsMatchingMask:handler:` fires for events
/// delivered to OTHER applications — our own window receives `KeyDownEvent`
/// via GPUI's normal path, so there's no double-fire.
///
/// The monitor sets `ESCAPE_REQUESTED` to `true`; the overlay pump (16ms tick)
/// picks it up in GPUI context where it can safely mutate state.
#[cfg(target_os = "macos")]
fn install_global_escape_monitor() {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("install_global_escape_monitor") {
        return;
    }

    // Already installed — don't double-register.
    if GLOBAL_ESCAPE_MONITOR.lock().is_some() {
        return;
    }

    // NSEventMaskKeyDown = 1 << 10
    let mask: u64 = 1 << 10;

    let block = block::ConcreteBlock::new(move |event: id| {
        // SAFETY: `event` is a valid NSEvent passed by AppKit.
        // keyCode 53 = Escape, keyCode 36 = Return/Enter.
        let key_code: u16 = unsafe { msg_send![event, keyCode] };
        match key_code {
            53 => {
                tracing::info!(
                    category = "DICTATION",
                    "Global key monitor: Escape pressed in external app"
                );
                ESCAPE_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            36 => {
                tracing::info!(
                    category = "DICTATION",
                    "Global key monitor: Enter pressed in external app"
                );
                ENTER_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            _ => {}
        }
    });
    let block = block.copy();

    // SAFETY: NSEvent is a valid AppKit class on macOS.
    // addGlobalMonitorForEventsMatchingMask:handler: is called on the main
    // thread (open_dictation_overlay runs on main). The returned monitor ID
    // is stored in GLOBAL_ESCAPE_MONITOR for cleanup.
    let monitor: id = unsafe {
        let ns_event_class = class!(NSEvent);
        msg_send![
            ns_event_class,
            addGlobalMonitorForEventsMatchingMask: mask
            handler: &*block
        ]
    };

    if monitor != nil {
        *GLOBAL_ESCAPE_MONITOR.lock() = Some(SendableId(monitor));
        tracing::debug!(category = "DICTATION", "Global escape monitor installed");
    } else {
        tracing::warn!(
            category = "DICTATION",
            "Failed to install global escape monitor"
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn install_global_escape_monitor() {}

/// Remove the global key-down monitor.
#[cfg(target_os = "macos")]
fn remove_global_escape_monitor() {
    use objc::{class, msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("remove_global_escape_monitor") {
        return;
    }

    let monitor = GLOBAL_ESCAPE_MONITOR.lock().take();
    if let Some(SendableId(monitor)) = monitor {
        // SAFETY: monitor is a valid id returned by
        // addGlobalMonitorForEventsMatchingMask:handler:.
        // removeMonitor: is the correct cleanup API.
        unsafe {
            let _: () = msg_send![class!(NSEvent), removeMonitor: monitor];
        }
        tracing::debug!(category = "DICTATION", "Global escape monitor removed");
    }
}

#[cfg(not(target_os = "macos"))]
fn remove_global_escape_monitor() {}

/// Flag: the global key monitor detected an Escape press that the overlay
/// needs to process. Checked by `process_global_keys_if_requested` inside
/// GPUI context on every pump tick.
static ESCAPE_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Flag: the global key monitor detected an Enter press while in Confirming
/// phase. Enter in Confirming = abort the session.
static ENTER_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Confirming-phase copy constants (single source of truth)
// ---------------------------------------------------------------------------

/// Single-word action label for stopping/submitting the current recording.
const ACTION_STOP_LABEL: &str = "Stop";
/// Changes the preference used by the next capture; the live session keeps its opened mic.
const ACTION_MIC_LABEL: &str = "Next Mic";
/// Single-word action label for discarding the current recording.
const ACTION_CANCEL_LABEL: &str = "Cancel";
/// Single-word action label for resuming from confirmation.
const ACTION_CONTINUE_LABEL: &str = "Continue";
/// Single-word action label for closing terminal overlay states.
const ACTION_CLOSE_LABEL: &str = "Close";
/// Keycap shown for Escape.
const ESC_KEYCAP: &str = "esc";
/// Keycap shown for Enter.
const ENTER_KEYCAP: &str = "\u{21b5}";

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

/// Return phase-appropriate (headline, action label) copy for the dictation overlay.
///
/// The headline is the primary status text (e.g. "Listening…", "Stop dictation?").
/// The action label names the visible compact action chip for each phase.
pub(crate) fn overlay_phase_copy(phase: &DictationSessionPhase) -> (&'static str, &'static str) {
    match phase {
        DictationSessionPhase::Recording => ("Listening\u{2026}", ACTION_CANCEL_LABEL),
        DictationSessionPhase::Confirming => ("Stop dictation?", ACTION_CONTINUE_LABEL),
        DictationSessionPhase::Transcribing => ("Transcribing\u{2026}", ACTION_CLOSE_LABEL),
        DictationSessionPhase::Delivering => ("Delivering\u{2026}", ACTION_CLOSE_LABEL),
        DictationSessionPhase::Finished => ("Done", ACTION_CLOSE_LABEL),
        DictationSessionPhase::Failed(_) => ("Dictation failed", ACTION_CLOSE_LABEL),
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
        Self {
            state: DictationOverlayState::default(),
            display_bars: silent_bars(),
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

    /// Advance the live dictation session to its next configured destination.
    fn cycle_target(&mut self, cx: &mut Context<Self>) {
        if self.state.phase != DictationSessionPhase::Recording {
            return;
        }

        let Some(next_target) = crate::dictation::cycle_dictation_target() else {
            return;
        };

        self.state.target = next_target;
        cx.notify();
    }

    /// Cycle the configured dictation microphone through the shared picker items.
    ///
    /// The current recording keeps using the device it opened with; this updates
    /// the persisted preference used by the next dictation capture.
    fn cycle_microphone(&mut self, cx: &mut Context<Self>) {
        let prefs = crate::config::load_user_preferences();
        let selected_device_id = prefs.dictation.selected_device_id.as_deref();
        let menu_items = match crate::dictation::list_input_device_menu_items(selected_device_id) {
            Ok(items) if !items.is_empty() => items,
            Ok(_) => {
                tracing::warn!(
                    category = "DICTATION",
                    "Microphone selector found no input devices"
                );
                return;
            }
            Err(error) => {
                tracing::warn!(
                    category = "DICTATION",
                    error = %error,
                    "Failed to list microphones from overlay selector"
                );
                return;
            }
        };

        let selected_index = menu_items
            .iter()
            .position(|item| item.is_selected)
            .unwrap_or(0);
        let next_index = (selected_index + 1) % menu_items.len();
        let next_item = &menu_items[next_index];
        if let Err(error) = crate::dictation::apply_device_selection(&next_item.action) {
            tracing::warn!(
                category = "DICTATION",
                error = %error,
                "Failed to persist microphone selection from overlay"
            );
            return;
        }

        tracing::info!(
            category = "DICTATION",
            microphone = %next_item.title,
            "Overlay microphone selector updated preference for next recording"
        );
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
        *OVERLAY_SUBMIT_CALLBACK.lock() = None;
        // Pre-clear the global slot so if the callback calls
        // close_dictation_overlay, the handle is already gone and that
        // call becomes a harmless no-op.
        let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().take();
        remove_global_escape_monitor();
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

    /// Submit the active recording via the registered callback.
    ///
    /// Closing happens before callback dispatch so the app-owned stop path can
    /// reopen/update the overlay into its transcribing state without reentrant
    /// updates to this entity.
    fn submit_overlay_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let callback = OVERLAY_SUBMIT_CALLBACK.lock().take();
        *OVERLAY_ABORT_CALLBACK.lock() = None;
        let slot = DICTATION_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
        slot.lock().take();
        remove_global_escape_monitor();
        prepare_overlay_window_for_close(window);
        window.remove_window();

        if let Some(cb) = callback {
            cx.defer(move |cx| {
                cb(cx);
            });
        }

        tracing::info!(
            category = "DICTATION",
            "Overlay closed from within entity (submit)"
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
        *OVERLAY_SUBMIT_CALLBACK.lock() = None;
        remove_global_escape_monitor();
        prepare_overlay_window_for_close(window);
        window.remove_window();
        tracing::info!(category = "DICTATION", "Overlay closed from within entity");
    }

    /// Check whether the global key monitor flagged an Escape or Enter press
    /// and process it.  Called from the overlay pump tick (every 16ms) so the
    /// action runs inside GPUI context with full `&mut self` access.
    pub fn process_global_keys_if_requested(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Enter is only meaningful in the Confirming phase (= abort session).
        let enter = ENTER_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst);
        if enter && self.state.phase == DictationSessionPhase::Confirming {
            tracing::info!(
                category = "DICTATION",
                "Processing global Enter request in Confirming phase — aborting"
            );
            self.abort_overlay_session(window, cx);
            // Clear any pending escape too — we've already acted.
            ESCAPE_REQUESTED.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        }

        if !ESCAPE_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        let elapsed = crate::dictation::dictation_elapsed().unwrap_or(self.state.elapsed);
        let action = overlay_escape_action(&self.state.phase, elapsed);

        tracing::info!(
            category = "DICTATION",
            ?action,
            phase = ?self.state.phase,
            elapsed_ms = elapsed.as_millis() as u64,
            "Processing global escape request"
        );

        match action {
            OverlayEscapeAction::TransitionToConfirming => {
                self.enter_confirming(window, cx);
            }
            OverlayEscapeAction::ResumeRecording => {
                self.resume_recording(window, cx);
            }
            OverlayEscapeAction::AbortSession => {
                self.abort_overlay_session(window, cx);
            }
            OverlayEscapeAction::CloseOverlay => {
                self.close_overlay_from_within(window);
            }
            OverlayEscapeAction::Propagate => {}
        }
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

    /// Render the right-side target badge, matching footer-button mouse
    /// affordances when cycling is available.
    fn render_target_badge_slot(
        &self,
        interactive: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = get_cached_theme();
        let hover_bg = theme.colors.background.main.with_opacity(OPACITY_SELECTED);
        let active_bg = theme.colors.background.main.with_opacity(OPACITY_ACTIVE);

        let mut label = div()
            .text_size(px(STATUS_TEXT_SIZE_PX - 1.0))
            .font_family(FONT_SYSTEM_UI)
            .text_color(theme.colors.text.primary.with_opacity(OPACITY_ACTIVE))
            .max_w(px(TARGET_BADGE_SLOT_WIDTH_PX - 18.0))
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(target_badge_label(self.state.target));
        if interactive {
            label = label.cursor_pointer();
        }

        let mut badge = div()
            .id("dictation-target-badge")
            .px(px(8.))
            .py(px(2.))
            .rounded(px(999.))
            .bg(theme.colors.background.main.with_opacity(OPACITY_SUBTLE))
            .border_1()
            .border_color(theme.colors.ui.border.with_opacity(OPACITY_SUBTLE))
            .cursor_default()
            .child(label);

        if interactive {
            badge = badge
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .active(move |style| style.bg(active_bg))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                        this.cycle_target(cx);
                    }),
                );
        }

        div()
            .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .child(badge)
    }

    /// Render the runtime recording action rail.
    fn render_recording_actions(&self, cx: &mut Context<Self>) -> AnyElement {
        render_clickable_action_rail([
            render_clickable_action_chip(
                "dictation-stop-button",
                ACTION_STOP_LABEL.into(),
                dictation_stop_keycap(),
                cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                    this.submit_overlay_session(window, cx);
                }),
            )
            .into_any_element(),
            render_clickable_action_chip(
                "dictation-mic-button",
                ACTION_MIC_LABEL.into(),
                current_microphone_keycap(),
                cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.cycle_microphone(cx);
                }),
            )
            .into_any_element(),
            render_clickable_action_chip(
                "dictation-cancel-button",
                ACTION_CANCEL_LABEL.into(),
                ESC_KEYCAP.into(),
                cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                    this.abort_overlay_session(window, cx);
                }),
            )
            .into_any_element(),
        ])
    }

    /// Render the runtime confirmation action rail.
    fn render_confirming_actions(&self, cx: &mut Context<Self>) -> AnyElement {
        render_clickable_action_rail([
            render_clickable_action_chip(
                "dictation-stop-button",
                ACTION_STOP_LABEL.into(),
                ENTER_KEYCAP.into(),
                cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                    this.abort_overlay_session(window, cx);
                }),
            )
            .into_any_element(),
            render_clickable_action_chip(
                "dictation-continue-button",
                ACTION_CONTINUE_LABEL.into(),
                ESC_KEYCAP.into(),
                cx.listener(|this, _event: &MouseDownEvent, window, cx| {
                    this.resume_recording(window, cx);
                }),
            )
            .into_any_element(),
        ])
    }

    /// Render a compact Close action for terminal phases.
    fn render_close_action(&self, cx: &mut Context<Self>) -> AnyElement {
        render_clickable_action_rail([render_clickable_action_chip(
            "dictation-close-button",
            ACTION_CLOSE_LABEL.into(),
            ESC_KEYCAP.into(),
            cx.listener(|this, _event: &MouseDownEvent, window, _cx| {
                this.close_overlay_from_within(window);
            }),
        )
        .into_any_element()])
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
        let chrome = AppChromeColors::from_theme(&theme);

        // Glass bar surface: same theme-backed window surface and neutral rim
        // language as the launcher/main menu chrome.
        let surface_bg = rgba(chrome.window_surface_rgba);
        let border_color = theme.colors.ui.border.with_opacity(GLASS_BAR_RIM_OPACITY);

        let timer_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);
        let muted_text = theme.colors.text.muted.with_opacity(OPACITY_TEXT_MUTED);
        let text_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);

        let phase = &self.state.phase;
        let bars = &self.display_bars;
        let elapsed = &self.state.elapsed;

        // Primary controls plus a visible shortcut rail. The rail is part of
        // the runtime overlay, not hidden copy, so dictation affordances stay
        // discoverable while the glass bar stays visually aligned to the launcher.
        let inner = match phase {
            DictationSessionPhase::Recording => {
                let timer_text = format_elapsed(*elapsed);
                let active = has_sound(bars);
                let target_badge_interactive = crate::dictation::can_cycle_dictation_target();

                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(3.))
                    .w_full()
                    .child(render_glass_signal_band(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            // Left: timer
                            .child(
                                div()
                                    .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(STATUS_TEXT_SIZE_PX))
                                            .font_family(FONT_SYSTEM_UI)
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
                            // Right: destination badge
                            .child(self.render_target_badge_slot(target_badge_interactive, cx))
                            .into_any_element(),
                    ))
                    .child(wrap_dictation_overlay_action_rail(
                        self.render_recording_actions(cx),
                        surface_bg,
                    ))
            }
            DictationSessionPhase::Confirming => {
                let timer_text = format_elapsed(*elapsed);
                let (headline, _) = overlay_phase_copy(phase);

                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(3.))
                    .w_full()
                    .child(render_glass_signal_band(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            // Left: timer (same position as recording)
                            .child(
                                div()
                                    .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(STATUS_TEXT_SIZE_PX))
                                            .font_family(FONT_SYSTEM_UI)
                                            .text_color(timer_color)
                                            .child(timer_text),
                                    ),
                            )
                            // Center: confirmation headline
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(STATUS_TEXT_SIZE_PX))
                                    .font_family(FONT_SYSTEM_UI)
                                    .text_color(text_color)
                                    .child(headline),
                            )
                            // Right: destination badge
                            .child(self.render_target_badge_slot(false, cx))
                            .into_any_element(),
                    ))
                    .child(wrap_dictation_overlay_action_rail(
                        self.render_confirming_actions(cx),
                        surface_bg,
                    ))
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
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .w_full()
                    .child(render_glass_signal_band(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(render_transcribing_dots(&dot_opacities))
                            .into_any_element(),
                    ))
                    .child(wrap_dictation_overlay_action_rail(
                        self.render_close_action(cx),
                        surface_bg,
                    ))
            }
            DictationSessionPhase::Delivering => div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .w_full()
                .child(render_glass_signal_band(
                    div()
                        .text_size(px(STATUS_TEXT_SIZE_PX))
                        .font_family(FONT_SYSTEM_UI)
                        .text_color(text_color)
                        .overflow_hidden()
                        .child("Delivering\u{2026}")
                        .into_any_element(),
                ))
                .child(wrap_dictation_overlay_action_rail(
                    self.render_close_action(cx),
                    surface_bg,
                )),
            DictationSessionPhase::Finished => div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .w_full()
                .child(render_glass_signal_band(
                    div()
                        .text_size(px(STATUS_TEXT_SIZE_PX))
                        .font_family(FONT_SYSTEM_UI)
                        .text_color(text_color)
                        .overflow_hidden()
                        .child(finished_label())
                        .into_any_element(),
                ))
                .child(wrap_dictation_overlay_action_rail(
                    self.render_close_action(cx),
                    surface_bg,
                )),
            DictationSessionPhase::Failed(ref msg) => {
                let err_text: SharedString = format!("Error: {msg}").into();
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(3.))
                    .w_full()
                    .child(render_glass_signal_band(
                        div()
                            .text_size(px(STATUS_TEXT_SIZE_PX))
                            .font_family(FONT_SYSTEM_UI)
                            .text_color(muted_text)
                            .overflow_hidden()
                            .child(err_text)
                            .into_any_element(),
                    ))
                    .child(wrap_dictation_overlay_action_rail(
                        self.render_close_action(cx),
                        surface_bg,
                    ))
            }
            DictationSessionPhase::Idle => div(),
        };

        // Same capsule radius for all phases; confirming swaps content inline.
        let radius = OVERLAY_RADIUS_PX;

        // Capsule chrome only; the signal band uses the main-menu selected-row fill
        // and the action rail carries the launcher window surface underneath.
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
            .rounded(px(radius))
            .border_1()
            .border_color(border_color)
            .shadow(vec![gpui::BoxShadow {
                color: theme
                    .colors
                    .ui
                    .border
                    .with_opacity(GLASS_BAR_SHADOW_OPACITY),
                offset: gpui::point(px(0.0), px(8.0)),
                blur_radius: px(20.0),
                spread_radius: px(0.0),
            }])
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

/// Format the finished overlay state label.
pub(crate) fn finished_label() -> SharedString {
    "Done".into()
}

/// Render waveform bars for the glass bar.
///
/// Uses theme success color when sound is detected, muted text when silent.
fn render_waveform_bars(bars: &[f32; WAVEFORM_BAR_COUNT], active: bool) -> impl IntoElement {
    let theme = get_cached_theme();
    let bar_hex = if active {
        theme.colors.ui.success
    } else {
        theme.colors.text.primary
    };
    let inactive_opacity_scale = if active {
        1.0
    } else {
        theme.get_opacity().text_hint
    };

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(WAVEFORM_BAR_GAP_PX))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX));

    for &level in bars {
        let bar_color = bar_hex.with_opacity(waveform_bar_opacity(level) * inactive_opacity_scale);
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

fn dictation_stop_keycap() -> SharedString {
    crate::config::load_config()
        .get_dictation_hotkey()
        .map(|hotkey| dictation_hotkey_keycap(&hotkey))
        .filter(|key| !key.trim().is_empty())
        .unwrap_or_else(|| "click".to_string())
        .into()
}

fn dictation_hotkey_keycap(hotkey: &crate::config::HotkeyConfig) -> String {
    hotkey.to_display_string().replace("Semicolon", ";")
}

fn current_microphone_keycap() -> SharedString {
    let prefs = crate::config::load_user_preferences();
    let selected_device_id = prefs.dictation.selected_device_id.as_deref();
    let label = crate::dictation::list_input_device_menu_items(selected_device_id)
        .ok()
        .and_then(|items| items.into_iter().find(|item| item.is_selected))
        .map(|item| microphone_keycap_label(&item.title))
        .unwrap_or_else(|| "mic".to_string());
    label.into()
}

fn microphone_keycap_label(title: &str) -> String {
    let title = title
        .replace(" \u{00b7} default", "")
        .replace(" (current)", "");
    let title = title.trim();
    if title.eq_ignore_ascii_case("system default") {
        return "default".to_string();
    }

    const MAX_CHARS: usize = 8;
    let mut chars = title.chars();
    let mut compact = String::new();
    for _ in 0..MAX_CHARS {
        if let Some(ch) = chars.next() {
            compact.push(ch);
        } else {
            break;
        }
    }
    if chars.next().is_some() {
        compact.push('…');
    }
    if compact.is_empty() {
        "mic".to_string()
    } else {
        compact
    }
}

fn action_chip_width(label: &str) -> f32 {
    match label {
        ACTION_CONTINUE_LABEL => 112.0,
        ACTION_MIC_LABEL => 152.0,
        ACTION_CLOSE_LABEL => 72.0,
        _ => 96.0,
    }
}

fn render_glass_signal_band(body: AnyElement) -> impl IntoElement {
    let theme = get_cached_theme();
    let list_item_colors = crate::list_item::ListItemColors::from_theme(&theme);
    let selected_row_bg = crate::list_item::row_selected_background_rgba(&list_item_colors);

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.0))
        .py(px(6.0))
        .bg(rgba(selected_row_bg))
        .border_1()
        .border_color(
            theme
                .colors
                .ui
                .border
                .with_opacity(GLASS_SIGNAL_BORDER_OPACITY),
        )
        .rounded(px(9.0))
        .child(body)
}

fn render_action_chip_content(label: SharedString, key: SharedString) -> impl IntoElement {
    let theme = get_cached_theme();
    let footer_text = theme
        .colors
        .text
        .primary
        .with_opacity(crate::window_resize::mini_layout::HINT_TEXT_OPACITY)
        .to_rgb();
    let shortcut_colors = crate::components::hint_strip::whisper_inline_shortcut_colors(
        footer_text.into(),
        theme.colors.text.primary.to_rgb(),
        false,
    );
    let key = key.to_string();
    let shortcut_tokens = crate::components::hint_strip::shortcut_tokens_from_hint(&key);

    div()
        .px(px(4.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.0))
        .child(
            div()
                .text_size(px(12.5))
                .text_color(footer_text)
                .child(label),
        )
        .child(crate::components::hint_strip::render_inline_shortcut_keys(
            shortcut_tokens.iter().map(|token| token.as_str()),
            shortcut_colors,
        ))
}

fn render_action_chip(label: &'static str, key: SharedString) -> impl IntoElement {
    div()
        .w(px(action_chip_width(label)))
        .h_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .child(render_action_chip_content(label.into(), key))
}

fn render_clickable_action_chip(
    id: &'static str,
    label: SharedString,
    key: SharedString,
    listener: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let hover_bg = rgba(chrome.hover_rgba);
    let active_bg = rgba(chrome.selection_rgba);
    let width = action_chip_width(label.as_ref());

    div()
        .id(id)
        .w(px(width))
        .h_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg))
        .active(move |style| style.bg(active_bg))
        .on_mouse_down(MouseButton::Left, listener)
        .child(render_action_chip_content(label, key))
}

/// Paint the bottom action rail on the launcher window surface without stacking
/// that fill under the selected-row signal band above it.
fn wrap_dictation_overlay_action_rail(
    rail: impl IntoElement,
    surface_bg: gpui::Rgba,
) -> impl IntoElement {
    div().w_full().pt(px(3.0)).bg(surface_bg).child(rail)
}

fn render_clickable_action_rail(actions: impl IntoIterator<Item = AnyElement>) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    let mut rail = div()
        .id("dictation-action-rail")
        .w_full()
        .min_h(px(24.0))
        .border_t_1()
        .border_color(rgba(chrome.divider_rgba))
        .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
        .pt(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(12.0));

    for action in actions {
        rail = rail.child(action);
    }

    rail.into_any_element()
}

fn render_static_action_rail(
    actions: impl IntoIterator<Item = (&'static str, SharedString)>,
) -> impl IntoElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    let mut rail = div()
        .id("dictation-action-rail")
        .w_full()
        .min_h(px(24.0))
        .border_t_1()
        .border_color(rgba(chrome.divider_rgba))
        .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
        .pt(px(4.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(12.0));

    for (label, key) in actions {
        rail = rail.child(render_action_chip(label, key));
    }

    rail
}

/// Render the live glass bar from a fixed state for Storybook previews.
///
/// This keeps canonical Dictation stories on the same constants, waveform
/// math, phase copy, and target-badge styling as the runtime overlay without
/// opening a real floating window.
pub(crate) fn render_dictation_overlay_state_preview(
    state: &DictationOverlayState,
) -> gpui::AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let surface_bg = rgba(chrome.window_surface_rgba);
    let border_color = theme.colors.ui.border.with_opacity(GLASS_BAR_RIM_OPACITY);
    let timer_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);
    let muted_text = theme.colors.text.muted.with_opacity(OPACITY_TEXT_MUTED);
    let text_color = theme.colors.text.primary.with_opacity(OPACITY_ACTIVE);

    if matches!(state.phase, DictationSessionPhase::Idle) {
        return div()
            .w(px(OVERLAY_WIDTH_PX))
            .h(px(OVERLAY_HEIGHT_PX))
            .into_any_element();
    }

    let inner = match &state.phase {
        DictationSessionPhase::Recording => {
            let timer_text = format_elapsed(state.elapsed);
            let active = has_sound(&state.bars);
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .w_full()
                .child(render_glass_signal_band(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .w_full()
                        .child(
                            div()
                                .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(STATUS_TEXT_SIZE_PX))
                                        .font_family(FONT_SYSTEM_UI)
                                        .text_color(timer_color)
                                        .child(timer_text),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .child(render_waveform_bars(&state.bars, active)),
                        )
                        .child(render_static_target_badge_slot(state.target))
                        .into_any_element(),
                ))
                .child(wrap_dictation_overlay_action_rail(
                    render_static_action_rail([
                        (ACTION_STOP_LABEL, dictation_stop_keycap()),
                        (ACTION_MIC_LABEL, current_microphone_keycap()),
                        (ACTION_CANCEL_LABEL, ESC_KEYCAP.into()),
                    ]),
                    surface_bg,
                ))
        }
        DictationSessionPhase::Confirming => {
            let timer_text = format_elapsed(state.elapsed);
            let (headline, _) = overlay_phase_copy(&state.phase);
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .w_full()
                .child(render_glass_signal_band(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .w_full()
                        .child(
                            div()
                                .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(STATUS_TEXT_SIZE_PX))
                                        .font_family(FONT_SYSTEM_UI)
                                        .text_color(timer_color)
                                        .child(timer_text),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .text_size(px(STATUS_TEXT_SIZE_PX))
                                .font_family(FONT_SYSTEM_UI)
                                .text_color(text_color)
                                .child(headline),
                        )
                        .child(render_static_target_badge_slot(state.target))
                        .into_any_element(),
                ))
                .child(wrap_dictation_overlay_action_rail(
                    render_static_action_rail([
                        (ACTION_STOP_LABEL, ENTER_KEYCAP.into()),
                        (ACTION_CONTINUE_LABEL, ESC_KEYCAP.into()),
                    ]),
                    surface_bg,
                ))
        }
        DictationSessionPhase::Transcribing => div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(4.))
            .w_full()
            .child(render_glass_signal_band(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(render_transcribing_dots(
                        &transcribing_dot_opacities_static(),
                    ))
                    .into_any_element(),
            ))
            .child(wrap_dictation_overlay_action_rail(
                render_static_action_rail([(ACTION_CLOSE_LABEL, ESC_KEYCAP.into())]),
                surface_bg,
            )),
        DictationSessionPhase::Delivering => div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(3.))
            .w_full()
            .child(render_glass_signal_band(
                div()
                    .text_size(px(STATUS_TEXT_SIZE_PX))
                    .font_family(FONT_SYSTEM_UI)
                    .text_color(text_color)
                    .overflow_hidden()
                    .child("Delivering\u{2026}")
                    .into_any_element(),
            ))
            .child(wrap_dictation_overlay_action_rail(
                render_static_action_rail([(ACTION_CLOSE_LABEL, ESC_KEYCAP.into())]),
                surface_bg,
            )),
        DictationSessionPhase::Finished => div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(3.))
            .w_full()
            .child(render_glass_signal_band(
                div()
                    .text_size(px(STATUS_TEXT_SIZE_PX))
                    .font_family(FONT_SYSTEM_UI)
                    .text_color(text_color)
                    .overflow_hidden()
                    .child(finished_label())
                    .into_any_element(),
            ))
            .child(wrap_dictation_overlay_action_rail(
                render_static_action_rail([(ACTION_CLOSE_LABEL, ESC_KEYCAP.into())]),
                surface_bg,
            )),
        DictationSessionPhase::Failed(msg) => {
            let err_text: SharedString = format!("Error: {msg}").into();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.))
                .w_full()
                .child(render_glass_signal_band(
                    div()
                        .text_size(px(STATUS_TEXT_SIZE_PX))
                        .font_family(FONT_SYSTEM_UI)
                        .text_color(muted_text)
                        .overflow_hidden()
                        .child(err_text)
                        .into_any_element(),
                ))
                .child(wrap_dictation_overlay_action_rail(
                    render_static_action_rail([(ACTION_CLOSE_LABEL, ESC_KEYCAP.into())]),
                    surface_bg,
                ))
        }
        DictationSessionPhase::Idle => div(),
    };

    div()
        .w(px(OVERLAY_WIDTH_PX))
        .h(px(OVERLAY_HEIGHT_PX))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .px(px(OVERLAY_HORIZONTAL_PADDING_PX))
        .gap(px(OVERLAY_CONTENT_GAP_PX))
        .rounded(px(OVERLAY_RADIUS_PX))
        .border_1()
        .border_color(border_color)
        .shadow(vec![gpui::BoxShadow {
            color: theme
                .colors
                .ui
                .border
                .with_opacity(GLASS_BAR_SHADOW_OPACITY),
            offset: gpui::point(px(0.0), px(8.0)),
            blur_radius: px(20.0),
            spread_radius: px(0.0),
        }])
        .child(inner)
        .into_any_element()
}

fn render_static_target_badge_slot(target: crate::dictation::DictationTarget) -> impl IntoElement {
    let theme = get_cached_theme();
    let label = div()
        .text_size(px(STATUS_TEXT_SIZE_PX - 1.0))
        .font_family(FONT_SYSTEM_UI)
        .text_color(theme.colors.text.primary.with_opacity(OPACITY_ACTIVE))
        .max_w(px(TARGET_BADGE_SLOT_WIDTH_PX - 18.0))
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .child(target_badge_label(target));

    let badge = div()
        .id("dictation-target-badge")
        .px(px(8.))
        .py(px(2.))
        .rounded(px(999.))
        .bg(theme.colors.background.main.with_opacity(OPACITY_SUBTLE))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(OPACITY_SUBTLE))
        .cursor_default()
        .child(label);

    div()
        .w(px(TARGET_BADGE_SLOT_WIDTH_PX))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .child(badge)
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

    ESCAPE_REQUESTED.store(false, std::sync::atomic::Ordering::SeqCst);

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

                        tracing::info!(
                            target: "script_kit::dictation",
                            event = "dictation_overlay_nswindow_surfaced",
                            main_was_visible,
                            made_key = should_key_overlay,
                            "Surfaced dictation overlay without surfacing sibling launcher panels"
                        );
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

    // Install global escape monitor so Escape works even when the overlay
    // doesn't have keyboard focus (user clicked on another app).
    install_global_escape_monitor();

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
        // Check for global escape before applying state — the escape may
        // close the overlay, in which case set_state is a no-op.
        view.process_global_keys_if_requested(window, cx);
        view.set_state(state, window, cx);
    });

    Ok(())
}

/// Close the dictation overlay window.
pub fn close_dictation_overlay(cx: &mut App) -> anyhow::Result<()> {
    *OVERLAY_ABORT_CALLBACK.lock() = None;
    *OVERLAY_SUBMIT_CALLBACK.lock() = None;
    ESCAPE_REQUESTED.store(false, std::sync::atomic::Ordering::SeqCst);
    // Overlay window already gone is a warning, not an error.
    remove_global_escape_monitor();

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
