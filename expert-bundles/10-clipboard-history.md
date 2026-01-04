🧩 Packing 8 file(s)...
📝 Files selected:
  • src/clipboard_history/clipboard.rs
  • src/clipboard_history/mod.rs
  • src/clipboard_history/image.rs
  • src/clipboard_history/monitor.rs
  • src/clipboard_history/config.rs
  • src/clipboard_history/database.rs
  • src/clipboard_history/types.rs
  • src/clipboard_history/cache.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 8
</notes>
</file_summary>

<directory_structure>
src/clipboard_history/clipboard.rs
src/clipboard_history/mod.rs
src/clipboard_history/image.rs
src/clipboard_history/monitor.rs
src/clipboard_history/config.rs
src/clipboard_history/database.rs
src/clipboard_history/types.rs
src/clipboard_history/cache.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/clipboard_history/clipboard.rs">
//! Clipboard operations
//!
//! Functions for copying entries back to the system clipboard.

use anyhow::{Context, Result};
use arboard::Clipboard;
use rusqlite::params;
use tracing::info;

use super::cache::refresh_entry_cache;
use super::database::get_connection;
use super::image::decode_base64_image;
use super::types::ContentType;

/// Copy an entry back to the clipboard
///
/// # Arguments
/// * `id` - The entry ID to copy
///
/// # Errors
/// Returns error if the entry doesn't exist or clipboard operation fails.
#[allow(dead_code)]
pub fn copy_entry_to_clipboard(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let (content, content_type): (String, String) = conn
        .query_row(
            "SELECT content, content_type FROM history WHERE id = ?",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("Entry not found")?;

    drop(conn); // Release lock before clipboard operation

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

    match ContentType::from_str(&content_type) {
        ContentType::Text => {
            clipboard
                .set_text(&content)
                .context("Failed to set clipboard text")?;
        }
        ContentType::Image => {
            if let Some(image_data) = decode_base64_image(&content) {
                clipboard
                    .set_image(image_data)
                    .context("Failed to set clipboard image")?;
            } else {
                anyhow::bail!("Failed to decode image data");
            }
        }
    }

    // Update timestamp to move entry to top
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    let timestamp = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE history SET timestamp = ? WHERE id = ?",
        params![timestamp, id],
    )?;

    info!(id = %id, "Copied entry to clipboard");

    drop(conn);
    refresh_entry_cache();

    Ok(())
}

</file>

<file path="src/clipboard_history/mod.rs">
//! Clipboard History Module
//!
//! Provides SQLite-backed clipboard history with background monitoring.
//!
//! ## Features
//! - Stores text and base64-encoded images
//! - Background polling every 500ms
//! - Time-based retention (default 30 days)
//! - Pin/unpin entries to prevent deletion
//! - Pagination support for lazy loading
//! - Time-based grouping (Today, Yesterday, This Week, etc.)
//! - OCR text storage for image entries
//!
//! ## Module Structure
//! - `types`: Core types (ContentType, TimeGroup, ClipboardEntry)
//! - `config`: Retention and text length configuration
//! - `cache`: LRU caching for images and entries
//! - `database`: SQLite operations (CRUD, migrations)
//! - `image`: Image encoding/decoding (PNG, RGBA)
//! - `monitor`: Background clipboard polling and maintenance
//! - `clipboard`: System clipboard operations

mod cache;
mod clipboard;
mod config;
mod database;
mod image;
mod monitor;
mod types;

// Re-export public API
// These exports form the public API of the clipboard_history module.
// Some may appear unused in this crate but are used by external consumers.

// Types
#[allow(unused_imports)]
pub use types::{
    classify_timestamp, group_entries_by_time, ClipboardEntry, ContentType, TimeGroup,
};

// Config
#[allow(unused_imports)]
pub use config::{
    get_max_text_content_len, get_retention_days, set_max_text_content_len, set_retention_days,
};

// Cache
pub use cache::{cache_image, get_cached_entries, get_cached_image};

// Database operations
#[allow(unused_imports)]
pub use database::{
    clear_history, get_clipboard_history, get_clipboard_history_page, get_entry_by_id,
    get_total_entry_count, pin_entry, remove_entry, trim_oversize_text_entries, unpin_entry,
    update_ocr_text,
};

// Image operations
pub use image::{decode_to_render_image, get_image_dimensions};

// Monitor/Init
#[allow(unused_imports)]
pub use monitor::{init_clipboard_history, stop_clipboard_monitoring};

// Clipboard operations
pub use clipboard::copy_entry_to_clipboard;

// Test-only exports
#[cfg(test)]
#[allow(unused_imports)]
pub use types::classify_timestamp_with_now;

</file>

<file path="src/clipboard_history/image.rs">
//! Clipboard image encoding and decoding
//!
//! Handles base64 encoding/decoding of clipboard images, including
//! PNG compression and legacy RGBA format support.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use gpui::RenderImage;
use smallvec::SmallVec;
use std::sync::Arc;
use tracing::{debug, warn};

/// Encode image data as base64 PNG string (compressed, ~90% smaller than raw RGBA)
///
/// Format: "png:{base64_encoded_png_data}"
/// The PNG format is detected by the "png:" prefix for decoding.
pub fn encode_image_as_png(image: &arboard::ImageData) -> Result<String> {
    use std::io::Cursor;

    // Create an RgbaImage from the raw bytes
    let rgba_image = image::RgbaImage::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
    .context("Failed to create RGBA image from clipboard data")?;

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    rgba_image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode image as PNG")?;

    // Base64 encode and prefix with "png:"
    let base64_data = BASE64.encode(&png_data);
    Ok(format!("png:{}", base64_data))
}

