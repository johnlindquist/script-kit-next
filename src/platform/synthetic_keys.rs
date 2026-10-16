//! Synthetic keystrokes aimed at the application the user is typing in.
//!
//! Text expansion, `setSelectedText` and paste-to-app all finish by sending
//! Delete or ⌘V. Posting those into the global HID stream has three problems:
//! the keys interleave with whatever the user is still typing, a modifier the
//! user is still holding (Shift from `:`, Option) leaks into them, and our own
//! keyboard monitor sees them as typing. This module instead:
//!
//! - resolves the focused application's process (AX focused application, then
//!   the frontmost application) and posts with `CGEventPostToPid`;
//! - sets the event flags to exactly what the shortcut needs;
//! - tags every event with [`SYNTHETIC_EVENT_TAG`] in the event-source user
//!   data field, so the keyboard monitor can ignore events that still reach
//!   it (the HID fallback route);
//! - finds the ⌘V key from the current keyboard layout instead of assuming
//!   the ANSI V position.
//!
//! `SCRIPT_KIT_SYNTHETIC_KEY_ROUTE=hid` restores the previous global-HID
//! routing if an application turns out not to accept process-targeted keys.

use anyhow::Result;
use std::time::Duration;

/// Marker written to `kCGEventSourceUserData` on every synthetic key event.
pub const SYNTHETIC_EVENT_TAG: i64 = 0x534B_5359_4E4B; // "SKSYNK"

/// Environment switch that forces the legacy global-HID route.
pub const SYNTHETIC_KEY_ROUTE_ENV: &str = "SCRIPT_KIT_SYNTHETIC_KEY_ROUTE";

/// ANSI keycodes used when the layout cannot be read.
pub const KEY_V_ANSI: u16 = 9;
pub const KEY_DELETE: u16 = 51;

/// Where synthetic key events are delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTarget {
    /// Posted directly to one process (`CGEventPostToPid`).
    Process(i32),
    /// Posted into the global HID event stream (legacy route).
    SystemHid,
}

impl KeyTarget {
    pub fn pid(self) -> Option<i32> {
        match self {
            Self::Process(pid) => Some(pid),
            Self::SystemHid => None,
        }
    }

    pub fn route_name(self) -> &'static str {
        match self {
            Self::Process(_) => "process",
            Self::SystemHid => "hid",
        }
    }
}

/// Whether process-targeted delivery is enabled (default) or the
/// `SCRIPT_KIT_SYNTHETIC_KEY_ROUTE=hid` escape hatch is set.
pub fn process_targeting_enabled() -> bool {
    route_from_env(std::env::var(SYNTHETIC_KEY_ROUTE_ENV).ok().as_deref())
}

fn route_from_env(value: Option<&str>) -> bool {
    !matches!(value.map(str::trim), Some(v) if v.eq_ignore_ascii_case("hid"))
}

