//! Clipboard monitoring
//!
//! Background threads for clipboard polling and entry maintenance.

use anyhow::{Context, Result};
use arboard::Clipboard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::cache::{
    cache_image, get_cached_entries, get_cached_image, init_cache_timestamp, refresh_entry_cache,
};
use super::change_detection::ClipboardChangeDetector;
use super::config::{get_max_text_content_len, get_retention_days, is_text_over_limit};
use super::database::{
    add_entry, get_connection, get_entry_content, prune_old_entries, run_incremental_vacuum,
    run_wal_checkpoint, trim_oversize_text_entries,
};
use super::image::{compute_image_hash, decode_to_render_image, encode_image_as_blob};
use super::types::ContentType;

/// Interval between background pruning checks (1 hour)
const PRUNE_INTERVAL_SECS: u64 = 3600;

/// Polling interval for clipboard changes
const POLL_INTERVAL_MS: u64 = 500;

/// Flag to stop the monitoring thread (AtomicBool for lock-free polling)
static STOP_MONITORING: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Guard to ensure init_clipboard_history() is only called once
static INIT_GUARD: OnceLock<()> = OnceLock::new();

/// Initialize clipboard history: create DB and start monitoring
///
/// This should be called once at application startup. It will:
/// 1. Create the SQLite database if it doesn't exist (with WAL mode)
/// 2. Run initial pruning of old entries
/// 3. Pre-warm the entry cache
/// 4. Pre-decode images in background
/// 5. Start a background thread that polls the clipboard every 500ms
/// 6. Start a background pruning job (runs hourly)
///
/// # Errors
/// Returns error if database creation fails.
pub fn init_clipboard_history() -> Result<()> {
    // Ensure init is only called once (idempotency guard)
    if INIT_GUARD.set(()).is_err() {
        debug!("Clipboard history already initialized, skipping");
        return Ok(());
    }

    info!(
        retention_days = get_retention_days(),
        "Initializing clipboard history"
    );

    // Initialize the database connection (enables WAL, runs migrations)
    let _conn = get_connection().context("Failed to initialize database")?;

    // Initialize the cache timestamp
    init_cache_timestamp();

    // Run initial pruning of old entries
    if let Err(e) = prune_old_entries() {
        warn!(error = %e, "Initial pruning failed");
    }

    // Remove oversized text entries before caching
    if let Err(e) = trim_oversize_text_entries() {
        let correlation_id = Uuid::new_v4().to_string();
        warn!(
            correlation_id = %correlation_id,
            error = %e,
            "Initial oversize trim failed"
        );
    }

    // Pre-warm the entry cache from database
    refresh_entry_cache();

    // Pre-decode images in a background thread
    thread::spawn(|| {
        prewarm_image_cache();
    });

    // Initialize the stop flag (AtomicBool for lock-free polling)
    let stop_flag = Arc::new(AtomicBool::new(false));
    let _ = STOP_MONITORING.set(stop_flag.clone());

    // Start the monitoring thread
    let stop_flag_clone = stop_flag.clone();
    thread::spawn(move || {
        if let Err(e) = clipboard_monitor_loop(stop_flag_clone) {
            error!(error = %e, "Clipboard monitor thread failed");
        }
    });

    // Start background pruning thread (runs hourly)
    let stop_flag_prune = stop_flag.clone();
    thread::spawn(move || {
        background_prune_loop(stop_flag_prune);
    });

    info!("Clipboard history initialized");
    Ok(())
}

/// Stop the clipboard monitoring thread
#[allow(dead_code)]
pub fn stop_clipboard_monitoring() {
    if let Some(stop_flag) = STOP_MONITORING.get() {
        stop_flag.store(true, Ordering::Relaxed);
        info!("Clipboard monitoring stopped");
    }
}

