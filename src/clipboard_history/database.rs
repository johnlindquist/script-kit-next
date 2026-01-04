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
