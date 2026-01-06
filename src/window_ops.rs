//! Coalescing Window Operations Module
//!
//! Provides coalesced window resize and move operations using GPUI's Window::defer API.
//!
//! # Why Coalescing?
//!
//! During rapid UI updates (filtering, typing, state changes), multiple resize/move requests
//! can fire in quick succession. Without coalescing, each request would trigger a macOS
//! window operation, causing:
//! - Visual jitter/flicker
//! - Performance degradation from redundant system calls
//! - Potential RefCell borrow conflicts during GPUI's render cycle
//!
//! # How It Works
//!
//! 1. Callers call `queue_resize()` or `queue_move()` with their desired values
//! 2. The value is stored in a pending slot (overwrites any previous pending value)
//! 3. A `Window::defer()` callback is scheduled (only once per effect cycle)
//! 4. At the end of the effect cycle, `flush_pending_ops()` executes the final values
//!
//! This ensures only ONE window operation happens per effect cycle, using the latest values.
//!
//! # Usage
//!
//! ```rust,ignore
//! // Instead of direct resize:
//! // platform::resize_first_window_to_height(height);
//!
//! // Use coalesced queue:
//! window_ops::queue_resize(height, window, cx);
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gpui::{Bounds, Pixels, Window};

use crate::logging;
use crate::platform;
use crate::window_resize;

// ============================================================================
// Coalescing State
// ============================================================================

/// Pending resize height (if Some, a resize is pending)
static PENDING_RESIZE: Mutex<Option<f32>> = Mutex::new(None);

/// Pending move bounds (if Some, a move is pending)
static PENDING_BOUNDS: Mutex<Option<Bounds<Pixels>>> = Mutex::new(None);

/// Whether a flush has been scheduled for this effect cycle
static FLUSH_SCHEDULED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Public API
// ============================================================================

/// Queue a window resize to happen at the end of the current effect cycle.
///
/// Multiple calls within the same cycle will coalesce - only the final height is used.
///
/// # Arguments
/// * `target_height` - The desired window height in pixels
/// * `window` - The GPUI Window reference (used for defer scheduling)
/// * `cx` - The GPUI App context
///
/// # Example
/// ```rust,ignore
/// use crate::window_ops;
///
/// // In a message handler or event callback:
/// window_ops::queue_resize(500.0, window, cx);
/// ```
pub fn queue_resize(target_height: f32, window: &mut Window, cx: &mut gpui::App) {
    // Store the pending height (overwrites any previous pending value)
    *PENDING_RESIZE.lock().unwrap() = Some(target_height);

    logging::log(
        "WINDOW_OPS",
        &format!("Queued resize to height: {:.0}px", target_height),
    );

    // Schedule flush if not already scheduled
    schedule_flush(window, cx);
}

