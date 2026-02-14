// Platform cursor management for non-activating popup windows.
//
// GPUI's built-in cursor system (`reset_cursor_style`) only applies when the window
// is "active" (`is_window_hovered()` -> `is_window_active()`). For non-activating
// `PopUp` panels (NSPanel with NonactivatingPanel style), the window is never "active",
// so GPUI never pushes cursor changes to the OS. This causes the underlying app's
// cursor to bleed through (e.g. I-beam from a terminal).
//
// Two-layer fix:
//
// Layer 1 — Window-server cursor permission (macOS):
//   `install_cursor_tracking()` enables the private CoreGraphics connection property
//   `SetsCursorInBackground`, which tells WindowServer to respect `[NSCursor set]`
//   from our non-activating panel context.
//
// Layer 2 — Mouse-move coordination (GPUI event level):
//   Interactive elements call `claim_cursor_pointer()` in their `on_mouse_move`.
//   The root element calls `apply_default_cursor()` last (bubble phase, outer-to-inner).
//   `apply_default_cursor()` calls `[NSCursor set]` unconditionally on every move
//   to override whatever cursor rects selected between events.

use std::cell::Cell;

thread_local! {
    static CURSOR_CLAIMED: Cell<bool> = const { Cell::new(false) };
}

// ============================================================================
// Layer 1 — Window-server cursor permission
// ============================================================================

#[cfg(target_os = "macos")]
use cocoa::base::YES;
#[cfg(target_os = "macos")]
use std::os::raw::c_int;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGSMainConnectionID() -> c_int;
    fn CGSSetConnectionProperty(
        cid: c_int,
        target_cid: c_int,
        key: cocoa::base::id,
        value: cocoa::base::id,
    ) -> c_int;
}

/// Enable background cursor-setting support for this app's WindowServer connection.
///
/// Safe to call multiple times — uses `Once` to ensure single installation.
#[cfg(target_os = "macos")]
pub fn install_cursor_tracking() {
    use std::sync::Once;
    static INSTALL: Once = Once::new();

    INSTALL.call_once(|| {
        // SAFETY: Main thread call site, ObjC usage with null checks, and direct
        // CoreGraphics C-API invocation for current process connection only.
        unsafe {
            let connection_id = CGSMainConnectionID();
            if connection_id == 0 {
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: CGSMainConnectionID returned 0",
                );
                return;
            }

            let key_alloc: id = msg_send![class!(NSString), alloc];
            if key_alloc.is_null() {
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: NSString alloc failed",
                );
                return;
            }

            let key: id =
                msg_send![key_alloc, initWithUTF8String: c"SetsCursorInBackground".as_ptr()];
            if key.is_null() {
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: NSString initWithUTF8String failed",
                );
                return;
            }

            let value: id = msg_send![class!(NSNumber), numberWithBool: YES];
            if value.is_null() {
                let _: () = msg_send![key, release];
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: NSNumber numberWithBool failed",
                );
                return;
            }

            let result = CGSSetConnectionProperty(connection_id, connection_id, key, value);
            let _: () = msg_send![key, release];

            if result == 0 {
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: enabled SetsCursorInBackground",
                );
            } else {
                crate::logging::log(
                    "CURSOR",
                    &format!(
                        "install_cursor_tracking: failed to enable SetsCursorInBackground (status={})",
                        result
                    ),
                );
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn install_cursor_tracking() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Layer 2 — Mouse-move cursor coordination
// ============================================================================

#[cfg(target_os = "macos")]
fn set_cursor_pointer() {
    // SAFETY: [NSCursor pointingHandCursor] is a singleton. set is main-thread safe.
    // Using `set` instead of `push` to override cursor rects immediately.
    unsafe {
        let cursor: id = msg_send![class!(NSCursor), pointingHandCursor];
        let _: () = msg_send![cursor, set];
    }
}

#[cfg(target_os = "macos")]
fn set_cursor_arrow() {
    // SAFETY: [NSCursor arrowCursor] is a singleton. set is main-thread safe.
    unsafe {
        let cursor: id = msg_send![class!(NSCursor), arrowCursor];
        let _: () = msg_send![cursor, set];
    }
}

/// Claim the pointer (finger) cursor for this mouse-move frame.
///
/// Call from interactive elements (buttons, links) in their `on_mouse_move`.
/// This only sets a flag — the actual cursor change happens in
/// [`apply_default_cursor`] which fires last (root element, bubble phase).
pub fn claim_cursor_pointer() {
    CURSOR_CLAIMED.with(|c| c.set(true));
}

/// Apply the final cursor for this frame.
///
/// If an inner element claimed pointer via [`claim_cursor_pointer`], set the
/// pointing-hand cursor. Otherwise set the arrow cursor. Called unconditionally
/// on every mouse move to override whatever the cursor rect (Layer 1) set.
///
/// Call from the root element's `on_mouse_move` handler. Since the root
/// fires last in bubble phase, all inner claims are already recorded.
pub fn apply_default_cursor() {
    CURSOR_CLAIMED.with(|claimed| {
        let is_claimed = claimed.get();
        claimed.set(false);

        #[cfg(target_os = "macos")]
        {
            // Call `set` unconditionally on every mouse move to override whatever
            // the cursor rect (Layer 1) set between events. `set` is immediate
            // and does not use the push/pop stack, so there is no drift.
            if is_claimed {
                set_cursor_pointer();
            } else {
                set_cursor_arrow();
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = is_claimed;
        }
    });
}