/// Encode image data as base64 raw RGBA string (legacy format, kept for compatibility)
///
/// Format: "rgba:{width}:{height}:{base64_data}"
/// This is the old format - new code should use encode_image_as_png() instead.
#[allow(dead_code)] // Kept for backward compatibility if needed
pub fn encode_image_as_base64(image: &arboard::ImageData) -> Result<String> {
    let base64_data = BASE64.encode(&image.bytes);
    Ok(format!(
        "rgba:{}:{}:{}",
        image.width, image.height, base64_data
    ))
}

/// Decode a base64 image string back to ImageData
///
/// Supports both formats:
/// - New PNG format: "png:{base64_encoded_png_data}"
/// - Legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
#[allow(dead_code)]
pub fn decode_base64_image(content: &str) -> Option<arboard::ImageData<'static>> {
    if content.starts_with("png:") {
        decode_png_to_image_data(content)
    } else if content.starts_with("rgba:") {
        decode_legacy_rgba(content)
    } else {
        warn!("Unknown clipboard image format prefix");
        None
    }
}

/// Decode PNG format: "png:{base64_encoded_png_data}"
fn decode_png_to_image_data(content: &str) -> Option<arboard::ImageData<'static>> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();

    Some(arboard::ImageData {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        bytes: rgba.into_raw().into(),
    })
}

/// Decode legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
fn decode_legacy_rgba(content: &str) -> Option<arboard::ImageData<'static>> {
    let parts: Vec<&str> = content.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "rgba" {
        return None;
    }

    let width: usize = parts[1].parse().ok()?;
    let height: usize = parts[2].parse().ok()?;
    let bytes = BASE64.decode(parts[3]).ok()?;

    Some(arboard::ImageData {
        width,
        height,
        bytes: bytes.into(),
    })
}

/// Decode a clipboard image content string to GPUI RenderImage
///
/// Supports both formats:
/// - New PNG format: "png:{base64_encoded_png_data}"
/// - Legacy RGBA format: "rgba:{width}:{height}:{base64_data}"
///
/// Returns an Arc<RenderImage> for efficient caching.
///
/// **IMPORTANT**: Call this ONCE per entry and cache the result. Do NOT
/// decode during rendering as this is expensive.
pub fn decode_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    if content.starts_with("png:") {
        decode_png_to_render_image(content)
    } else if content.starts_with("rgba:") {
        decode_rgba_to_render_image(content)
    } else {
        warn!("Invalid clipboard image format, expected png: or rgba: prefix");
        None
    }
}

/// Decode PNG format to RenderImage
fn decode_png_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let img_width = rgba.width();
    let img_height = rgba.height();

    let frame = image::Frame::new(rgba);
    let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

    debug!(
        width = img_width,
        height = img_height,
        format = "png",
        "Decoded clipboard image to RenderImage"
    );
    Some(Arc::new(render_image))
}

/// Decode legacy RGBA format to RenderImage
fn decode_rgba_to_render_image(content: &str) -> Option<Arc<RenderImage>> {
    let parts: Vec<&str> = content.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "rgba" {
        warn!("Invalid clipboard image format, expected rgba:W:H:data");
        return None;
    }

    let width: u32 = parts[1].parse().ok()?;
    let height: u32 = parts[2].parse().ok()?;
    let rgba_bytes = BASE64.decode(parts[3]).ok()?;

    let expected_bytes = (width as usize) * (height as usize) * 4;
    if rgba_bytes.len() != expected_bytes {
        warn!(
            "Clipboard image byte count mismatch: expected {}, got {}",
            expected_bytes,
            rgba_bytes.len()
        );
        return None;
    }

    let rgba_image = image::RgbaImage::from_raw(width, height, rgba_bytes)?;
    let frame = image::Frame::new(rgba_image);
    let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

    debug!(
        width,
        height,
        format = "rgba",
        "Decoded clipboard image to RenderImage"
    );
    Some(Arc::new(render_image))
}