/// Queue a window move to happen at the end of the current effect cycle.
///
/// Multiple calls within the same cycle will coalesce - only the final bounds are used.
///
/// # Arguments
/// * `bounds` - The desired window bounds (position + size)
/// * `window` - The GPUI Window reference (used for defer scheduling)
/// * `cx` - The GPUI App context
///
/// # Example
/// ```rust,ignore
/// use crate::window_ops;
/// use gpui::{point, px, size, Bounds};
///
/// let bounds = Bounds {
///     origin: point(px(100.0), px(200.0)),
///     size: size(px(750.0), px(500.0)),
/// };
/// window_ops::queue_move(bounds, window, cx);
/// ```
pub fn queue_move(bounds: Bounds<Pixels>, window: &mut Window, cx: &mut gpui::App) {
    // Store the pending bounds (overwrites any previous pending value)
    *PENDING_BOUNDS.lock().unwrap() = Some(bounds);

    logging::log(
        "WINDOW_OPS",
        &format!(
            "Queued move to bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        ),
    );

    // Schedule flush if not already scheduled
    schedule_flush(window, cx);
}

/// Check if there are any pending window operations.
///
/// Useful for debugging or testing.
#[allow(dead_code)]
pub fn has_pending_ops() -> bool {
    PENDING_RESIZE.lock().unwrap().is_some() || PENDING_BOUNDS.lock().unwrap().is_some()
}

/// Clear all pending operations without executing them.
///
/// Use this when the window is being hidden/closed to avoid stale operations.
#[allow(dead_code)]
pub fn clear_pending_ops() {
    *PENDING_RESIZE.lock().unwrap() = None;
    *PENDING_BOUNDS.lock().unwrap() = None;
    FLUSH_SCHEDULED.store(false, Ordering::SeqCst);
    logging::log("WINDOW_OPS", "Cleared all pending operations");
}

// ============================================================================
// Internal Implementation
// ============================================================================

/// Schedule the flush callback if not already scheduled.
///
/// Uses Window::defer to run at the end of the current effect cycle.
fn schedule_flush(window: &mut Window, cx: &mut gpui::App) {
    // Only schedule once per effect cycle
    if !FLUSH_SCHEDULED.swap(true, Ordering::SeqCst) {
        logging::log("WINDOW_OPS", "Scheduling flush via Window::defer");

        window.defer(cx, |_window, _cx| {
            flush_pending_ops();
        });
    }
}

/// Execute all pending window operations.
///
/// Called by the deferred callback at the end of the effect cycle.
fn flush_pending_ops() {
    // Reset the scheduled flag FIRST (allows new operations to schedule a new flush)
    FLUSH_SCHEDULED.store(false, Ordering::SeqCst);

    // Execute pending resize if any
    if let Some(height) = PENDING_RESIZE.lock().unwrap().take() {
        logging::log(
            "WINDOW_OPS",
            &format!("Flushing resize to height: {:.0}px", height),
        );
        window_resize::resize_first_window_to_height(gpui::px(height));
    }

    // Execute pending move if any
    if let Some(bounds) = PENDING_BOUNDS.lock().unwrap().take() {
        logging::log(
            "WINDOW_OPS",
            &format!(
                "Flushing move to bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
            ),
        );
        platform::move_first_window_to_bounds(&bounds);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_resize_state() {
        // Clear any leftover state
        clear_pending_ops();

        // Initially no pending ops
        assert!(!has_pending_ops());

        // Manually set pending resize (simulating what queue_resize does internally)
        *PENDING_RESIZE.lock().unwrap() = Some(500.0);
        assert!(has_pending_ops());

        // Clear clears it
        clear_pending_ops();
        assert!(!has_pending_ops());
    }

    #[test]
    fn test_pending_bounds_state() {
        use gpui::{point, px, size};

        clear_pending_ops();
        assert!(!has_pending_ops());

        // Manually set pending bounds
        let bounds = Bounds {
            origin: point(px(100.0), px(200.0)),
            size: size(px(750.0), px(500.0)),
        };
        *PENDING_BOUNDS.lock().unwrap() = Some(bounds);
        assert!(has_pending_ops());

        clear_pending_ops();
        assert!(!has_pending_ops());
    }

    #[test]
    fn test_flush_clears_state() {
        clear_pending_ops();

        // Set some pending state
        *PENDING_RESIZE.lock().unwrap() = Some(700.0);
        FLUSH_SCHEDULED.store(true, Ordering::SeqCst);

        // Note: We can't test the actual flush_pending_ops() without mocking platform functions,
        // but we can verify the state management
        assert!(has_pending_ops());
        assert!(FLUSH_SCHEDULED.load(Ordering::SeqCst));

        // Clear should reset everything
        clear_pending_ops();
        assert!(!has_pending_ops());
        assert!(!FLUSH_SCHEDULED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_coalesce_multiple_resizes() {
        clear_pending_ops();

        // Simulate multiple resize requests (what happens in practice)
        *PENDING_RESIZE.lock().unwrap() = Some(400.0);
        *PENDING_RESIZE.lock().unwrap() = Some(500.0);
        *PENDING_RESIZE.lock().unwrap() = Some(600.0);

        // Only the last value should be stored
        assert_eq!(*PENDING_RESIZE.lock().unwrap(), Some(600.0));

        clear_pending_ops();
    }

    #[test]
    fn test_coalesce_multiple_moves() {
        use gpui::{point, px, size};

        clear_pending_ops();

        // Simulate multiple move requests
        let bounds1 = Bounds {
            origin: point(px(0.0), px(0.0)),
            size: size(px(750.0), px(500.0)),
        };
        let bounds2 = Bounds {
            origin: point(px(100.0), px(100.0)),
            size: size(px(750.0), px(500.0)),
        };
        let bounds3 = Bounds {
            origin: point(px(200.0), px(200.0)),
            size: size(px(750.0), px(500.0)),
        };

        *PENDING_BOUNDS.lock().unwrap() = Some(bounds1);
        *PENDING_BOUNDS.lock().unwrap() = Some(bounds2);
        *PENDING_BOUNDS.lock().unwrap() = Some(bounds3);

        // Only the last value should be stored
        let pending = PENDING_BOUNDS.lock().unwrap();
        assert!(pending.is_some());
        let bounds = pending.unwrap();
        assert_eq!(f32::from(bounds.origin.x), 200.0);
        assert_eq!(f32::from(bounds.origin.y), 200.0);

        drop(pending);
        clear_pending_ops();
    }
}