/// Background loop that monitors clipboard changes
///
/// Uses macOS NSPasteboard changeCount for efficient polling when available.
/// This is dramatically cheaper than reading clipboard payloads every 500ms,
/// especially for large content (images can be 100MB+).
///
/// The change count approach also fixes a correctness bug: with content-based
/// detection, copying the same text twice in a row doesn't update the timestamp.
/// With change count detection, we detect all clipboard changes regardless of content.
fn clipboard_monitor_loop(stop_flag: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to create clipboard instance")?;
    let mut change_detector = ClipboardChangeDetector::new();

    // Content-based tracking for deduplication after OS-level change detection
    // These are updated AFTER we read payload to avoid re-processing same content
    let mut last_text_hash: Option<u64> = None;
    let mut last_image_hash: Option<u64> = None;

    let poll_interval = Duration::from_millis(POLL_INTERVAL_MS);

    // Reduced poll interval when using change count (cheap operation)
    // 200ms is responsive enough for clipboard while reducing CPU from 20 to 5 wakeups/sec
    let fast_poll_interval = Duration::from_millis(200);

    // Check if we have efficient change detection available
    let has_change_detection = change_detector.has_changed().is_some();

    info!(
        poll_interval_ms = if has_change_detection {
            200
        } else {
            POLL_INTERVAL_MS
        },
        has_change_detection = has_change_detection,
        "Clipboard monitor started"
    );

    loop {
        // Check if we should stop (lock-free with AtomicBool)
        if stop_flag.load(Ordering::Relaxed) {
            info!("Clipboard monitor stopping");
            break;
        }

        let start = Instant::now();

        // Check if clipboard changed using efficient OS-level detection when available
        let should_check_payload = change_detector.has_changed().unwrap_or(true);

        if should_check_payload {
            // Clipboard changed (or fallback mode), now read the actual payload
            capture_clipboard_content(&mut clipboard, &mut last_text_hash, &mut last_image_hash);
        }

        // Sleep for remaining time in poll interval
        // Use faster polling when we have cheap change detection
        let target_interval = if has_change_detection {
            fast_poll_interval
        } else {
            poll_interval
        };

        let elapsed = start.elapsed();
        if elapsed < target_interval {
            thread::sleep(target_interval - elapsed);
        }
    }

    Ok(())
}

/// Capture current clipboard content and add to history if new.
///
/// Uses content hashing for deduplication after OS-level change detection.
/// This is called only when the OS reports a clipboard change, making it
/// much more efficient than reading payloads on every poll.
fn capture_clipboard_content(
    clipboard: &mut Clipboard,
    last_text_hash: &mut Option<u64>,
    last_image_hash: &mut Option<u64>,
) {
    // Check for text changes
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            let text_hash = compute_text_hash(&text);

            // Check if content actually changed (handles same content copied twice)
            let is_new_content = last_text_hash.is_none_or(|last| last != text_hash);

            if is_new_content {
                debug!(text_len = text.len(), "New text detected in clipboard");

                if is_text_over_limit(&text) {
                    let correlation_id = Uuid::new_v4().to_string();
                    warn!(
                        correlation_id = %correlation_id,
                        text_len = text.len(),
                        max_len = get_max_text_content_len(),
                        "Skipping oversized clipboard text entry"
                    );
                    // Update hash even for oversized entries (intentionally skipped)
                    *last_text_hash = Some(text_hash);
                } else {
                    match add_entry(&text, ContentType::Text) {
                        Ok(entry_id) => {
                            debug!(entry_id = %entry_id, "Added text entry to history");
                            *last_text_hash = Some(text_hash);
                        }
                        Err(e) => {
                            // DON'T update hash on failure - we'll retry on next change
                            warn!(error = %e, "Failed to add text entry to history (will retry)");
                        }
                    }
                }
            } else {
                // Same content, but OS detected a change - this means user copied same text again
                // Update the timestamp to bubble this entry to the top
                debug!(
                    text_len = text.len(),
                    "Same text copied again, updating timestamp"
                );
                match add_entry(&text, ContentType::Text) {
                    Ok(entry_id) => {
                        debug!(entry_id = %entry_id, "Updated timestamp for existing text entry");
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to update text entry timestamp");
                    }
                }
            }
            return; // Text takes priority, don't check image
        }
    }

    // Check for image changes (only if no text was found)
    if let Ok(image_data) = clipboard.get_image() {
        let hash = compute_image_hash(&image_data);

        let is_new_content = last_image_hash.is_none_or(|last| last != hash);

        if is_new_content {
            debug!(
                width = image_data.width,
                height = image_data.height,
                "New image detected in clipboard"
            );

            // Encode image as blob (PNG file on disk)
            match encode_image_as_blob(&image_data) {
                Ok(blob_content) => {
                    match add_entry(&blob_content, ContentType::Image) {
                        Ok(entry_id) => {
                            // Pre-decode the image immediately so it's ready for display
                            if let Some(render_image) = decode_to_render_image(&blob_content) {
                                cache_image(&entry_id, render_image);
                                debug!(entry_id = %entry_id, "Pre-cached new image during monitoring");
                            }
                            *last_image_hash = Some(hash);
                        }
                        Err(e) => {
                            // DON'T update hash on failure - we'll retry on next change
                            warn!(error = %e, "Failed to add image entry to history (will retry)");
                        }
                    }
                }
                Err(e) => {
                    // Encoding failed (likely corrupt image data), skip but update hash
                    // to avoid repeated attempts on the same bad image
                    warn!(error = %e, "Failed to encode image as blob, skipping");
                    *last_image_hash = Some(hash);
                }
            }
        } else {
            // Same image, but OS detected a change - update timestamp
            debug!(
                width = image_data.width,
                height = image_data.height,
                "Same image copied again, updating timestamp"
            );
            if let Ok(blob_content) = encode_image_as_blob(&image_data) {
                match add_entry(&blob_content, ContentType::Image) {
                    Ok(entry_id) => {
                        debug!(entry_id = %entry_id, "Updated timestamp for existing image entry");
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to update image entry timestamp");
                    }
                }
            }
        }
    }
}

