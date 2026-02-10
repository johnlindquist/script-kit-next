//! Clipboard operations
//!
//! Functions for copying entries back to the system clipboard.

use anyhow::{Context, Result};
use arboard::Clipboard;
use rusqlite::params;
use tracing::{debug, info};

use super::blob_store::is_blob_content;
use super::cache::refresh_entry_cache;
use super::database::get_connection;
use super::image::decode_base64_image;
use super::macos_paste::copy_blob_with_file_url;
use super::types::ContentType;

/// Copy an entry back to the clipboard
///
/// # Arguments
/// * `id` - The entry ID to copy
///
/// # Errors
/// Returns error if the entry doesn't exist or clipboard operation fails.
///
/// # Image Pasting Behavior (CleanShot-style)
///
/// For blob-format images (the current storage format), this function uses
/// CleanShot-style pasting which sets BOTH:
/// - Image data: for apps that accept images (Photoshop, Preview, etc.)
/// - File URL: for text fields, terminals, Claude Code, etc.
///
/// This allows pasting images to work naturally in any context - text fields
/// get the file path while image apps get the image data.
///
/// For legacy png: and rgba: formats, falls back to image-only pasting.
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

    match ContentType::from_str(&content_type) {
        ContentType::Text | ContentType::Link | ContentType::File | ContentType::Color => {
            let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
            clipboard
                .set_text(&content)
                .context("Failed to set clipboard text")?;
        }
        ContentType::Image => {
            // Use CleanShot-style pasting for blob images (sets both image AND file URL)
            if is_blob_content(&content) {
                debug!("Using CleanShot-style paste for blob image");
                copy_blob_with_file_url(&content)
                    .context("Failed to copy blob image with file URL")?;
            } else {
                // Legacy png: and rgba: formats - use standard image-only paste
                debug!("Using legacy image-only paste for non-blob image");
                let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
                if let Some(image_data) = decode_base64_image(&content) {
                    clipboard
                        .set_image(image_data)
                        .context("Failed to set clipboard image")?;
                } else {
                    anyhow::bail!("Failed to decode image data");
                }
            }
        }
    }

    // Update timestamp to move entry to top
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    let timestamp = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "UPDATE history SET timestamp = ? WHERE id = ?",
        params![timestamp, id],
    )?;

    info!(id = %id, "Copied entry to clipboard");

    drop(conn);
    refresh_entry_cache();

    Ok(())
}