/// Get image dimensions from content string without fully decoding
///
/// Returns (width, height) if the content is a valid image format.
/// For PNG format, reads PNG header to extract dimensions (fast, no full decode).
/// For legacy RGBA format, parses dimensions from metadata prefix.
pub fn get_image_dimensions(content: &str) -> Option<(u32, u32)> {
    if content.starts_with("png:") {
        get_png_dimensions(content)
    } else if content.starts_with("rgba:") {
        let parts: Vec<&str> = content.splitn(4, ':').collect();
        if parts.len() >= 3 {
            let width: u32 = parts[1].parse().ok()?;
            let height: u32 = parts[2].parse().ok()?;
            Some((width, height))
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract dimensions from PNG header without full decode
fn get_png_dimensions(content: &str) -> Option<(u32, u32)> {
    let base64_data = content.strip_prefix("png:")?;
    let png_bytes = BASE64.decode(base64_data).ok()?;

    let cursor = std::io::Cursor::new(&png_bytes);
    let reader = image::ImageReader::with_format(cursor, image::ImageFormat::Png);
    let (width, height) = reader.into_dimensions().ok()?;

    Some((width, height))
}

/// Compute a simple hash of image data for change detection
pub fn compute_image_hash(image: &arboard::ImageData) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image.width.hash(&mut hasher);
    image.height.hash(&mut hasher);

    // Hash first 1KB of pixels for quick comparison
    let sample_size = 1024.min(image.bytes.len());
    image.bytes[..sample_size].hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_hash_deterministic() {
        let image = arboard::ImageData {
            width: 100,
            height: 100,
            bytes: vec![0u8; 40000].into(),
        };

        let hash1 = compute_image_hash(&image);
        let hash2 = compute_image_hash(&image);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_base64_image_roundtrip_legacy() {
        let original = arboard::ImageData {
            width: 2,
            height: 2,
            bytes: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ]
            .into(),
        };

        let encoded = encode_image_as_base64(&original).expect("Should encode");
        assert!(
            encoded.starts_with("rgba:"),
            "Legacy format should have rgba: prefix"
        );
        let decoded = decode_base64_image(&encoded).expect("Should decode");

        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
        assert_eq!(original.bytes.as_ref(), decoded.bytes.as_ref());
    }

    #[test]
    fn test_png_image_roundtrip() {
        let original = arboard::ImageData {
            width: 2,
            height: 2,
            bytes: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ]
            .into(),
        };

        let encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        assert!(
            encoded.starts_with("png:"),
            "PNG format should have png: prefix"
        );

        let decoded = decode_base64_image(&encoded).expect("Should decode");

        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
        assert_eq!(original.bytes.as_ref(), decoded.bytes.as_ref());
    }

    #[test]
    fn test_png_compression_saves_space() {
        let original = arboard::ImageData {
            width: 100,
            height: 100,
            bytes: vec![128u8; 100 * 100 * 4].into(),
        };

        let png_encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        let rgba_encoded = encode_image_as_base64(&original).expect("Should encode as RGBA");

        assert!(
            png_encoded.len() < rgba_encoded.len(),
            "PNG should be smaller for 100x100 image: PNG={} vs RGBA={}",
            png_encoded.len(),
            rgba_encoded.len()
        );

        let decoded = decode_base64_image(&png_encoded).expect("Should decode");
        assert_eq!(original.width, decoded.width);
        assert_eq!(original.height, decoded.height);
    }

    #[test]
    fn test_get_image_dimensions_both_formats() {
        let original = arboard::ImageData {
            width: 100,
            height: 50,
            bytes: vec![0u8; 100 * 50 * 4].into(),
        };

        let rgba_encoded = encode_image_as_base64(&original).expect("Should encode");
        let dims = get_image_dimensions(&rgba_encoded).expect("Should get dimensions");
        assert_eq!(dims, (100, 50));

        let png_encoded = encode_image_as_png(&original).expect("Should encode as PNG");
        let dims = get_image_dimensions(&png_encoded).expect("Should get PNG dimensions");
        assert_eq!(dims, (100, 50));
    }
}

</file>

<file path="src/clipboard_history/monitor.rs">
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

</file>

<file path="src/clipboard_history/config.rs">
//! Clipboard history configuration
//!
//! Retention settings and text length limits.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Default retention period in days (entries older than this are pruned)
pub const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Default maximum number of bytes allowed for text clipboard entries.
pub const DEFAULT_MAX_TEXT_CONTENT_LEN: usize = 100_000;

/// Configured retention days (loaded from config, defaults to DEFAULT_RETENTION_DAYS)
static RETENTION_DAYS: OnceLock<u32> = OnceLock::new();

/// Configured maximum text entry length (bytes). usize::MAX means no limit.
static MAX_TEXT_CONTENT_LEN: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_TEXT_CONTENT_LEN);

/// Get the configured retention period in days
pub fn get_retention_days() -> u32 {
    *RETENTION_DAYS.get().unwrap_or(&DEFAULT_RETENTION_DAYS)
}

/// Get the configured max text length (bytes).
pub fn get_max_text_content_len() -> usize {
    MAX_TEXT_CONTENT_LEN.load(Ordering::Relaxed)
}

/// Set the retention period (call before init_clipboard_history)
#[allow(dead_code)] // Used by downstream subtasks (config)
pub fn set_retention_days(days: u32) {
    let _ = RETENTION_DAYS.set(days);
}

/// Set the max text length (bytes). Use 0 to disable the limit.
pub fn set_max_text_content_len(max_len: usize) {
    let value = if max_len == 0 { usize::MAX } else { max_len };
    MAX_TEXT_CONTENT_LEN.store(value, Ordering::Relaxed);
}

/// Check if text exceeds the configured limit
pub fn is_text_over_limit(text: &str) -> bool {
    text.len() > get_max_text_content_len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_days_default() {
        assert_eq!(DEFAULT_RETENTION_DAYS, 30);
    }

    #[test]
    fn test_text_length_limit() {
        let ok_text = "a".repeat(DEFAULT_MAX_TEXT_CONTENT_LEN);
        assert!(!is_text_over_limit(&ok_text));

        let long_text = "a".repeat(DEFAULT_MAX_TEXT_CONTENT_LEN + 1);
        assert!(is_text_over_limit(&long_text));
    }
}

</file>

<file path="src/clipboard_history/database.rs">
//! Clipboard history database operations
//!
//! SQLite database management for clipboard entries, including CRUD operations,
//! migrations, and background maintenance.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, error, info};
use uuid::Uuid;

use super::cache::{clear_all_caches, evict_image_cache, refresh_entry_cache};
use super::config::{get_max_text_content_len, get_retention_days, is_text_over_limit};
use super::types::{ClipboardEntry, ContentType};

