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
