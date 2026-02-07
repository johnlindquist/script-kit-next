use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use macos_accessibility_client::accessibility;
use thiserror::Error;
use tracing::{debug, error, info, warn};
// =============================================================================
// DEBOUNCED LOGGING
// =============================================================================
// To avoid log spam, we accumulate keystroke counts and only log a summary
// after 3 seconds of inactivity.

/// How long to wait after the last keystroke before logging a summary
const LOG_DEBOUNCE_SECS: u64 = 3;
/// State for debounced keystroke logging
struct DebouncedLogState {
    /// Number of keystrokes since last log
    count: AtomicU64,
    /// Timestamp of first keystroke in current batch (millis since UNIX epoch)
    batch_start_ms: AtomicU64,
    /// Timestamp of last keystroke (millis since UNIX epoch)
    last_keystroke_ms: AtomicU64,
}
impl DebouncedLogState {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            batch_start_ms: AtomicU64::new(0),
            last_keystroke_ms: AtomicU64::new(0),
        }
    }

    /// Record a keystroke. Returns (should_log_summary, count, duration_ms) if we should
    /// log a summary of the previous batch before starting a new one.
    fn record(&self) -> Option<(u64, u64)> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last = self.last_keystroke_ms.load(Ordering::Relaxed);
        let debounce_ms = LOG_DEBOUNCE_SECS * 1000;

        // Check if we should flush the previous batch
        let result = if last > 0 && now_ms.saturating_sub(last) > debounce_ms {
            // Time gap exceeded - log previous batch and reset
            let count = self.count.swap(0, Ordering::Relaxed);
            let batch_start = self.batch_start_ms.load(Ordering::Relaxed);
            let duration_ms = last.saturating_sub(batch_start);

            // Start new batch
            self.batch_start_ms.store(now_ms, Ordering::Relaxed);

            if count > 0 {
                Some((count, duration_ms))
            } else {
                None
            }
        } else if self.count.load(Ordering::Relaxed) == 0 {
            // First keystroke of a new batch
            self.batch_start_ms.store(now_ms, Ordering::Relaxed);
            None
        } else {
            None
        };

        // Record this keystroke
        self.count.fetch_add(1, Ordering::Relaxed);
        self.last_keystroke_ms.store(now_ms, Ordering::Relaxed);

        result
    }
}
/// Global debounced log state
static DEBOUNCED_LOG: std::sync::OnceLock<DebouncedLogState> = std::sync::OnceLock::new();
fn debounced_log_state() -> &'static DebouncedLogState {
    DEBOUNCED_LOG.get_or_init(DebouncedLogState::new)
}
/// Errors that can occur when using the keyboard monitor
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KeyboardMonitorError {
    #[error("Accessibility permissions not granted. Please enable in System Preferences > Privacy & Security > Accessibility")]
    AccessibilityNotGranted,

    #[error("Failed to create event tap - this may indicate accessibility permissions issue")]
    EventTapCreationFailed,

    #[error("Failed to create run loop source from event tap")]
    RunLoopSourceCreationFailed,

    #[error("Monitor is already running")]
    AlreadyRunning,

    #[error("Monitor is not running")]
    NotRunning,

    #[error("Failed to start monitor thread")]
    ThreadSpawnFailed,
}
/// Represents a keyboard event captured by the monitor
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyEvent {
    /// The character that was typed (if available)
    /// This is the actual character produced, taking into account modifiers
    pub character: Option<String>,

    /// The virtual key code (hardware key identifier)
    pub key_code: u16,

    /// Whether the shift modifier was held
    pub shift: bool,

    /// Whether the control modifier was held
    pub control: bool,

    /// Whether the option/alt modifier was held
    pub option: bool,

    /// Whether the command modifier was held
    pub command: bool,

    /// Whether this is an auto-repeat event
    pub is_repeat: bool,
}
/// Callback type for receiving keyboard events
/// Must be Send + Sync since it's shared across threads via Arc
pub type KeyEventCallback = Box<dyn Fn(KeyEvent) + Send + Sync + 'static>;
/// Wrapper for CFMachPortRef that is Send + Sync
/// SAFETY: The mach port is only accessed from within the event tap callback
/// and the event loop thread. The closure and event loop run on the same thread.
struct SendableMachPortRef(Option<CFMachPortRef>);
// SAFETY: CFMachPortRef is a thread-safe Core Foundation object reference.
// We only access it from the same thread (event loop thread).
unsafe impl Send for SendableMachPortRef {}
unsafe impl Sync for SendableMachPortRef {}
/// Global keyboard monitor using macOS CGEventTap
///
/// This monitor captures keystrokes system-wide, regardless of which application
/// has focus. It runs on a dedicated background thread with its own CFRunLoop.
pub struct KeyboardMonitor {
    /// Whether the monitor is currently running
    running: Arc<AtomicBool>,

    /// Handle to the background thread running the event loop
    thread_handle: Option<JoinHandle<()>>,

    /// The callback to invoke for each key event
    callback: Arc<KeyEventCallback>,

    /// Run loop reference for stopping (stored after start)
    run_loop: Arc<std::sync::Mutex<Option<CFRunLoop>>>,
}
