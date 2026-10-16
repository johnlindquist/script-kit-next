//! Global keyboard monitoring using macOS CGEventTap API
//!
//! This module provides system-wide keyboard event capture, regardless of which
//! application has focus. This is essential for text expansion/snippet features
//! that need to detect trigger sequences typed in any application.
//!
//! # Requirements
//! - macOS only (uses Core Graphics CGEventTap)
//! - Requires Accessibility permissions to be enabled in System Preferences
//!
//! # Example
//! ```no_run
//! use script_kit_gpui::keyboard_monitor::{KeyboardMonitor, KeyEvent};
//!
//! let mut monitor = KeyboardMonitor::new(|event: KeyEvent| {
//!     println!("Key pressed: {:?}", event.character);
//! });
//!
//! monitor.start().expect("Failed to start keyboard monitor");
//! // ... monitor runs in background thread ...
//! monitor.stop();
//! ```

// --- merged from part_000.rs ---
use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use macos_accessibility_client::accessibility;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
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
/// How often the event loop checks that the tap is still enabled and whether
/// Secure Input changed.
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// How long `start()` waits for the monitor thread to report its event tap.
const TAP_READY_TIMEOUT: Duration = Duration::from_secs(2);

/// Last observed Secure Input state (see [`secure_input_status`]).
static LAST_SECURE_INPUT: std::sync::Mutex<crate::platform::synthetic_keys::SecureInputStatus> =
    std::sync::Mutex::new(crate::platform::synthetic_keys::SecureInputStatus {
        enabled: false,
        owner_pid: None,
        owner_name: None,
    });

/// Secure Input as last seen by the running keyboard monitor (checked every
/// few seconds). While `enabled`, keyword expansion cannot see typing.
pub fn secure_input_status() -> crate::platform::synthetic_keys::SecureInputStatus {
    LAST_SECURE_INPUT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Global debounced log state
static DEBOUNCED_LOG: std::sync::LazyLock<DebouncedLogState> =
    std::sync::LazyLock::new(DebouncedLogState::new);
fn debounced_log_state() -> &'static DebouncedLogState {
    &DEBOUNCED_LOG
}
/// Errors that can occur when using the keyboard monitor
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KeyboardMonitorError {
    #[error("{0}")]
    PolicyRefused(crate::runtime_policy::EffectRefusal),
    #[error(
        "Accessibility permissions not granted. Please enable in System Preferences > Privacy & Security > Accessibility"
    )]
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
/// What kind of input the monitor saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventKind {
    /// A key went down. `character` and `key_code` describe it.
    KeyDown,
    /// A mouse button went down. The caret may have moved, so anything typed
    /// so far no longer sits in front of it. Only `kind` is meaningful.
    PointerDown,
}