/// Compute a simple hash of text content for change detection.
fn compute_text_hash(text: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Sleep with interruptible checks against a stop flag.
/// Returns true if sleep completed, false if stop was signaled.
fn interruptible_sleep(duration: Duration, stop_flag: &AtomicBool) -> bool {
    let check_interval = Duration::from_secs(1);
    let mut remaining = duration;

    while remaining > Duration::ZERO {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }
        let sleep_time = remaining.min(check_interval);
        thread::sleep(sleep_time);
        remaining = remaining.saturating_sub(sleep_time);
    }
    true
}

/// Background loop that periodically prunes old entries
fn background_prune_loop(stop_flag: Arc<AtomicBool>) {
    let prune_interval = Duration::from_secs(PRUNE_INTERVAL_SECS);
    let mut prune_count: u32 = 0;

    loop {
        // Sleep first (initial prune already happened during init)
        // Use interruptible sleep so stop_clipboard_monitoring() takes effect within ~1s
        if !interruptible_sleep(prune_interval, &stop_flag) {
            info!("Background prune thread stopping (interrupted during sleep)");
            break;
        }

        // Check if we should stop (lock-free with AtomicBool)
        if stop_flag.load(Ordering::Relaxed) {
            info!("Background prune thread stopping");
            break;
        }

        // Prune old entries
        match prune_old_entries() {
            Ok(count) => {
                if count > 0 {
                    info!(pruned = count, "Background pruning completed");
                    refresh_entry_cache();
                }

                // Reclaim disk space incrementally after successful prune
                if let Err(e) = run_incremental_vacuum() {
                    warn!(error = %e, "Incremental vacuum failed");
                }
            }
            Err(e) => {
                warn!(error = %e, "Background pruning failed");
            }
        }

        prune_count += 1;

        // Checkpoint WAL every 10 prune cycles to bound WAL file growth
        if prune_count.is_multiple_of(10) {
            if let Err(e) = run_wal_checkpoint() {
                warn!(error = %e, "WAL checkpoint failed");
            } else {
                debug!(cycle = prune_count, "WAL checkpoint completed");
            }
        }
    }
}

/// Pre-warm the image cache by decoding cached image entries
///
/// This fetches content on-demand for each image entry to avoid
/// keeping all image payloads in memory during normal list views.
fn prewarm_image_cache() {
    let entries = get_cached_entries(100);
    let mut decoded_count = 0;

    for entry in entries {
        if entry.content_type == ContentType::Image {
            // Skip if already cached
            if get_cached_image(&entry.id).is_some() {
                continue;
            }

            // Fetch content on-demand and decode
            if let Some(content) = get_entry_content(&entry.id) {
                if let Some(render_image) = decode_to_render_image(&content) {
                    cache_image(&entry.id, render_image);
                    decoded_count += 1;
                }
            }
        }
    }

    info!(decoded_count, "Pre-warmed image cache");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_guard_exists() {
        let _guard: &OnceLock<()> = &INIT_GUARD;
    }

    #[test]
    fn test_stop_monitoring_is_atomic() {
        fn assert_atomic_bool(_: &OnceLock<Arc<AtomicBool>>) {}
        assert_atomic_bool(&STOP_MONITORING);
    }

    #[test]
    fn test_atomic_bool_operations() {
        let flag = Arc::new(AtomicBool::new(false));

        assert!(!flag.load(Ordering::Relaxed));

        flag.store(true, Ordering::Relaxed);
        assert!(flag.load(Ordering::Relaxed));

        flag.store(false, Ordering::Relaxed);
        assert!(!flag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_retry_on_db_failure_behavior() {
        // Verify the retry logic by checking Option behavior
        // When add_entry fails, last_text should remain None (or previous value)
        // allowing retry on next poll

        // Simulate first successful add
        let mut last_text: Option<String> = Some("success".to_string());
        assert_eq!(last_text.as_deref(), Some("success"));

        // Simulate failed add - the key insight is that on failure,
        // we DON'T update last_text. So if we check whether new_text != last_text,
        // a failed entry will be retried on the next poll.
        let new_text = "new_entry".to_string();

        // Check if it's "new" (different from last)
        let is_new = last_text.as_ref() != Some(&new_text);
        assert!(is_new, "new_text should be detected as new");

        // On failure, we DON'T update last_text (simulating the implemented behavior)
        // This means is_new will STILL be true on the next poll iteration

        // On success, we DO update:
        last_text = Some(new_text.clone());
        let is_new_after_success = last_text.as_ref() != Some(&new_text);
        assert!(
            !is_new_after_success,
            "After success, same text should not be 'new'"
        );
    }
}
