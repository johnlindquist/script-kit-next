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
use super::config::{get_max_text_content_len, get_retention_days, is_text_over_limit};
use super::database::{
    add_entry, get_connection, prune_old_entries, run_incremental_vacuum, run_wal_checkpoint,
    trim_oversize_text_entries,
};
use super::image::{compute_image_hash, decode_to_render_image, encode_image_as_png};
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
fn clipboard_monitor_loop(stop_flag: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to create clipboard instance")?;
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;
    let poll_interval = Duration::from_millis(POLL_INTERVAL_MS);

    info!(
        poll_interval_ms = POLL_INTERVAL_MS,
        "Clipboard monitor started"
    );

    loop {
        // Check if we should stop (lock-free with AtomicBool)
        if stop_flag.load(Ordering::Relaxed) {
            info!("Clipboard monitor stopping");
            break;
        }

        let start = Instant::now();

        // Check for text changes
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let is_new = match &last_text {
                    Some(last) => last != &text,
                    None => true,
                };

                if is_new {
                    debug!(text_len = text.len(), "New text detected in clipboard");
                    if is_text_over_limit(&text) {
                        let correlation_id = Uuid::new_v4().to_string();
                        warn!(
                            correlation_id = %correlation_id,
                            text_len = text.len(),
                            max_len = get_max_text_content_len(),
                            "Skipping oversized clipboard text entry"
                        );
                    } else {
                        match add_entry(&text, ContentType::Text) {
                            Ok(entry_id) => {
                                debug!(entry_id = %entry_id, "Added text entry to history");
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to add text entry to history");
                            }
                        }
                    }
                    last_text = Some(text);
                }
            }
        }

        // Check for image changes
        if let Ok(image_data) = clipboard.get_image() {
            let hash = compute_image_hash(&image_data);

            let is_new = match last_image_hash {
                Some(last) => last != hash,
                None => true,
            };

            if is_new {
                debug!(
                    width = image_data.width,
                    height = image_data.height,
                    "New image detected in clipboard"
                );

                // Encode image as compressed PNG (base64)
                if let Ok(base64_content) = encode_image_as_png(&image_data) {
                    match add_entry(&base64_content, ContentType::Image) {
                        Ok(entry_id) => {
                            // Pre-decode the image immediately so it's ready for display
                            if let Some(render_image) = decode_to_render_image(&base64_content) {
                                cache_image(&entry_id, render_image);
                                debug!(entry_id = %entry_id, "Pre-cached new image during monitoring");
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to add image entry to history");
                        }
                    }
                }
                last_image_hash = Some(hash);
            }
        }

        // Sleep for remaining time in poll interval
        let elapsed = start.elapsed();
        if elapsed < poll_interval {
            thread::sleep(poll_interval - elapsed);
        }
    }

    Ok(())
}

/// Background loop that periodically prunes old entries
fn background_prune_loop(stop_flag: Arc<AtomicBool>) {
    let prune_interval = Duration::from_secs(PRUNE_INTERVAL_SECS);
    let mut prune_count: u32 = 0;

    loop {
        // Sleep first (initial prune already happened during init)
        thread::sleep(prune_interval);

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

/// Pre-warm the image cache by decoding all cached image entries
fn prewarm_image_cache() {
    let entries = get_cached_entries(100);
    let mut decoded_count = 0;

    for entry in entries {
        if entry.content_type == ContentType::Image {
            // Skip if already cached
            if get_cached_image(&entry.id).is_some() {
                continue;
            }

            // Decode and cache
            if let Some(render_image) = decode_to_render_image(&entry.content) {
                cache_image(&entry.id, render_image);
                decoded_count += 1;
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
}