/// Global database connection (thread-safe)
static DB_CONNECTION: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Compute SHA-256 hash of content for fast dedup lookups
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the database path (~/.sk/kit/db/clipboard-history.sqlite)
pub fn get_db_path() -> Result<PathBuf> {
    let kit_dir = PathBuf::from(shellexpand::tilde("~/.sk/kit").as_ref());
    let db_dir = kit_dir.join("db");

    if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir).context("Failed to create ~/.sk/kit/db directory")?;
    }

    Ok(db_dir.join("clipboard-history.sqlite"))
}

/// Get or create the database connection
pub fn get_connection() -> Result<Arc<Mutex<Connection>>> {
    if let Some(conn) = DB_CONNECTION.get() {
        return Ok(conn.clone());
    }

    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database at {:?}", db_path))?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .context("Failed to enable WAL mode")?;
    debug!("Enabled WAL mode for clipboard history database");

    // Enable incremental vacuum for disk space recovery after large blob deletes
    conn.execute_batch("PRAGMA auto_vacuum = INCREMENTAL;")
        .context("Failed to enable incremental auto_vacuum")?;
    debug!("Enabled incremental auto_vacuum for clipboard history database");

    // Create the table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            content_hash TEXT,
            content_type TEXT NOT NULL DEFAULT 'text',
            timestamp INTEGER NOT NULL,
            pinned INTEGER DEFAULT 0,
            ocr_text TEXT
        )",
        [],
    )
    .context("Failed to create history table")?;

    // Migration: Add ocr_text column if it doesn't exist
    let has_ocr_column: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('history') WHERE name='ocr_text'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !has_ocr_column {
        conn.execute("ALTER TABLE history ADD COLUMN ocr_text TEXT", [])
            .context("Failed to add ocr_text column")?;
        info!("Migrated clipboard history: added ocr_text column");
    }

    // Migration: Add content_hash column if it doesn't exist
    let has_hash_column: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('history') WHERE name='content_hash'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !has_hash_column {
        conn.execute("ALTER TABLE history ADD COLUMN content_hash TEXT", [])
            .context("Failed to add content_hash column")?;
        info!("Migrated clipboard history: added content_hash column");
    }

    // Create indexes for faster queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON history(timestamp DESC)",
        [],
    )
    .context("Failed to create timestamp index")?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pinned_timestamp ON history(pinned DESC, timestamp DESC)",
        [],
    )
    .context("Failed to create pinned+timestamp index")?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dedup ON history(content_type, content_hash)",
        [],
    )
    .context("Failed to create dedup index")?;

    let conn = Arc::new(Mutex::new(conn));

    if DB_CONNECTION.set(conn.clone()).is_err() {
        return Ok(DB_CONNECTION.get().unwrap().clone());
    }

    Ok(conn)
}

/// Add a new entry to clipboard history
///
/// Returns the ID of the entry (either existing or newly created).
pub fn add_entry(content: &str, content_type: ContentType) -> Result<String> {
    if content_type == ContentType::Text && is_text_over_limit(content) {
        anyhow::bail!(
            "Clipboard text exceeds max length ({} bytes)",
            get_max_text_content_len()
        );
    }

    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let timestamp = chrono::Utc::now().timestamp();
    let content_hash = compute_content_hash(content);

    // Check if entry with same hash exists (O(1) dedup via index)
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM history WHERE content_type = ? AND content_hash = ?",
            params![content_type.as_str(), &content_hash],
            |row| row.get(0),
        )
        .ok();

    if let Some(existing_id) = existing {
        conn.execute(
            "UPDATE history SET timestamp = ? WHERE id = ?",
            params![timestamp, &existing_id],
        )
        .context("Failed to update existing entry timestamp")?;
        debug!(id = %existing_id, "Updated existing clipboard entry timestamp");
        drop(conn);
        refresh_entry_cache();
        return Ok(existing_id);
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO history (id, content, content_hash, content_type, timestamp, pinned, ocr_text) VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL)",
        params![&id, content, &content_hash, content_type.as_str(), timestamp],
    )
    .context("Failed to insert clipboard entry")?;

    debug!(id = %id, content_type = content_type.as_str(), "Added clipboard entry");

    drop(conn);
    refresh_entry_cache();

    Ok(id)
}

/// Prune entries older than retention period (except pinned entries)
///
/// Returns the number of entries deleted.
pub fn prune_old_entries() -> Result<usize> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let retention_days = get_retention_days();
    let cutoff_timestamp = chrono::Utc::now().timestamp() - (retention_days as i64 * 24 * 60 * 60);

    let deleted = conn
        .execute(
            "DELETE FROM history WHERE pinned = 0 AND timestamp < ?",
            params![cutoff_timestamp],
        )
        .context("Failed to prune old entries")?;

    if deleted > 0 {
        debug!(
            deleted,
            retention_days, cutoff_timestamp, "Pruned old clipboard entries"
        );
    }

    Ok(deleted)
}

/// Remove text entries that exceed the configured max length.
///
/// Returns the number of entries deleted.
pub fn trim_oversize_text_entries() -> Result<usize> {
    let max_len = get_max_text_content_len();
    if max_len == usize::MAX {
        return Ok(0);
    }

    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let max_len_db = i64::try_from(max_len).unwrap_or(i64::MAX);
    let deleted = conn
        .execute(
            "DELETE FROM history WHERE content_type = 'text' AND length(CAST(content AS BLOB)) > ?",
            params![max_len_db],
        )
        .context("Failed to trim oversized text entries")?;

    if deleted > 0 {
        let correlation_id = Uuid::new_v4().to_string();
        info!(
            correlation_id = %correlation_id,
            deleted,
            max_len = max_len_db,
            "Trimmed oversized clipboard text entries"
        );
    }

    drop(conn);
    refresh_entry_cache();

    Ok(deleted)
}

