// Platform cursor management for non-activating popup windows.
//
// GPUI's built-in cursor system (`reset_cursor_style`) only applies when the window
// is "active" (`is_window_hovered()` → `is_window_active()`). For non-activating
// `PopUp` panels (NSPanel with NonactivatingPanel style), the window is never "active",
// so GPUI never pushes cursor changes to the OS. This causes the underlying app's
// cursor to bleed through (e.g. I-beam from a terminal).
//
// Two-layer fix:
//
// Layer 1 — Cursor rects (macOS window-server level):
//   `install_cursor_tracking()` adds a `resetCursorRects` override to GPUI's content
//   view that registers an arrow cursor rect covering the entire view. The window server
//   uses cursor rects from the frontmost window, so this prevents the underlying app's
//   cursor rects (e.g. Terminal's I-beam) from bleeding through.
//
// Layer 2 — Mouse-move coordination (GPUI event level):
//   Interactive elements call `claim_cursor_pointer()` in their `on_mouse_move`.
//   The root element calls `apply_default_cursor()` last (bubble phase, outer-to-inner).
//   `apply_default_cursor()` pushes/pops the pointing-hand cursor on top of the
//   cursor-rect stack, so buttons get the finger pointer while non-interactive areas
//   keep the arrow from cursor rects.

use std::cell::Cell;

thread_local! {
    static CURSOR_CLAIMED: Cell<bool> = const { Cell::new(false) };
    /// Whether we currently have a pushed cursor on the stack (for pointer).
    static CURSOR_PUSHED: Cell<bool> = const { Cell::new(false) };
}

// ============================================================================
// Layer 1 — Cursor rect installation
// ============================================================================

/// Install cursor rect management on the main window's content view.
///
/// Adds a `resetCursorRects` method to GPUI's view class (it doesn't have one)
/// that registers an arrow cursor rect for the entire view. This tells macOS's
/// window server to use our cursor instead of the underlying window's cursor.
///
/// Safe to call multiple times — uses `Once` to ensure single installation.
#[cfg(target_os = "macos")]
pub fn install_cursor_tracking() {
    use std::sync::Once;
    static INSTALL: Once = Once::new();

    INSTALL.call_once(|| {
        // SAFETY: Main thread (called from configure_as_floating_panel).
        // All ObjC calls target standard AppKit/runtime APIs with nil-checked pointers.
        unsafe {
            let window = match crate::window_manager::get_main_window() {
                Some(w) => w,
                None => {
                    crate::logging::log(
                        "CURSOR",
                        "install_cursor_tracking: no main window, skipping",
                    );
                    return;
                }
            };

            let content_view: id = msg_send![window, contentView];
            if content_view.is_null() {
                crate::logging::log(
                    "CURSOR",
                    "install_cursor_tracking: contentView is nil, skipping",
                );
                return;
            }

            // Get the view's actual class (GPUI's GPUIView or similar)
            let view_class: *const std::ffi::c_void = msg_send![content_view, class];
            if view_class.is_null() {
                return;
            }

            // Our resetCursorRects implementation: add arrow cursor rect for entire view
            extern "C" fn reset_cursor_rects_impl(
                this: *mut std::ffi::c_void,
                _cmd: objc::runtime::Sel,
            ) {
                unsafe {
                    let this = this as id;
                    let bounds: cocoa::foundation::NSRect = msg_send![this, bounds];
                    let arrow: id = msg_send![class!(NSCursor), arrowCursor];
                    let _: () = msg_send![this, addCursorRect:bounds cursor:arrow];
                }
            }

            // Add resetCursorRects to the view's class.
            // class_addMethod returns false if the method already exists (safe no-op).
            let sel = sel!(resetCursorRects);
            let types = c"v@:"; // void return, id self, SEL _cmd
            #[allow(clippy::missing_transmute_annotations)]
            let imp: objc::runtime::Imp =
                std::mem::transmute::<_, objc::runtime::Imp>(
                    reset_cursor_rects_impl
                        as extern "C" fn(*mut std::ffi::c_void, objc::runtime::Sel),
                );
            let added = objc::runtime::class_addMethod(
                view_class as *mut objc::runtime::Class,
                sel,
                imp,
                types.as_ptr(),
            );

            // Trigger initial cursor rect evaluation
            let _: () = msg_send![window, invalidateCursorRectsForView: content_view];

            crate::logging::log(
                "CURSOR",
                &format!(
                    "Installed resetCursorRects on content view (added={})",
                    added
                ),
            );
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
fn push_cursor_pointer() {
    // SAFETY: [NSCursor pointingHandCursor] is a singleton. push is main-thread safe.
    unsafe {
        let cursor: id = msg_send![class!(NSCursor), pointingHandCursor];
        let _: () = msg_send![cursor, push];
    }
}

#[cfg(target_os = "macos")]
fn pop_cursor() {
    // SAFETY: [NSCursor pop] is a class method, main-thread safe.
    unsafe {
        let _: () = msg_send![class!(NSCursor), pop];
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
/// If an inner element claimed pointer via [`claim_cursor_pointer`], push the
/// pointing-hand cursor onto the cursor stack (on top of the cursor-rect arrow).
/// If no element claimed pointer, pop any previously pushed pointer cursor so
/// the cursor-rect arrow shows through.
///
/// Call from the root element's `on_mouse_move` handler. Since the root
/// fires last in bubble phase, all inner claims are already recorded.
pub fn apply_default_cursor() {
    CURSOR_CLAIMED.with(|claimed| {
        let is_claimed = claimed.get();
        claimed.set(false);

        CURSOR_PUSHED.with(|pushed| {
            #[cfg(target_os = "macos")]
            {
                if is_claimed && !pushed.get() {
                    // Entering a button: push pointer on top of cursor rect's arrow
                    push_cursor_pointer();
                    pushed.set(true);
                } else if !is_claimed && pushed.get() {
                    // Leaving a button: pop pointer, reveals cursor rect's arrow
                    pop_cursor();
                    pushed.set(false);
                }
                // is_claimed && pushed: still on a button, cursor already correct
                // !is_claimed && !pushed: not on a button, cursor rect's arrow is showing
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = is_claimed;
                let _ = pushed;
            }
        });
    });
}
