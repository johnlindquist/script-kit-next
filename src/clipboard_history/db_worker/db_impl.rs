//! Database operation implementations for the worker thread

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use tracing::{debug, error, info};

use crate::clipboard_history::types::{ClipboardEntry, ClipboardEntryMeta, ContentType};

#[allow(clippy::too_many_arguments)]
pub fn add_or_touch_impl(
    conn: &Connection,
    content: &str,
    content_type: ContentType,
    content_hash: &str,
    text_preview: Option<String>,
    image_width: Option<u32>,
    image_height: Option<u32>,
    byte_size: usize,
) -> Result<String> {
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Check if entry with same hash exists (O(1) dedup via index)
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM history WHERE content_type = ? AND content_hash = ?",
            params![content_type.as_str(), content_hash],
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
        return Ok(existing_id);
    }

    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO history (id, content, content_hash, content_type, timestamp, pinned, ocr_text, text_preview, image_width, image_height, byte_size)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL, ?6, ?7, ?8, ?9)",
        params![&id, content, content_hash, content_type.as_str(), timestamp, text_preview, image_width, image_height, byte_size as i64],
    )
    .context("Failed to insert clipboard entry")?;

    debug!(id = %id, content_type = content_type.as_str(), "Added clipboard entry");
    Ok(id)
}

pub fn get_content_impl(conn: &Connection, id: &str) -> Option<String> {
    conn.query_row(
        "SELECT content FROM history WHERE id = ?",
        params![id],
        |row| row.get(0),
    )
    .ok()
}

pub fn get_entry_impl(conn: &Connection, id: &str) -> Option<ClipboardEntry> {
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
                source_app_name: None,
                source_app_bundle_id: None,
            })
        },
    )
    .ok()
}

pub fn get_meta_impl(conn: &Connection, limit: usize, offset: usize) -> Vec<ClipboardEntryMeta> {
    let mut stmt = match conn.prepare(
        "SELECT id, content_type, timestamp, pinned, text_preview, image_width, image_height, byte_size, ocr_text
         FROM history ORDER BY pinned DESC, timestamp DESC LIMIT ? OFFSET ?",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to prepare metadata query");
            return Vec::new();
        }
    };

    stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(ClipboardEntryMeta {
            id: row.get(0)?,
            content_type: ContentType::from_str(&row.get::<_, String>(1)?),
            timestamp: row.get(2)?,
            pinned: row.get::<_, i64>(3)? != 0,
            text_preview: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            image_width: row.get::<_, Option<i64>>(5)?.map(|v| v as u32),
            image_height: row.get::<_, Option<i64>>(6)?.map(|v| v as u32),
            byte_size: row.get::<_, Option<i64>>(7)?.unwrap_or(0) as usize,
            ocr_text: row.get(8)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn get_page_impl(conn: &Connection, limit: usize, offset: usize) -> Vec<ClipboardEntry> {
    let mut stmt = match conn.prepare(
        "SELECT id, content, content_type, timestamp, pinned, ocr_text
         FROM history ORDER BY pinned DESC, timestamp DESC LIMIT ? OFFSET ?",
    ) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to prepare query");
            return Vec::new();
        }
    };

    stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(ClipboardEntry {
            id: row.get(0)?,
            content: row.get(1)?,
            content_type: ContentType::from_str(&row.get::<_, String>(2)?),
            timestamp: row.get(3)?,
            pinned: row.get::<_, i64>(4)? != 0,
            ocr_text: row.get(5)?,
            source_app_name: None,
            source_app_bundle_id: None,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn get_count_impl(conn: &Connection) -> usize {
    conn.query_row("SELECT COUNT(*) FROM history", [], |row| {
        row.get::<_, i64>(0)
    })
    .map(|c| c as usize)
    .unwrap_or(0)
}

pub fn pin_impl(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn
        .execute("UPDATE history SET pinned = 1 WHERE id = ?", params![id])
        .context("Failed to pin entry")?;
    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }
    info!(id = %id, "Pinned clipboard entry");
    Ok(())
}

pub fn unpin_impl(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn
        .execute("UPDATE history SET pinned = 0 WHERE id = ?", params![id])
        .context("Failed to unpin entry")?;
    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }
    info!(id = %id, "Unpinned clipboard entry");
    Ok(())
}

pub fn remove_impl(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn
        .execute("DELETE FROM history WHERE id = ?", params![id])
        .context("Failed to remove entry")?;
    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }
    info!(id = %id, "Removed clipboard entry");
    Ok(())
}

pub fn clear_impl(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM history", [])
        .context("Failed to clear history")?;
    info!("Cleared all clipboard history");
    Ok(())
}

pub fn prune_impl(conn: &Connection, cutoff_timestamp_ms: i64) -> Result<usize> {
    let deleted = conn
        .execute(
            "DELETE FROM history WHERE pinned = 0 AND timestamp < ?",
            params![cutoff_timestamp_ms],
        )
        .context("Failed to prune old entries")?;
    if deleted > 0 {
        debug!(deleted, cutoff_timestamp_ms, "Pruned old clipboard entries");
    }
    Ok(deleted)
}

pub fn trim_oversized_impl(conn: &Connection, max_len: usize) -> Result<usize> {
    if max_len == usize::MAX {
        return Ok(0);
    }
    let max_len_db = i64::try_from(max_len).unwrap_or(i64::MAX);
    let deleted = conn
        .execute(
            "DELETE FROM history WHERE content_type = 'text' AND length(CAST(content AS BLOB)) > ?",
            params![max_len_db],
        )
        .context("Failed to trim oversized text entries")?;
    if deleted > 0 {
        info!(
            deleted,
            max_len = max_len_db,
            "Trimmed oversized text entries"
        );
    }
    Ok(deleted)
}

pub fn update_ocr_impl(conn: &Connection, id: &str, text: &str) -> Result<()> {
    let affected = conn
        .execute(
            "UPDATE history SET ocr_text = ? WHERE id = ?",
            params![text, id],
        )
        .context("Failed to update OCR text")?;
    if affected == 0 {
        anyhow::bail!("Entry not found: {}", id);
    }
    debug!(id = %id, text_len = text.len(), "Updated OCR text");
    Ok(())
}

pub fn vacuum_impl(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA incremental_vacuum(100);")
        .context("Vacuum failed")?;
    debug!("Incremental vacuum completed");
    Ok(())
}

pub fn checkpoint_impl(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")
        .context("WAL checkpoint failed")?;
    debug!("WAL checkpoint completed");
    Ok(())
}