/// Represents a keyboard event captured by the monitor
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyEvent {
    /// Key press or pointer press
    pub kind: KeyEventKind,

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

// --- merged from part_001.rs ---
impl KeyboardMonitor {
    /// Create a new keyboard monitor with the given callback
    ///
    /// The callback will be invoked for each key-down event captured.
    /// The monitor does not start automatically - call `start()` to begin monitoring.
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(KeyEvent) + Send + Sync + 'static,
    {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            callback: Arc::new(Box::new(callback)),
            run_loop: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Check if accessibility permissions are granted
    ///
    /// Returns true if the application has been granted accessibility permissions.
    /// If false, the monitor will fail to start.
    pub fn has_accessibility_permission() -> bool {
        crate::permissions::accessibility_granted()
    }

    /// Check if accessibility permissions are granted, prompting the user if not
    ///
    /// This will show the macOS accessibility permission dialog if permissions
    /// haven't been granted yet. Returns true if permissions are granted.
    #[allow(dead_code)]
    pub fn request_accessibility_permission() -> bool {
        if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)
            .is_err()
        {
            return false;
        }
        accessibility::application_is_trusted_with_prompt()
    }

    /// Start the keyboard monitor
    ///
    /// This spawns a background thread that captures keyboard events system-wide.
    /// The provided callback will be invoked for each key-down event.
    ///
    /// # Errors
    /// - `AccessibilityNotGranted` - Accessibility permissions not enabled
    /// - `AlreadyRunning` - Monitor is already running
    /// - `EventTapCreationFailed` - Failed to create the event tap
    pub fn start(&mut self) -> Result<(), KeyboardMonitorError> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::GlobalMonitor)
            .map_err(KeyboardMonitorError::PolicyRefused)?;
        // Check if already running
        if self.running.load(Ordering::SeqCst) {
            return Err(KeyboardMonitorError::AlreadyRunning);
        }

        // Check accessibility permissions
        if !Self::has_accessibility_permission() {
            warn!("Accessibility permissions not granted for keyboard monitor");
            return Err(KeyboardMonitorError::AccessibilityNotGranted);
        }

        info!("Starting global keyboard monitor");

        let running = Arc::clone(&self.running);
        let callback = Arc::clone(&self.callback);
        let run_loop_storage = Arc::clone(&self.run_loop);
        // The tap is created on the monitor thread; wait for its answer so a
        // refused tap is an error here instead of a log line nobody reads.
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);

        // Set running flag before spawning thread
        self.running.store(true, Ordering::SeqCst);

        let handle = thread::Builder::new()
            .name("keyboard-monitor".to_string())
            .spawn(move || {
                Self::event_loop(running, callback, run_loop_storage, ready_tx);
            })
            .map_err(|e| {
                error!("Failed to spawn keyboard monitor thread: {}", e);
                self.running.store(false, Ordering::SeqCst);
                KeyboardMonitorError::ThreadSpawnFailed
            })?;

        match ready_rx.recv_timeout(TAP_READY_TIMEOUT) {
            Ok(Ok(())) => {
                self.thread_handle = Some(handle);
                Ok(())
            }
            Ok(Err(error)) => {
                self.running.store(false, Ordering::SeqCst);
                let _ = handle.join();
                Err(error)
            }
            Err(_) => {
                // No answer in time: keep the thread (it may still finish
                // setting up) and let `is_running` report the truth later.
                warn!(
                    category = "KEYWORD",
                    "Keyboard monitor did not report its event tap in time"
                );
                self.thread_handle = Some(handle);
                Ok(())
            }
        }
    }

    /// Stop the keyboard monitor
    ///
    /// This stops the background thread and cleans up resources.
    /// Safe to call even if the monitor is not running.
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            debug!("Keyboard monitor already stopped");
            return;
        }

        info!("Stopping global keyboard monitor");

        // Signal the thread to stop
        self.running.store(false, Ordering::SeqCst);

        // Stop the run loop
        if let Ok(guard) = self.run_loop.lock() {
            if let Some(ref run_loop) = *guard {
                run_loop.stop();
            }
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Keyboard monitor thread panicked: {:?}", e);
            }
        }

        debug!("Keyboard monitor stopped");
    }

    /// Check if the monitor is currently running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// The main event loop that runs on the background thread
    fn event_loop(
        running: Arc<AtomicBool>,
        callback: Arc<KeyEventCallback>,
        run_loop_storage: Arc<std::sync::Mutex<Option<CFRunLoop>>>,
        ready: std::sync::mpsc::SyncSender<Result<(), KeyboardMonitorError>>,
    ) {
        debug!("Keyboard monitor event loop starting");

        // Get current run loop and store it for stopping
        let current_run_loop = CFRunLoop::get_current();
        if let Ok(mut guard) = run_loop_storage.lock() {
            *guard = Some(current_run_loop.clone());
        }

        // Shared storage for the mach port ref, so callback can re-enable tap if disabled
        // SAFETY: This is set immediately after tap creation, before any callbacks can fire
        let mach_port_ref: Arc<std::sync::Mutex<SendableMachPortRef>> =
            Arc::new(std::sync::Mutex::new(SendableMachPortRef(None)));
        let mach_port_for_callback = Arc::clone(&mach_port_ref);

        // Key presses feed the keyword buffer; mouse presses reset it (a click
        // can move the caret away from what was typed).
        debug!("Creating CGEventTap with HID location for KeyDown and mouse-down events");
        let event_tap_result = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly, // We only observe, don't modify
            vec![
                CGEventType::KeyDown,
                CGEventType::LeftMouseDown,
                CGEventType::RightMouseDown,
                CGEventType::OtherMouseDown,
            ],
            move |_proxy, event_type, event: &CGEvent| {
                // CRITICAL: Check for tap disabled events first
                // When a tap is disabled (timeout or user), macOS sends a special event
                // We must re-enable the tap to continue receiving events
                match event_type {
                    CGEventType::TapDisabledByTimeout => {
                        warn!("CGEventTap disabled by timeout - re-enabling");
                        Self::reenable_tap(&mach_port_for_callback);
                        return None;
                    }
                    CGEventType::TapDisabledByUserInput => {
                        warn!("CGEventTap disabled by user input - re-enabling");
                        Self::reenable_tap(&mach_port_for_callback);
                        return None;
                    }
                    _ => {}
                }

                if matches!(
                    event_type,
                    CGEventType::LeftMouseDown
                        | CGEventType::RightMouseDown
                        | CGEventType::OtherMouseDown
                ) {
                    callback(Self::pointer_down_event());
                    return None;
                }

                // Only process KeyDown events (filter out any other event types)
                if !matches!(event_type, CGEventType::KeyDown) {
                    return None;
                }

                // Keys Script Kit posted itself (Delete, ⌘V) are not typing.
                if crate::platform::synthetic_keys::is_synthetic_event(event) {
                    return None;
                }

                // Extract key event information
                let key_event = Self::extract_key_event(event);

                // Debounced logging: accumulate keystrokes and log summary after 3s of inactivity
                // NOTE: We intentionally do NOT log individual characters for privacy
                let log_state = debounced_log_state();
                if let Some((count, duration_ms)) = log_state.record() {
                    debug!(
                        keystroke_count = count,
                        duration_ms = duration_ms,
                        "Keyboard monitor: {} keystrokes captured over {}ms",
                        count,
                        duration_ms
                    );
                }

                // Invoke callback
                callback(key_event);

                // Return None to not modify the event (we're just observing)
                None
            },
        );

        let event_tap = match event_tap_result {
            Ok(tap) => tap,
            Err(()) => {
                error!(
                    "Failed to create CGEventTap - accessibility permissions may not be granted"
                );
                running.store(false, Ordering::SeqCst);
                let _ = ready.send(Err(KeyboardMonitorError::EventTapCreationFailed));
                return;
            }
        };

        // Store the mach port ref so the callback can re-enable if disabled
        if let Ok(mut guard) = mach_port_ref.lock() {
            guard.0 = Some(event_tap.mach_port.as_concrete_TypeRef());
        }

        // Create run loop source from the event tap
        let run_loop_source = match event_tap.mach_port.create_runloop_source(0) {
            Ok(source) => source,
            Err(()) => {
                error!("Failed to create run loop source from event tap");
                running.store(false, Ordering::SeqCst);
                let _ = ready.send(Err(KeyboardMonitorError::RunLoopSourceCreationFailed));
                return;
            }
        };

        // Add source to run loop and enable the tap
        // SAFETY: run_loop_source was just created from a valid event tap.
        // kCFRunLoopCommonModes is a valid Core Foundation constant.
        unsafe {
            current_run_loop.add_source(&run_loop_source, kCFRunLoopCommonModes);
        }
        event_tap.enable();
        let _ = ready.send(Ok(()));

        info!("Keyboard monitor event tap enabled, entering run loop");

        let mut last_health_check = std::time::Instant::now();
        let tap_port = event_tap.mach_port.as_concrete_TypeRef();
        Self::check_secure_input();

        // Run the loop until stopped
        while running.load(Ordering::SeqCst) {
            if last_health_check.elapsed() >= HEALTH_CHECK_INTERVAL {
                last_health_check = std::time::Instant::now();
                Self::check_tap_enabled(tap_port);
                Self::check_secure_input();
            }

            // Run for a short interval, then check if we should stop
            // Note: Must use kCFRunLoopDefaultMode (not kCFRunLoopCommonModes) for run_in_mode
            let result = CFRunLoop::run_in_mode(
                // SAFETY: kCFRunLoopDefaultMode is a valid static Core Foundation string constant.
                unsafe { kCFRunLoopDefaultMode },
                Duration::from_millis(100),
                true,
            );

            // Check if run loop was stopped externally
            if matches!(
                result,
                core_foundation::runloop::CFRunLoopRunResult::Stopped
            ) {
                debug!("Run loop was stopped");
                break;
            }
        }

        // Clean up: remove source from run loop
        // Note: The event tap will be disabled when it goes out of scope

        debug!("Keyboard monitor event loop exiting");
        running.store(false, Ordering::SeqCst);

        // Clear the stored run loop
        if let Ok(mut guard) = run_loop_storage.lock() {
            *guard = None;
        }
    }

    /// Watchdog: macOS can disable a tap without delivering the
    /// "tap disabled" event (for example after the system was under load).
    /// A disabled tap means silent text expansion, so turn it back on.
    fn check_tap_enabled(port: CFMachPortRef) {
        extern "C" {
            fn CGEventTapIsEnabled(tap: CFMachPortRef) -> bool;
            fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        }
        // SAFETY: `port` is the live tap created in `event_loop`, which owns
        // it for the duration of this loop.
        unsafe {
            if !CGEventTapIsEnabled(port) {
                warn!(
                    category = "KEYWORD",
                    event = "keyboard_monitor_tap_found_disabled",
                    "Keyboard monitor tap was disabled without notice - re-enabling"
                );
                CGEventTapEnable(port, true);
            }
        }
    }

    /// Track Secure Event Input. While it is on, macOS sends no keystrokes to
    /// event taps, so keywords cannot expand; say so once per change.
    fn check_secure_input() {
        let status = crate::platform::synthetic_keys::secure_input_status();
        let mut last = LAST_SECURE_INPUT.lock().unwrap_or_else(|e| e.into_inner());
        if *last == status {
            return;
        }
        if status.enabled {
            warn!(
                category = "KEYWORD",
                event = "secure_input_changed",
                enabled = true,
                owner_pid = status.owner_pid.unwrap_or(0),
                owner = %status.owner_name.as_deref().unwrap_or("unknown"),
                "Secure Input is on: keyword expansion cannot see typing until it turns off"
            );
        } else {
            info!(
                category = "KEYWORD",
                event = "secure_input_changed",
                enabled = false,
                "Secure Input is off: keyword expansion can see typing"
            );
        }
        *last = status;
    }

    fn pointer_down_event() -> KeyEvent {
        KeyEvent {
            kind: KeyEventKind::PointerDown,
            character: None,
            key_code: 0,
            shift: false,
            control: false,
            option: false,
            command: false,
            is_repeat: false,
        }
    }

    /// Re-enable an event tap that was disabled by timeout or user input
    fn reenable_tap(mach_port_ref: &Arc<std::sync::Mutex<SendableMachPortRef>>) {
        extern "C" {
            fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        }

        if let Ok(guard) = mach_port_ref.lock() {
            if let Some(port) = guard.0 {
                // SAFETY: `port` is a valid CFMachPort obtained from CGEventTapCreate.
                // CGEventTapEnable is safe to call with a valid port and boolean.
                unsafe {
                    CGEventTapEnable(port, true);
                }
                info!("CGEventTap re-enabled successfully");
            } else {
                error!("Cannot re-enable tap: mach port ref not set");
            }
        } else {
            error!("Cannot re-enable tap: failed to acquire lock");
        }
    }

    /// Extract key event information from a CGEvent
    fn extract_key_event(event: &CGEvent) -> KeyEvent {
        use core_graphics::event::CGEventFlags;

        // Get key code
        let key_code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;

        // Get modifier flags
        let flags = event.get_flags();
        let shift = flags.contains(CGEventFlags::CGEventFlagShift);
        let control = flags.contains(CGEventFlags::CGEventFlagControl);
        let option = flags.contains(CGEventFlags::CGEventFlagAlternate);
        let command = flags.contains(CGEventFlags::CGEventFlagCommand);

        // Check if auto-repeat
        let is_repeat = event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;

        // Try to get the character from the event
        // This uses the CGEventKeyboardGetUnicodeString function internally
        let character = Self::get_character_from_event(event);

        KeyEvent {
            kind: KeyEventKind::KeyDown,
            character,
            key_code,
            shift,
            control,
            option,
            command,
            is_repeat,
        }
    }

    /// Get the character string from a keyboard event
    ///
    /// This attempts to get the actual character that would be typed,
    /// taking into account the keyboard layout and modifier keys.
    fn get_character_from_event(event: &CGEvent) -> Option<String> {
        // We need to use the CGEventKeyboardGetUnicodeString function
        // which isn't directly exposed by core-graphics, so we'll use FFI
        extern "C" {
            fn CGEventKeyboardGetUnicodeString(
                event: core_graphics::sys::CGEventRef,
                max_len: libc::c_ulong,
                actual_len: *mut libc::c_ulong,
                buffer: *mut u16,
            );
        }

        // Buffer size of 32 UTF-16 code units handles:
        // - Simple characters (1 code unit)
        // - BMP characters including most CJK (1 code unit)
        // - Emoji and surrogate pairs (2 code units)
        // - Complex emoji sequences with ZWJ (up to ~15 code units)
        // - IME composed characters
        const BUFFER_SIZE: usize = 32;
        let mut buffer: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut actual_len: libc::c_ulong = 0;

        // SAFETY: `event` is a valid CGEvent reference. `buffer` is a stack-allocated
        // array of sufficient size (BUFFER_SIZE=32). `actual_len` receives the number of
        // UTF-16 code units written, bounded by BUFFER_SIZE.
        unsafe {
            use foreign_types::ForeignType;
            let event_ptr = event.as_ptr();
            CGEventKeyboardGetUnicodeString(
                event_ptr,
                BUFFER_SIZE as libc::c_ulong,
                &mut actual_len,
                buffer.as_mut_ptr(),
            );
        }

        if actual_len > 0 && (actual_len as usize) <= BUFFER_SIZE {
            String::from_utf16(&buffer[..actual_len as usize]).ok()
        } else {
            None
        }
    }
}
impl Drop for KeyboardMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}
// SAFETY: KeyboardMonitor is Send because all its fields are Send: running (Arc<AtomicBool>),
// thread_handle (Option<JoinHandle>), callback (Arc<KeyEventCallback> where KeyEventCallback: Send),
// and run_loop (Arc<Mutex<Option<CFRunLoop>>>). The event tap runs on a dedicated thread.
unsafe impl Send for KeyboardMonitor {}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_check_does_not_panic() {
        // This test just verifies the accessibility check doesn't panic
        // The actual result depends on system permissions
        let _ = KeyboardMonitor::has_accessibility_permission();
    }

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent {
            kind: KeyEventKind::KeyDown,
            character: Some("a".to_string()),
            key_code: 0,
            shift: false,
            control: false,
            option: false,
            command: false,
            is_repeat: false,
        };

        assert_eq!(event.character, Some("a".to_string()));
        assert!(!event.shift);
        assert!(!event.is_repeat);
    }

    #[test]
    fn test_monitor_not_running_initially() {
        let monitor = KeyboardMonitor::new(|_| {});
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_stop_when_not_running_is_safe() {
        let mut monitor = KeyboardMonitor::new(|_| {});
        // Should not panic
        monitor.stop();
        assert!(!monitor.is_running());
    }

    // Integration tests that require accessibility permissions are marked as ignored
    // Run with: cargo test --features system-tests -- --ignored
    #[test]
    #[ignore = "Requires accessibility permissions"]
    fn test_start_and_stop() {
        let mut monitor = KeyboardMonitor::new(|event| {
            println!("Key event: {:?}", event);
        });

        if !KeyboardMonitor::has_accessibility_permission() {
            eprintln!("Skipping test - accessibility permissions not granted");
            return;
        }

        assert!(monitor.start().is_ok());
        assert!(monitor.is_running());

        // Let it run briefly
        std::thread::sleep(std::time::Duration::from_millis(100));

        monitor.stop();
        assert!(!monitor.is_running());
    }

    #[test]
    #[ignore = "Requires accessibility permissions"]
    fn test_double_start_fails() {
        let mut monitor = KeyboardMonitor::new(|_| {});

        if !KeyboardMonitor::has_accessibility_permission() {
            eprintln!("Skipping test - accessibility permissions not granted");
            return;
        }

        assert!(monitor.start().is_ok());
        assert!(matches!(
            monitor.start(),
            Err(KeyboardMonitorError::AlreadyRunning)
        ));

        monitor.stop();
    }
}