/// Get paginated clipboard history entries
///
/// Returns entries ordered by pinned status (pinned first) then by timestamp descending.
pub fn get_clipboard_history_page(limit: usize, offset: usize) -> Vec<ClipboardEntry> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to get database connection");
            return Vec::new();
        }
    };

    let conn = match conn.lock() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to lock database connection");
            return Vec::new();
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT id, content, content_type, timestamp, pinned, ocr_text 
         FROM history 
         ORDER BY pinned DESC, timestamp DESC 
         LIMIT ? OFFSET ?",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to prepare query");
            return Vec::new();
        }
    };

    let entries = stmt
        .query_map(params![limit, offset], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: ContentType::from_str(&row.get::<_, String>(2)?),
                timestamp: row.get(3)?,
                pinned: row.get::<_, i64>(4)? != 0,
                ocr_text: row.get(5)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to query clipboard history");
            Vec::new()
        });

    debug!(
        count = entries.len(),
        limit, offset, "Retrieved clipboard history page"
    );
    entries
}

/// Get total number of entries in clipboard history
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub fn get_total_entry_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to get database connection");
            return 0;
        }
    };

    let conn = match conn.lock() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to lock database connection");
            return 0;
        }
    };

    conn.query_row("SELECT COUNT(*) FROM history", [], |row| {
        row.get::<_, i64>(0)
    })
    .map(|c| c as usize)
    .unwrap_or_else(|e| {
        error!(error = %e, "Failed to count clipboard entries");
        0
    })
}

/// Get clipboard history entries (convenience wrapper)
pub fn get_clipboard_history(limit: usize) -> Vec<ClipboardEntry> {
    get_clipboard_history_page(limit, 0)
}

/// Pin a clipboard entry to prevent LRU eviction
pub fn pin_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("UPDATE history SET pinned = 1 WHERE id = ?", params![id])
        .context("Failed to pin entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Pinned clipboard entry");

    drop(conn);
    refresh_entry_cache();

    Ok(())
}

/// Unpin a clipboard entry
pub fn unpin_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("UPDATE history SET pinned = 0 WHERE id = ?", params![id])
        .context("Failed to unpin entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Unpinned clipboard entry");

    drop(conn);
    refresh_entry_cache();

    Ok(())
}

/// Remove a single entry from clipboard history
pub fn remove_entry(id: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute("DELETE FROM history WHERE id = ?", params![id])
        .context("Failed to remove entry")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    info!(id = %id, "Removed clipboard entry");

    drop(conn);

    evict_image_cache(id);
    refresh_entry_cache();

    Ok(())
}

/// Clear all clipboard history
pub fn clear_history() -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    conn.execute("DELETE FROM history", [])
        .context("Failed to clear history")?;

    info!("Cleared all clipboard history");

    drop(conn);

    clear_all_caches();

    Ok(())
}

/// Update OCR text for an entry (async OCR results)
#[allow(dead_code)] // Used by downstream subtasks (OCR)
pub fn update_ocr_text(id: &str, text: &str) -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let affected = conn
        .execute(
            "UPDATE history SET ocr_text = ? WHERE id = ?",
            params![text, id],
        )
        .context("Failed to update OCR text")?;

    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }

    debug!(id = %id, text_len = text.len(), "Updated OCR text for clipboard entry");

    drop(conn);

    refresh_entry_cache();

    Ok(())
}

/// Get entry by ID
#[allow(dead_code)] // Used by downstream subtasks (UI, OCR)
pub fn get_entry_by_id(id: &str) -> Option<ClipboardEntry> {
    let conn = get_connection().ok()?;
    let conn = conn.lock().ok()?;

    conn.query_row(
        "SELECT id, content, content_type, timestamp, pinned, ocr_text FROM history WHERE id = ?",
        params![id],
        |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: ContentType::from_str(&row.get::<_, String>(2)?),
                timestamp: row.get(3)?,
                pinned: row.get::<_, i64>(4)? != 0,
                ocr_text: row.get(5)?,
            })
        },
    )
    .ok()
}

/// Run incremental vacuum to reclaim disk space
pub fn run_incremental_vacuum() -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    conn.execute_batch("PRAGMA incremental_vacuum(100);")
        .context("Incremental vacuum failed")?;
    debug!("Incremental vacuum completed");

    Ok(())
}