/// Secure Event Input state. While it is on, macOS withholds keystrokes from
/// every event tap, so text expansion cannot see typing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SecureInputStatus {
    pub enabled: bool,
    pub owner_pid: Option<i32>,
    pub owner_name: Option<String>,
}

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use anyhow::Context;
    use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, EventField};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use foreign_types::ForeignType;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
    use tracing::{debug, info, warn};

    type AXUIElementRef = *const c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: *const c_void,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementGetPid(element: AXUIElementRef, pid: *mut i32) -> i32;
        fn AXUIElementSetMessagingTimeout(element: AXUIElementRef, seconds: f32) -> i32;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSetLocalEventsFilterDuringSuppressionState(
            source: core_graphics::sys::CGEventSourceRef,
            filter: u32,
            state: u32,
        );
        fn CGEventSourceSetLocalEventsSuppressionInterval(
            source: core_graphics::sys::CGEventSourceRef,
            seconds: f64,
        );
        fn CGSessionCopyCurrentDictionary() -> CFTypeRef;
        fn CGEventPostToPid(pid: libc::pid_t, event: core_graphics::sys::CGEventRef);
    }

    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn IsSecureEventInputEnabled() -> u8;
        fn TISCopyCurrentKeyboardLayoutInputSource() -> *mut c_void;
        fn TISGetInputSourceProperty(source: *mut c_void, key: CFStringRef) -> *const c_void;
        static kTISPropertyUnicodeKeyLayoutData: CFStringRef;
        fn LMGetKbdType() -> u8;
        #[allow(clippy::too_many_arguments)]
        fn UCKeyTranslate(
            key_layout: *const c_void,
            virtual_key_code: u16,
            key_action: u16,
            modifier_key_state: u32,
            keyboard_type: u32,
            key_translate_options: u32,
            dead_key_state: *mut u32,
            max_string_length: usize,
            actual_string_length: *mut usize,
            unicode_string: *mut u16,
        ) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
    }

    // libSystem is linked by default.
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// `kCGEventFilterMaskPermitLocalMouseEvents | …LocalKeyboardEvents | …SystemDefinedEvents`
    const PERMIT_ALL_LOCAL_EVENTS: u32 = 0b111;
    const SUPPRESSION_STATE_INTERVAL: u32 = 0;
    const SUPPRESSION_STATE_REMOTE_MOUSE_DRAG: u32 = 1;

    const AX_MESSAGING_TIMEOUT_SECONDS: f32 = 0.25;

    /// Gap between a key's down and up events.
    const KEY_DOWN_UP_GAP: Duration = Duration::from_millis(2);

    static PASTE_KEYCODE: AtomicU16 = AtomicU16::new(KEY_V_ANSI);
    static LAYOUT_REFRESH_PENDING: AtomicBool = AtomicBool::new(false);
    /// Bumped after every layout read on the main thread.
    static LAYOUT_GENERATION: AtomicU64 = AtomicU64::new(0);
    /// Longest an off-main paste waits for the main thread to read the layout.
    const LAYOUT_REFRESH_WAIT: Duration = Duration::from_millis(100);

    /// True when `event` was posted by this module.
    pub fn is_synthetic_event(event: &CGEvent) -> bool {
        event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA) == SYNTHETIC_EVENT_TAG
    }

    /// The process that will receive typed keys: the AX focused application,
    /// then the frontmost application. Falls back to the HID route when
    /// neither is known or targeting is switched off.
    pub fn focused_key_target() -> KeyTarget {
        if !process_targeting_enabled() {
            return KeyTarget::SystemHid;
        }
        if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
            .is_err()
        {
            return KeyTarget::SystemHid;
        }
        objc::rc::autoreleasepool(|| {
            if let Some(pid) = ax_focused_application_pid() {
                return KeyTarget::Process(pid);
            }
            match crate::platform::accessibility::app_identity::current_frontmost_app_identity()
                .process_id
            {
                Some(pid) => KeyTarget::Process(pid),
                None => KeyTarget::SystemHid,
            }
        })
    }

    /// After Script Kit hides its window, the previous app becomes active
    /// asynchronously. Wait (bounded) until keys would go somewhere other
    /// than Script Kit, then return that target. If focus never leaves Script
    /// Kit, fall back to the HID route, which is what the caller did before.
    pub fn key_target_after_hiding_self(timeout: Duration) -> KeyTarget {
        let own_pid = std::process::id() as i32;
        // The hide itself needs the main thread, so waiting there can't help.
        let timeout = if is_main_thread() {
            Duration::ZERO
        } else {
            timeout
        };
        let started = std::time::Instant::now();
        loop {
            let target = focused_key_target();
            match target {
                KeyTarget::Process(pid) if pid != own_pid => {
                    debug!(
                        category = "SYNTHETIC_KEYS",
                        waited_ms = started.elapsed().as_millis() as u64,
                        "key_target_resolved_after_hide"
                    );
                    return target;
                }
                KeyTarget::SystemHid => return target,
                KeyTarget::Process(_) => {}
            }
            if started.elapsed() >= timeout {
                warn!(
                    category = "SYNTHETIC_KEYS",
                    waited_ms = started.elapsed().as_millis() as u64,
                    "focus_still_on_script_kit_after_hide; using HID route"
                );
                return KeyTarget::SystemHid;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn ax_focused_application_pid() -> Option<i32> {
        let attribute = CFString::from_static_string("AXFocusedApplication");
        // SAFETY: the system-wide element and the copied attribute follow the
        // Create/Copy rule and are released below; the pid out-param is a
        // plain i32.
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return None;
            }
            // A hung app must not stall the paste for AX's default ~6 s.
            AXUIElementSetMessagingTimeout(system, AX_MESSAGING_TIMEOUT_SECONDS);
            let mut app: CFTypeRef = std::ptr::null();
            let status = AXUIElementCopyAttributeValue(
                system,
                attribute.as_concrete_TypeRef() as *const c_void,
                &mut app,
            );
            CFRelease(system as CFTypeRef);
            if status != 0 || app.is_null() {
                return None;
            }
            let mut pid: i32 = 0;
            let pid_status = AXUIElementGetPid(app as AXUIElementRef, &mut pid);
            CFRelease(app);
            (pid_status == 0 && pid > 0).then_some(pid)
        }
    }

    fn new_event_source() -> Result<CGEventSource> {
        // A private source: the events carry only the flags we set, not the
        // live modifier state of the hardware keyboard.
        let source = CGEventSource::new(CGEventSourceStateID::Private)
            .ok()
            .context("Failed to create CGEventSource")?;
        // Don't suppress the user's real keyboard and mouse while our events
        // are in flight (the default suppression interval is 0.25 s).
        // SAFETY: `source` is a valid CGEventSource for these calls.
        unsafe {
            let raw = source.as_ptr();
            CGEventSourceSetLocalEventsFilterDuringSuppressionState(
                raw,
                PERMIT_ALL_LOCAL_EVENTS,
                SUPPRESSION_STATE_INTERVAL,
            );
            CGEventSourceSetLocalEventsFilterDuringSuppressionState(
                raw,
                PERMIT_ALL_LOCAL_EVENTS,
                SUPPRESSION_STATE_REMOTE_MOUSE_DRAG,
            );
            CGEventSourceSetLocalEventsSuppressionInterval(raw, 0.0);
        }
        Ok(source)
    }

    fn post(event: &CGEvent, target: KeyTarget) {
        match target {
            // SAFETY: `event` is a valid CGEvent for the duration of the call.
            KeyTarget::Process(pid) => unsafe { CGEventPostToPid(pid, event.as_ptr()) },
            KeyTarget::SystemHid => event.post(CGEventTapLocation::HID),
        }
    }

    /// Post one key press (down + up) with exactly `flags`.
    pub fn post_key(target: KeyTarget, keycode: u16, flags: CGEventFlags) -> Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
        let source = new_event_source()?;
        let down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .ok()
            .context("Failed to create key down event")?;
        let up = CGEvent::new_keyboard_event(source, keycode, false)
            .ok()
            .context("Failed to create key up event")?;
        for event in [&down, &up] {
            event.set_flags(flags);
            event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, SYNTHETIC_EVENT_TAG);
        }
        post(&down, target);
        std::thread::sleep(KEY_DOWN_UP_GAP);
        post(&up, target);
        Ok(())
    }

    /// Post `count` Delete presses with no modifiers.
    pub fn post_deletes(target: KeyTarget, count: usize, gap: Duration) -> Result<()> {
        for index in 0..count {
            post_key(target, KEY_DELETE, CGEventFlags::CGEventFlagNull)?;
            if index + 1 < count && !gap.is_zero() {
                std::thread::sleep(gap);
            }
        }
        Ok(())
    }

    /// Post ⌘V using the V key of the current keyboard layout.
    pub fn post_paste_shortcut(target: KeyTarget) -> Result<()> {
        let keycode = paste_keycode();
        post_key(target, keycode, CGEventFlags::CGEventFlagCommand)?;
        debug!(
            category = "SYNTHETIC_KEYS",
            route = target.route_name(),
            keycode,
            "posted_paste_shortcut"
        );
        Ok(())
    }

    /// The keycode that produces "v" with ⌘ held in the current layout.
    ///
    /// Off the main thread this asks the main thread to re-read the layout
    /// (so a layout switch since the last paste is honoured) and waits briefly
    /// for the answer, falling back to the last known keycode.
    pub fn paste_keycode() -> u16 {
        if is_main_thread() {
            refresh_layout_keycodes_now();
        } else {
            let generation = LAYOUT_GENERATION.load(Ordering::Acquire);
            refresh_layout_keycodes();
            let started = std::time::Instant::now();
            while LAYOUT_GENERATION.load(Ordering::Acquire) == generation
                && started.elapsed() < LAYOUT_REFRESH_WAIT
            {
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        PASTE_KEYCODE.load(Ordering::Acquire)
    }

    /// Recompute layout-dependent keycodes. Text Input Source APIs must run on
    /// the main thread (they assert on recent macOS), so off-main callers
    /// schedule the work there and keep using the cached value.
    pub fn refresh_layout_keycodes() {
        if is_main_thread() {
            refresh_layout_keycodes_now();
            return;
        }
        if LAYOUT_REFRESH_PENDING.swap(true, Ordering::AcqRel) {
            return;
        }
        extern "C" fn trampoline(_context: *mut c_void) {
            LAYOUT_REFRESH_PENDING.store(false, Ordering::Release);
            let _ = std::panic::catch_unwind(refresh_layout_keycodes_now);
        }
        // SAFETY: the main queue symbol is a valid GCD queue; the trampoline
        // takes no context.
        unsafe {
            dispatch_async_f(
                &DISPATCH_MAIN_QUEUE as *const c_void,
                std::ptr::null_mut(),
                trampoline,
            );
        }
    }

    fn refresh_layout_keycodes_now() {
        let found = keycode_for_command_character('v');
        LAYOUT_GENERATION.fetch_add(1, Ordering::AcqRel);
        match found {
            Some(keycode) => {
                let previous = PASTE_KEYCODE.swap(keycode, Ordering::AcqRel);
                if previous != keycode {
                    info!(
                        category = "SYNTHETIC_KEYS",
                        keycode, "paste_keycode_updated_from_layout"
                    );
                }
            }
            None => debug!(
                category = "SYNTHETIC_KEYS",
                "keyboard layout unavailable; keeping cached paste keycode"
            ),
        }
    }

    /// Search the current layout for the key that types `wanted` while ⌘ is
    /// held. Main thread only.
    fn keycode_for_command_character(wanted: char) -> Option<u16> {
        const KEY_ACTION_DISPLAY: u16 = 3;
        const COMMAND_MODIFIER_STATE: u32 = 1; // (cmdKey >> 8) & 0xFF
        const NO_DEAD_KEYS: u32 = 1;
        // SAFETY: Main-thread TIS calls. The input source is released after
        // use; the layout data pointer stays valid while the source lives.
        unsafe {
            let source = TISCopyCurrentKeyboardLayoutInputSource();
            if source.is_null() {
                return None;
            }
            let data = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData);
            let layout = if data.is_null() {
                std::ptr::null()
            } else {
                CFDataGetBytePtr(data)
            };
            let mut found = None;
            if !layout.is_null() {
                let keyboard_type = LMGetKbdType() as u32;
                for keycode in 0u16..128 {
                    let mut dead_key_state = 0u32;
                    let mut length = 0usize;
                    let mut chars = [0u16; 4];
                    let status = UCKeyTranslate(
                        layout as *const c_void,
                        keycode,
                        KEY_ACTION_DISPLAY,
                        COMMAND_MODIFIER_STATE,
                        keyboard_type,
                        NO_DEAD_KEYS,
                        &mut dead_key_state,
                        chars.len(),
                        &mut length,
                        chars.as_mut_ptr(),
                    );
                    if status == 0
                        && length == 1
                        && char::from_u32(chars[0] as u32)
                            .is_some_and(|c| c.eq_ignore_ascii_case(&wanted))
                    {
                        found = Some(keycode);
                        break;
                    }
                }
            }
            CFRelease(source as CFTypeRef);
            found
        }
    }

    fn is_main_thread() -> bool {
        use objc::{class, msg_send, sel, sel_impl};
        // SAFETY: NSThread.isMainThread only reads thread identity.
        unsafe { msg_send![class!(NSThread), isMainThread] }
    }

    /// Whether Secure Event Input is on and, when macOS reports it, which
    /// process turned it on.
    pub fn secure_input_status() -> SecureInputStatus {
        // SAFETY: plain Carbon query with no arguments.
        let enabled = unsafe { IsSecureEventInputEnabled() } != 0;
        if !enabled {
            return SecureInputStatus::default();
        }
        let owner_pid = secure_input_owner_pid();
        let owner_name = objc::rc::autoreleasepool(|| owner_pid.and_then(application_name_for_pid));
        SecureInputStatus {
            enabled,
            owner_pid,
            owner_name,
        }
    }

    fn secure_input_owner_pid() -> Option<i32> {
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::number::CFNumber;
        // SAFETY: CGSessionCopyCurrentDictionary follows the Create rule; the
        // wrapper releases it.
        unsafe {
            let raw = CGSessionCopyCurrentDictionary();
            if raw.is_null() {
                return None;
            }
            let dictionary: CFDictionary<CFString, core_foundation::base::CFType> =
                CFDictionary::wrap_under_create_rule(raw as _);
            let key = CFString::from_static_string("kCGSSessionSecureInputPID");
            let value = dictionary.find(&key)?;
            let number = value.downcast::<CFNumber>()?;
            number.to_i32().filter(|pid| *pid > 0)
        }
    }

    fn application_name_for_pid(pid: i32) -> Option<String> {
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};
        // SAFETY: NSRunningApplication lookups are nil-checked.
        unsafe {
            let app: *mut Object = msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid];
            if app.is_null() {
                return None;
            }
            let name: *mut Object = msg_send![app, localizedName];
            if name.is_null() {
                return None;
            }
            let utf8: *const std::os::raw::c_char = msg_send![name, UTF8String];
            if utf8.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(utf8)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod fallback {
    use super::*;

    pub fn focused_key_target() -> KeyTarget {
        KeyTarget::SystemHid
    }

    pub fn key_target_after_hiding_self(_timeout: Duration) -> KeyTarget {
        KeyTarget::SystemHid
    }

    pub fn post_deletes(_target: KeyTarget, _count: usize, _gap: Duration) -> Result<()> {
        anyhow::bail!("synthetic keys require macOS")
    }

    pub fn post_paste_shortcut(_target: KeyTarget) -> Result<()> {
        anyhow::bail!("synthetic keys require macOS")
    }

    pub fn refresh_layout_keycodes() {}

    pub fn secure_input_status() -> SecureInputStatus {
        SecureInputStatus::default()
    }
}

#[cfg(not(target_os = "macos"))]
pub use fallback::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_route_is_default_and_hid_is_opt_in() {
        assert!(route_from_env(None));
        assert!(route_from_env(Some("")));
        assert!(route_from_env(Some("process")));
        assert!(!route_from_env(Some("hid")));
        assert!(!route_from_env(Some(" HID ")));
    }

    #[test]
    fn key_target_reports_route_and_pid() {
        assert_eq!(KeyTarget::Process(42).pid(), Some(42));
        assert_eq!(KeyTarget::Process(42).route_name(), "process");
        assert_eq!(KeyTarget::SystemHid.pid(), None);
        assert_eq!(KeyTarget::SystemHid.route_name(), "hid");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn synthetic_events_carry_the_tag_and_real_ones_do_not() {
        use core_graphics::event::{CGEvent, EventField};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let source = CGEventSource::new(CGEventSourceStateID::Private).expect("event source");
        let event = CGEvent::new_keyboard_event(source, KEY_DELETE, true).expect("event");
        assert!(!is_synthetic_event(&event));
        event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, SYNTHETIC_EVENT_TAG);
        assert!(is_synthetic_event(&event));
    }
}
