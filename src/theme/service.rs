#![allow(dead_code)]
//! Global Theme Service
//!
//! Provides a singleton theme watcher that broadcasts changes to all windows,
//! replacing per-window theme watchers. This eliminates duplicate watchers
//! and ensures consistent theme updates across the entire application.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::theme::service::ensure_theme_service;
//!
//! // Call once at app startup or before opening any window
//! ensure_theme_service(cx);
//! ```
//!
//! The service will:
//! 1. Watch ~/.scriptkit/kit/theme.json for changes
//! 2. Sync gpui-component theme when changes are detected
//! 3. Notify all registered windows to re-render
//!
//! # Architecture
//!
//! - Uses AtomicBool to ensure only one watcher runs
//! - Uses the WindowRegistry to notify all windows
//! - Polls for changes every 200ms (same as previous per-window watchers)

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use gpui::{App, Timer};
use tracing::{debug, info, warn};

use crate::watcher::ThemeWatcher;
use crate::windows;

const FAST_POLL_MS: u64 = 200;
const MEDIUM_POLL_MS: u64 = 500;
const SLOW_POLL_MS: u64 = 2000;
const FAST_POLL_IDLE_CUTOFF: u64 = 5;
const MEDIUM_POLL_IDLE_CUTOFF: u64 = 10;

/// Flag to track if the theme service is running
static THEME_SERVICE_RUNNING: AtomicBool = AtomicBool::new(false);

/// Global theme revision counter.
/// Incremented each time theme.json changes.
/// Views use this to invalidate cached theme-derived values (like box shadows).
static THEME_REVISION: AtomicU64 = AtomicU64::new(1);

/// Get the current theme revision.
///
/// Views should compare this against a stored value to detect theme changes
/// and recompute cached theme-derived values (like box shadows).
///
/// # Example
///
/// ```rust,ignore
/// let rev = crate::theme::service::theme_revision();
/// if self.theme_rev_seen != rev {
///     self.theme_rev_seen = rev;
///     self.cached_box_shadows = Self::compute_box_shadows();
/// }
/// ```
pub fn theme_revision() -> u64 {
    THEME_REVISION.load(Ordering::Relaxed)
}

/// Increment the theme revision.
/// Called internally when theme.json changes are detected.
fn bump_theme_revision() {
    let old = THEME_REVISION.fetch_add(1, Ordering::SeqCst);
    debug!(old_revision = old, new_revision = old + 1, "Theme revision bumped");
}

/// Ensure the global theme service is running.
///
/// This is idempotent - calling it multiple times is safe and will only
/// start one watcher. The watcher runs until the application shuts down.
///
/// # Arguments
/// * `cx` - The GPUI App context
pub fn ensure_theme_service(cx: &mut App) {
    // Use swap to atomically check and set in one operation
    if THEME_SERVICE_RUNNING.swap(true, Ordering::SeqCst) {
        // Already running
        return;
    }

    info!("Starting global theme service");

    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        let (mut watcher, rx) = ThemeWatcher::new();

        if let Err(error) = watcher.start() {
            warn!(error = ?error, "Failed to start theme file watcher");
            THEME_SERVICE_RUNNING.store(false, Ordering::SeqCst);
            return;
        }

        info!("Theme file watcher started successfully");

        // Adaptive polling: starts at 200ms, increases to 2s when idle
        let mut idle_count = 0u32;
        loop {
            // Adaptive polling: 200ms when active, up to 2000ms when idle
            let poll_interval = if u64::from(idle_count) < FAST_POLL_IDLE_CUTOFF {
                FAST_POLL_MS
            } else if u64::from(idle_count) < MEDIUM_POLL_IDLE_CUTOFF {
                MEDIUM_POLL_MS
            } else {
                SLOW_POLL_MS
            };
            Timer::after(std::time::Duration::from_millis(poll_interval)).await;

            if rx.try_recv().is_ok() {
                idle_count = 0; // Reset on activity
                info!("Theme changed, syncing to all windows");

                // Reload the theme cache FIRST (before syncing)
                // This ensures get_cached_theme() returns fresh data
                crate::theme::reload_theme_cache();

                let update_result = cx.update(|cx| {
                    // Re-sync gpui-component theme with updated Script Kit theme
                    crate::theme::sync_gpui_component_theme(cx);

                    // Bump revision so views know to update cached values
                    bump_theme_revision();

                    // Notify all registered windows to re-render
                    windows::notify_all_windows(cx);
                });

                // If the update failed, the app may be shutting down
                if let Err(error) = update_result {
                    warn!(error = ?error, "App context gone, stopping theme service");
                    break;
                }
            } else {
                idle_count = idle_count.saturating_add(1);
            }
        }

        THEME_SERVICE_RUNNING.store(false, Ordering::SeqCst);
        info!("Theme service stopped");
    })
    .detach();
}

/// Check if the theme service is currently running.
///
/// Mainly useful for debugging/testing.
pub fn is_theme_service_running() -> bool {
    THEME_SERVICE_RUNNING.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_service_flag() {
        // Reset flag for test
        THEME_SERVICE_RUNNING.store(false, Ordering::SeqCst);

        assert!(!is_theme_service_running());

        // Manually set flag (since we can't run actual service in unit test)
        THEME_SERVICE_RUNNING.store(true, Ordering::SeqCst);
        assert!(is_theme_service_running());

        // Clean up
        THEME_SERVICE_RUNNING.store(false, Ordering::SeqCst);
    }

    #[test]
    fn test_theme_revision_starts_at_one() {
        // Revision starts at 1 (not 0) so initial comparison fails
        let rev = theme_revision();
        assert!(rev >= 1);
    }

    #[test]
    fn test_bump_increments_revision() {
        let before = theme_revision();
        bump_theme_revision();
        let after = theme_revision();
        assert!(after > before);
    }
}