/// Run WAL checkpoint (passive mode, doesn't block writers)
pub fn run_wal_checkpoint() -> Result<()> {
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")
        .context("WAL checkpoint failed")?;
    debug!("WAL checkpoint completed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex as StdMutex;

    /// Test-only override for database path
    static TEST_DB_PATH: OnceLock<StdMutex<Option<PathBuf>>> = OnceLock::new();

    #[cfg(test)]
    fn set_test_db_path(path: Option<PathBuf>) {
        let lock = TEST_DB_PATH.get_or_init(|| StdMutex::new(None));
        if let Ok(mut guard) = lock.lock() {
            *guard = path;
        }
    }

    #[cfg(test)]
    fn get_test_db_path() -> Option<PathBuf> {
        TEST_DB_PATH
            .get()
            .and_then(|m| m.lock().ok())
            .and_then(|guard| guard.clone())
    }

    #[test]
    fn test_db_path_format() {
        let expected_filename = "clipboard-history.sqlite";
        let kit_dir = PathBuf::from(shellexpand::tilde("~/.sk/kit").as_ref());
        let expected_path = kit_dir.join("db").join(expected_filename);

        assert!(expected_path.to_string_lossy().contains(expected_filename));
        assert!(expected_path.to_string_lossy().contains(".sk/kit/db"));
    }

    #[test]
    fn test_db_path_with_override() {
        let temp_path = PathBuf::from("/tmp/test-clipboard.db");
        set_test_db_path(Some(temp_path.clone()));

        let retrieved = get_test_db_path();
        assert_eq!(retrieved, Some(temp_path));

        set_test_db_path(None);
    }

    #[test]
    fn test_compute_content_hash_deterministic() {
        let content = "Hello, World!";
        let hash1 = compute_content_hash(content);
        let hash2 = compute_content_hash(content);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_compute_content_hash_different_content() {
        let hash1 = compute_content_hash("Hello");
        let hash2 = compute_content_hash("World");
        assert_ne!(
            hash1, hash2,
            "Different content should have different hashes"
        );
    }

    #[test]
    fn test_compute_content_hash_format() {
        let hash = compute_content_hash("test");
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Hash should be lowercase hex"
        );
    }

    #[test]
    fn test_add_entry_returns_id() {
        fn assert_returns_result_string<F>(_: F)
        where
            F: Fn(&str, ContentType) -> Result<String>,
        {
        }
        assert_returns_result_string(add_entry);
    }
}

</file>

<file path="src/clipboard_history/types.rs">
//! Core types for clipboard history
//!
//! Contains the main data types: ContentType, TimeGroup, and ClipboardEntry.

use chrono::{Datelike, Local, NaiveDate, TimeZone};

/// Content types for clipboard entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "image" => ContentType::Image,
            _ => ContentType::Text,
        }
    }
}

/// Time grouping for clipboard entries (like Raycast)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub enum TimeGroup {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    Older,
}

impl TimeGroup {
    /// Get display name for UI labels
    #[allow(dead_code)] // Used by downstream subtasks (UI)
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeGroup::Today => "Today",
            TimeGroup::Yesterday => "Yesterday",
            TimeGroup::ThisWeek => "This Week",
            TimeGroup::LastWeek => "Last Week",
            TimeGroup::ThisMonth => "This Month",
            TimeGroup::Older => "Older",
        }
    }

    /// Order for sorting groups (lower = earlier in list)
    #[allow(dead_code)] // Used by downstream subtasks (UI)
    pub fn sort_order(&self) -> u8 {
        match self {
            TimeGroup::Today => 0,
            TimeGroup::Yesterday => 1,
            TimeGroup::ThisWeek => 2,
            TimeGroup::LastWeek => 3,
            TimeGroup::ThisMonth => 4,
            TimeGroup::Older => 5,
        }
    }
}

/// A single clipboard history entry
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    /// OCR text extracted from images (None for text entries or pending OCR)
    #[allow(dead_code)] // Used by downstream subtasks (OCR, UI)
    pub ocr_text: Option<String>,
}

/// Classify a Unix timestamp into a TimeGroup using local timezone
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub fn classify_timestamp(timestamp: i64) -> TimeGroup {
    classify_timestamp_with_now(timestamp, Local::now())
}

/// Internal testable version of classify_timestamp that accepts a "now" parameter
/// This avoids DST-related flakiness in tests by allowing fixed reference times
pub fn classify_timestamp_with_now<Tz: chrono::TimeZone>(
    timestamp: i64,
    now: chrono::DateTime<Tz>,
) -> TimeGroup {
    let today = now.date_naive();
    let entry_date = match Local.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.date_naive(),
        _ => return TimeGroup::Older,
    };

    // Check Today
    if entry_date == today {
        return TimeGroup::Today;
    }

    // Check Yesterday
    if let Some(yesterday) = today.pred_opt() {
        if entry_date == yesterday {
            return TimeGroup::Yesterday;
        }
    }

    // Calculate week boundaries
    // Week starts on Monday (ISO 8601)
    let days_since_monday = today.weekday().num_days_from_monday();
    let this_week_start = today - chrono::Duration::days(days_since_monday as i64);
    let last_week_start = this_week_start - chrono::Duration::days(7);

    // Check This Week (not including today/yesterday which are handled above)
    if entry_date >= this_week_start && entry_date < today {
        return TimeGroup::ThisWeek;
    }

    // Check Last Week
    if entry_date >= last_week_start && entry_date < this_week_start {
        return TimeGroup::LastWeek;
    }

    // Check This Month
    let this_month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    if entry_date >= this_month_start {
        return TimeGroup::ThisMonth;
    }

    TimeGroup::Older
}

/// Group entries by time period
///
/// Returns a vector of (TimeGroup, Vec<ClipboardEntry>) tuples,
/// sorted by time group order (Today first, Older last).
/// Entries within each group maintain their original order.
#[allow(dead_code)] // Used by downstream subtasks (UI)
pub fn group_entries_by_time(
    entries: Vec<ClipboardEntry>,
) -> Vec<(TimeGroup, Vec<ClipboardEntry>)> {
    use std::collections::HashMap;

    let mut groups: HashMap<TimeGroup, Vec<ClipboardEntry>> = HashMap::new();

    for entry in entries {
        let group = classify_timestamp(entry.timestamp);
        groups.entry(group).or_default().push(entry);
    }

    // Sort groups by their display order
    let mut result: Vec<(TimeGroup, Vec<ClipboardEntry>)> = groups.into_iter().collect();
    result.sort_by_key(|(group, _)| group.sort_order());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::Image.as_str(), "image");
        assert_eq!(ContentType::from_str("text"), ContentType::Text);
        assert_eq!(ContentType::from_str("image"), ContentType::Image);
        assert_eq!(ContentType::from_str("unknown"), ContentType::Text);
    }

    #[test]
    fn test_time_group_display_names() {
        assert_eq!(TimeGroup::Today.display_name(), "Today");
        assert_eq!(TimeGroup::Yesterday.display_name(), "Yesterday");
        assert_eq!(TimeGroup::ThisWeek.display_name(), "This Week");
        assert_eq!(TimeGroup::LastWeek.display_name(), "Last Week");
        assert_eq!(TimeGroup::ThisMonth.display_name(), "This Month");
        assert_eq!(TimeGroup::Older.display_name(), "Older");
    }

    #[test]
    fn test_time_group_sort_order() {
        assert!(TimeGroup::Today.sort_order() < TimeGroup::Yesterday.sort_order());
        assert!(TimeGroup::Yesterday.sort_order() < TimeGroup::ThisWeek.sort_order());
        assert!(TimeGroup::ThisWeek.sort_order() < TimeGroup::LastWeek.sort_order());
        assert!(TimeGroup::LastWeek.sort_order() < TimeGroup::ThisMonth.sort_order());
        assert!(TimeGroup::ThisMonth.sort_order() < TimeGroup::Older.sort_order());
    }

    #[test]
    fn test_classify_timestamp_today() {
        // Use a fixed reference date (Wed, Jan 15, 2025 at noon UTC) to avoid DST flakiness
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let same_day_timestamp = fixed_now.timestamp();

        assert_eq!(
            classify_timestamp_with_now(same_day_timestamp, fixed_now),
            TimeGroup::Today
        );
    }

    #[test]
    fn test_classify_timestamp_yesterday() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let yesterday_timestamp = chrono::Utc
            .with_ymd_and_hms(2025, 1, 14, 12, 0, 0)
            .unwrap()
            .timestamp();

        assert_eq!(
            classify_timestamp_with_now(yesterday_timestamp, fixed_now),
            TimeGroup::Yesterday
        );
    }

    #[test]
    fn test_classify_timestamp_very_old() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let old_timestamp = chrono::Utc
            .with_ymd_and_hms(2024, 10, 7, 12, 0, 0)
            .unwrap()
            .timestamp();

        assert_eq!(
            classify_timestamp_with_now(old_timestamp, fixed_now),
            TimeGroup::Older
        );
    }

    #[test]
    fn test_classify_timestamp_this_week() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 17, 12, 0, 0).unwrap();
        let this_week_timestamp = chrono::Utc
            .with_ymd_and_hms(2025, 1, 15, 12, 0, 0)
            .unwrap()
            .timestamp();

        assert_eq!(
            classify_timestamp_with_now(this_week_timestamp, fixed_now),
            TimeGroup::ThisWeek
        );
    }

    #[test]
    fn test_classify_timestamp_last_week() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let last_week_timestamp = chrono::Utc
            .with_ymd_and_hms(2025, 1, 8, 12, 0, 0)
            .unwrap()
            .timestamp();

        assert_eq!(
            classify_timestamp_with_now(last_week_timestamp, fixed_now),
            TimeGroup::LastWeek
        );
    }

    #[test]
    fn test_classify_timestamp_this_month() {
        let fixed_now = chrono::Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let this_month_timestamp = chrono::Utc
            .with_ymd_and_hms(2025, 1, 2, 12, 0, 0)
            .unwrap()
            .timestamp();

        assert_eq!(
            classify_timestamp_with_now(this_month_timestamp, fixed_now),
            TimeGroup::ThisMonth
        );
    }

    #[test]
    fn test_group_entries_by_time() {
        let today_ts = chrono::Utc
            .with_ymd_and_hms(2025, 1, 15, 12, 0, 0)
            .unwrap()
            .timestamp();
        let yesterday_ts = chrono::Utc
            .with_ymd_and_hms(2025, 1, 14, 12, 0, 0)
            .unwrap()
            .timestamp();
        let old_ts = chrono::Utc
            .with_ymd_and_hms(2024, 10, 7, 12, 0, 0)
            .unwrap()
            .timestamp();

        let entries = vec![
            ClipboardEntry {
                id: "1".to_string(),
                content: "today".to_string(),
                content_type: ContentType::Text,
                timestamp: today_ts,
                pinned: false,
                ocr_text: None,
            },
            ClipboardEntry {
                id: "2".to_string(),
                content: "yesterday".to_string(),
                content_type: ContentType::Text,
                timestamp: yesterday_ts,
                pinned: false,
                ocr_text: None,
            },
            ClipboardEntry {
                id: "3".to_string(),
                content: "old".to_string(),
                content_type: ContentType::Text,
                timestamp: old_ts,
                pinned: false,
                ocr_text: None,
            },
        ];

        let grouped = group_entries_by_time(entries);

        assert!(!grouped.is_empty(), "Should have at least one group");

        for i in 1..grouped.len() {
            assert!(
                grouped[i - 1].0.sort_order() <= grouped[i].0.sort_order(),
                "Groups should be sorted by sort_order"
            );
        }

        let total_entries: usize = grouped.iter().map(|(_, entries)| entries.len()).sum();
        assert_eq!(total_entries, 3, "All entries should be grouped");
    }
}

</file>

<file path="src/clipboard_history/cache.rs">
//! Clipboard history caching
//!
//! LRU caching for decoded images and clipboard entries.

use gpui::RenderImage;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::debug;

use super::database::get_clipboard_history_page;
use super::types::ClipboardEntry;

/// Maximum number of decoded images to keep in memory (LRU eviction)
/// Each image can be 1-4MB, so 100 images = ~100-400MB max memory
pub const MAX_IMAGE_CACHE_ENTRIES: usize = 100;

/// Maximum entries to cache in memory for fast access
pub const MAX_CACHED_ENTRIES: usize = 500;

/// Global image cache for decoded RenderImages (thread-safe)
/// Key: entry ID, Value: decoded RenderImage
/// Uses LRU eviction to cap memory usage at ~100-400MB (100 images max)
static IMAGE_CACHE: OnceLock<Mutex<LruCache<String, Arc<RenderImage>>>> = OnceLock::new();

/// Cached clipboard entries to avoid re-fetching from SQLite on each view open
/// Updated whenever a new entry is added
static ENTRY_CACHE: OnceLock<Mutex<Vec<ClipboardEntry>>> = OnceLock::new();

/// Timestamp of last cache update
static CACHE_UPDATED: OnceLock<Mutex<i64>> = OnceLock::new();

/// Get the global image cache, initializing if needed
pub fn get_image_cache() -> &'static Mutex<LruCache<String, Arc<RenderImage>>> {
    IMAGE_CACHE.get_or_init(|| {
        let cap = NonZeroUsize::new(MAX_IMAGE_CACHE_ENTRIES).expect("cache size must be non-zero");
        Mutex::new(LruCache::new(cap))
    })
}

/// Get the global entry cache, initializing if needed
pub fn get_entry_cache() -> &'static Mutex<Vec<ClipboardEntry>> {
    ENTRY_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Initialize the cache timestamp tracker
pub fn init_cache_timestamp() {
    let _ = CACHE_UPDATED.set(Mutex::new(0));
}

/// Get cached image by entry ID (updates LRU order)
pub fn get_cached_image(id: &str) -> Option<Arc<RenderImage>> {
    get_image_cache().lock().ok()?.get(id).cloned()
}

/// Cache a decoded image (with LRU eviction at MAX_IMAGE_CACHE_ENTRIES limit)
pub fn cache_image(id: &str, image: Arc<RenderImage>) {
    if let Ok(mut cache) = get_image_cache().lock() {
        // LruCache automatically evicts oldest entry when capacity is exceeded
        let evicted = cache.len() >= cache.cap().get();
        cache.put(id.to_string(), image);
        if evicted {
            debug!(
                id = %id,
                cache_size = cache.len(),
                max_size = MAX_IMAGE_CACHE_ENTRIES,
                "Cached decoded image (evicted oldest)"
            );
        } else {
            debug!(id = %id, cache_size = cache.len(), "Cached decoded image");
        }
    }
}

/// Get cached clipboard entries (faster than querying SQLite)
/// Falls back to SQLite if cache is empty
pub fn get_cached_entries(limit: usize) -> Vec<ClipboardEntry> {
    if let Ok(cache) = get_entry_cache().lock() {
        if !cache.is_empty() {
            let result: Vec<_> = cache.iter().take(limit).cloned().collect();
            debug!(
                count = result.len(),
                cached = true,
                "Retrieved clipboard entries from cache"
            );
            return result;
        }
    }
    // Fall back to database
    get_clipboard_history_page(limit, 0)
}

/// Invalidate the entry cache (called when entries change)
pub fn invalidate_entry_cache() {
    if let Ok(mut cache) = get_entry_cache().lock() {
        cache.clear();
    }
}

/// Refresh the entry cache from database
pub fn refresh_entry_cache() {
    let entries = get_clipboard_history_page(MAX_CACHED_ENTRIES, 0);
    if let Ok(mut cache) = get_entry_cache().lock() {
        *cache = entries;
        debug!(count = cache.len(), "Refreshed entry cache");
    }
    if let Some(updated) = CACHE_UPDATED.get() {
        if let Ok(mut ts) = updated.lock() {
            *ts = chrono::Utc::now().timestamp();
        }
    }
}

/// Evict a single entry from the image cache
pub fn evict_image_cache(id: &str) {
    if let Some(cache) = IMAGE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.pop(id);
            debug!(id = %id, "Evicted image from cache");
        }
    }
}

/// Clear all caches (entry + image)
pub fn clear_all_caches() {
    invalidate_entry_cache();
    if let Some(cache) = IMAGE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.clear();
            debug!("Cleared image cache");
        }
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 8 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~14.4K (14,399 exact)
  Total Chars: 61,366 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
      4.2K - src/clipboard_history/database.rs
      2.8K - src/clipboard_history/image.rs
      2.6K - src/clipboard_history/types.rs
      2.2K - src/clipboard_history/monitor.rs
      1.1K - src/clipboard_history/cache.rs
       507 - src/clipboard_history/clipboard.rs
       504 - src/clipboard_history/config.rs
       493 - src/clipboard_history/mod.rs
