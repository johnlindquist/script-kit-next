//! Notes Storage Layer
//!
//! SQLite-backed persistence for notes with CRUD operations.
//! Follows the same patterns as clipboard_history.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Note, NoteId};

/// Global database connection for notes
static NOTES_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the notes database
fn get_notes_db_path() -> PathBuf {
    if cfg!(test) {
        return std::env::temp_dir()
            .join("script-kit-gpui-tests")
            .join(std::process::id().to_string())
            .join("db")
            .join("notes.sqlite");
    }

    crate::setup::get_kit_path().join("db").join("notes.sqlite")
}

/// Ensure the notes tables and virtual search table exist.
fn ensure_notes_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_deleted_at ON notes(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_notes_is_pinned ON notes(is_pinned);

        CREATE TABLE IF NOT EXISTS note_cart_items (
            id TEXT PRIMARY KEY,
            note_id TEXT NOT NULL,
            label TEXT NOT NULL DEFAULT '',
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_note_cart_items_note_id_sort
            ON note_cart_items(note_id, sort_order, updated_at DESC);

        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            content,
            content='notes',
            content_rowid='rowid'
        );
        "#,
    )
    .context("Failed to create notes tables")?;

    ensure_notes_fts_triggers(conn)?;
    Ok(())
}

/// Recreate the FTS triggers so migrations are applied even on an existing DB connection.
fn ensure_notes_fts_triggers(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS notes_ai;
        DROP TRIGGER IF EXISTS notes_ad;
        DROP TRIGGER IF EXISTS notes_au;

        CREATE TRIGGER notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;

        CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
        END;

        CREATE TRIGGER notes_au AFTER UPDATE ON notes
        WHEN OLD.title <> NEW.title OR OLD.content <> NEW.content
        BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;
        "#,
    )
    .context("Failed to create FTS triggers")?;

    Ok(())
}

/// Initialize the notes database
///
/// This function is idempotent - it's safe to call multiple times.
/// If the database is already initialized, it verifies schema and triggers
/// are up-to-date on the existing connection.
pub fn init_notes_db() -> Result<()> {
    if let Some(db) = NOTES_DB.get() {
        let conn = db
            .lock()
            .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

        ensure_notes_schema(&conn)?;
        debug!("Notes database already initialized, schema verified");
        return Ok(());
    }

    let db_path = get_notes_db_path();

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create notes db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open notes database")?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .context("Failed to enable WAL mode")?;

    ensure_notes_schema(&conn)?;

    rebuild_notes_search_index_with_conn(&conn).context("Failed to backfill notes FTS index")?;

    info!(db_path = %db_path.display(), "Notes database initialized");

    let _ = NOTES_DB.get_or_init(|| Arc::new(Mutex::new(conn)));

    Ok(())
}

/// Rebuild the FTS index so that pre-existing notes rows become searchable.
///
/// Uses the FTS5 `'rebuild'` command which drops and repopulates the index
/// from the content table. Safe to call repeatedly (idempotent).
fn rebuild_notes_search_index_with_conn(conn: &Connection) -> Result<()> {
    conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", [])
        .context("Failed to rebuild notes FTS index")?;
    info!("Rebuilt notes FTS index");
    Ok(())
}

/// Rebuild the full-text search index for notes.
///
/// Public wrapper that acquires the DB lock. Call this when you suspect the
/// FTS index is out of sync with the notes table (e.g. after a migration).
pub fn rebuild_notes_search_index() -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;
    rebuild_notes_search_index_with_conn(&conn)
}

/// Get a reference to the notes database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    NOTES_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Notes database not initialized"))
}

/// Save a note (insert or update)
pub fn save_note(note: &Note) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            is_pinned = excluded.is_pinned,
            sort_order = excluded.sort_order
        "#,
        params![
            note.id.as_str(),
            note.title,
            note.content,
            note.created_at.to_rfc3339(),
            note.updated_at.to_rfc3339(),
            note.deleted_at.map(|dt| dt.to_rfc3339()),
            note.is_pinned as i32,
            note.sort_order,
        ],
    )
    .context("Failed to save note")?;

    debug!(note_id = %note.id, title = %note.title, "Note saved");
    Ok(())
}

/// Get a note by ID
pub fn get_note(id: NoteId) -> Result<Option<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_note query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_note)
        .optional()
        .context("Failed to get note")?;

    Ok(result)
}

/// Get all active notes (not deleted), sorted by pinned first then updated_at desc
pub fn get_all_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NULL
            ORDER BY is_pinned DESC, updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect notes")?;

    debug!(count = notes.len(), "Retrieved all notes");
    Ok(notes)
}

/// Get notes in trash (soft-deleted)
pub fn get_deleted_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query deleted notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted notes")?;

    debug!(count = notes.len(), "Retrieved deleted notes");
    Ok(notes)
}

/// Sanitize a query string for FTS5 MATCH
///
/// FTS5 special characters that need escaping: * " ' ( ) : - ^
/// We wrap the query in double quotes for phrase matching and escape internal quotes.
fn sanitize_fts_query(query: &str) -> String {
    let escaped = query.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Search notes using full-text search
///
/// Uses FTS5 search when possible with a fallback to LIKE queries for robustness
/// against special characters that break FTS5 MATCH syntax.
pub fn search_notes(query: &str) -> Result<Vec<Note>> {
    if query.trim().is_empty() {
        return get_all_notes();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Try FTS search first with sanitized query
    let sanitized_query = sanitize_fts_query(query);

    // FTS5 search with BM25 ranking
    let fts_result: rusqlite::Result<Vec<Note>> = (|| {
        let mut stmt = conn.prepare(
            r#"
            SELECT n.id, n.title, n.content, n.created_at, n.updated_at,
                   n.deleted_at, n.is_pinned, n.sort_order
            FROM notes n
            INNER JOIN notes_fts fts ON n.rowid = fts.rowid
            WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
            ORDER BY bm25(notes_fts)
            LIMIT 200
            "#,
        )?;

        let notes = stmt
            .query_map(params![sanitized_query], row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    })();

    match fts_result {
        Ok(notes) => {
            debug!(query = %query, count = notes.len(), method = "fts", "Note search completed");
            Ok(notes)
        }
        Err(e) => {
            // FTS failed (possibly due to special characters), fall back to LIKE search
            debug!(
                query = %query,
                error = %e,
                method = "like_fallback",
                "FTS search failed, using LIKE fallback"
            );

            let like_pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT id, title, content, created_at, updated_at,
                           deleted_at, is_pinned, sort_order
                    FROM notes
                    WHERE deleted_at IS NULL
                      AND (title LIKE ?1 OR content LIKE ?1)
                    ORDER BY updated_at DESC
                    "#,
                )
                .context("Failed to prepare LIKE fallback query")?;

            let notes = stmt
                .query_map(params![like_pattern], row_to_note)
                .context("Failed to execute LIKE fallback search")?
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to collect LIKE fallback results")?;

            debug!(query = %query, count = notes.len(), method = "like_fallback", "Note search completed");
            Ok(notes)
        }
    }
}

/// Permanently delete a note
pub fn delete_note_permanently(id: NoteId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
        .context("Failed to delete note")?;

    info!(note_id = %id, "Note permanently deleted");
    Ok(())
}

/// Permanently delete all soft-deleted notes in a single batch operation.
pub fn delete_all_deleted_notes() -> Result<()> {
    let db = get_db()?;
    let mut conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let tx = conn
        .transaction()
        .context("Failed to start delete_all_deleted_notes transaction")?;

    let count = tx
        .execute("DELETE FROM notes WHERE deleted_at IS NOT NULL", [])
        .context("Failed to delete all soft-deleted notes")?;

    tx.commit()
        .context("Failed to commit delete_all_deleted_notes transaction")?;

    info!(deleted_count = count, "Deleted all soft-deleted notes");
    Ok(())
}

/// Prune notes deleted more than `days` ago
pub fn prune_old_deleted_notes(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    let count = conn
        .execute(
            "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted notes")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted notes");
    }

    Ok(count)
}

// ── Cart item persistence ───────────────────────────────────────────

/// Save a cart item (insert or update).
pub fn save_note_cart_item(item: &super::model::NoteCartItem) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let payload_json =
        serde_json::to_string(&item.payload).context("Failed to serialize cart item payload")?;

    conn.execute(
        r#"
        INSERT INTO note_cart_items (id, note_id, label, payload_json, created_at, updated_at, sort_order)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            payload_json = excluded.payload_json,
            updated_at = excluded.updated_at,
            sort_order = excluded.sort_order
        "#,
        params![
            item.id,
            item.note_id.as_str(),
            item.label,
            payload_json,
            item.created_at.to_rfc3339(),
            item.updated_at.to_rfc3339(),
            item.sort_order,
        ],
    )
    .context("Failed to save cart item")?;

    debug!(cart_item_id = %item.id, note_id = %item.note_id, "Cart item saved");
    Ok(())
}

/// List all cart items for a note, ordered by sort_order ascending.
pub fn list_note_cart_items(note_id: NoteId) -> Result<Vec<super::model::NoteCartItem>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, note_id, label, payload_json, created_at, updated_at, sort_order
            FROM note_cart_items
            WHERE note_id = ?1
            ORDER BY sort_order ASC, updated_at DESC
            "#,
        )
        .context("Failed to prepare list_note_cart_items query")?;

    let items = stmt
        .query_map(params![note_id.as_str()], row_to_cart_item)
        .context("Failed to query cart items")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect cart items")?;

    debug!(note_id = %note_id, count = items.len(), "Retrieved cart items");
    Ok(items)
}

/// Delete a cart item by ID.
pub fn delete_note_cart_item(item_id: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        "DELETE FROM note_cart_items WHERE id = ?1",
        params![item_id],
    )
    .context("Failed to delete cart item")?;

    info!(cart_item_id = %item_id, "Cart item deleted");
    Ok(())
}

/// Convert a database row to a NoteCartItem.
fn row_to_cart_item(row: &rusqlite::Row) -> rusqlite::Result<super::model::NoteCartItem> {
    let id: String = row.get(0)?;
    let note_id_str: String = row.get(1)?;
    let label: String = row.get(2)?;
    let payload_json: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let updated_at_str: String = row.get(5)?;
    let sort_order: i32 = row.get(6)?;

    let note_id = NoteId::parse(&note_id_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            format!("Invalid note_id UUID in note_cart_items: {note_id_str}").into(),
        )
    })?;

    let payload: super::model::NoteCartItemPayload =
        serde_json::from_str(&payload_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(super::model::NoteCartItem {
        id,
        note_id,
        label,
        payload,
        created_at,
        updated_at,
        sort_order,
    })
}

/// Convert a database row to a Note
fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let content: String = row.get(2)?;
    let created_at_str: String = row.get(3)?;
    let updated_at_str: String = row.get(4)?;
    let deleted_at_str: Option<String> = row.get(5)?;
    let is_pinned: i32 = row.get(6)?;
    let sort_order: i32 = row.get(7)?;

    let id = NoteId::parse(&id_str).unwrap_or_default();

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let deleted_at = deleted_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(Note {
        id,
        title,
        content,
        created_at,
        updated_at,
        deleted_at,
        is_pinned: is_pinned != 0,
        sort_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_token(prefix: &str) -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        format!(
            "{prefix}_{millis}_{}",
            NoteId::new().as_str().replace('-', "")
        )
    }

    #[test]
    fn test_db_path() {
        let path = get_notes_db_path();
        assert!(path.to_string_lossy().contains("notes.sqlite"));
    }

    #[test]
    fn test_search_notes_handles_special_characters() {
        init_notes_db().expect("notes db should initialize for special-character search");

        // Search with special characters should not error (even if no results)
        // These are FTS5 special characters that can break MATCH queries
        let special_queries = [
            "test@example.com", // @ symbol
            "foo*bar",          // wildcard
            "hello\"world",     // quote
            "foo:bar",          // colon (FTS column prefix syntax)
            "(test)",           // parentheses
            "test^2",           // caret (boost syntax)
            "test-query",       // hyphen (can be operator)
            "'test'",           // single quotes
            "test AND OR NOT",  // operators
        ];

        for query in special_queries {
            let result = search_notes(query);
            assert!(
                result.is_ok(),
                "Search with '{}' should not error: {:?}",
                query,
                result.err()
            );
        }
    }

    #[test]
    fn test_notes_au_trigger_has_when_guard_for_real_content_changes() {
        init_notes_db().expect("notes db should initialize before trigger inspection");

        let db = get_db().expect("notes db should be initialized");
        let conn = db.lock().expect("notes db lock should succeed");

        let trigger_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'trigger' AND name = 'notes_au'",
                [],
                |row| row.get(0),
            )
            .expect("notes_au trigger should exist");

        assert!(
            trigger_sql.contains("WHEN OLD.title <> NEW.title OR OLD.content <> NEW.content"),
            "notes_au trigger should only fire when title/content differ: {trigger_sql}"
        );
    }

    #[test]
    fn test_init_notes_db_recreates_triggers_for_existing_connection() {
        init_notes_db().expect("notes db should initialize before trigger recreation");

        let db = get_db().expect("notes db should be initialized");
        let conn = db.lock().expect("notes db lock should succeed");

        // Install a legacy unguarded trigger to simulate stale schema
        conn.execute_batch(
            r#"
            DROP TRIGGER IF EXISTS notes_au;
            CREATE TRIGGER notes_au AFTER UPDATE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, title, content)
                VALUES('delete', OLD.rowid, OLD.title, OLD.content);
                INSERT INTO notes_fts(rowid, title, content)
                VALUES (NEW.rowid, NEW.title, NEW.content);
            END;
            "#,
        )
        .expect("should install legacy notes_au trigger");
        drop(conn);

        // Re-init should verify schema and recreate triggers
        init_notes_db().expect("re-init should verify schema and recreate triggers");

        let db = get_db().expect("notes db should still be initialized");
        let conn = db.lock().expect("notes db lock should still succeed");

        let trigger_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'trigger' AND name = 'notes_au'",
                [],
                |row| row.get(0),
            )
            .expect("notes_au trigger should exist after re-init");

        assert!(
            trigger_sql.contains("WHEN OLD.title <> NEW.title OR OLD.content <> NEW.content"),
            "re-init should restore the guarded notes_au trigger: {trigger_sql}"
        );
    }

    #[test]
    fn test_search_notes_limits_fts_results_to_200() {
        init_notes_db().expect("notes db should initialize before search limit test");
        let token = unique_test_token("search_limit");
        let now = Utc::now();
        let mut note_ids = Vec::new();

        for index in 0..220 {
            let note = Note {
                id: NoteId::new(),
                title: format!("{token} title {index}"),
                content: format!("{token} content {index}"),
                created_at: now,
                updated_at: now,
                deleted_at: None,
                is_pinned: false,
                sort_order: index,
            };

            save_note(&note).expect("failed to save note for search limit test");
            note_ids.push(note.id);
        }

        let results = search_notes(&token).expect("search should succeed");

        for id in note_ids {
            delete_note_permanently(id).expect("cleanup failed for search limit test");
        }

        assert!(
            results.len() <= 200,
            "search should cap FTS results at 200, got {}",
            results.len()
        );
    }

    #[test]
    fn test_delete_all_deleted_notes_removes_soft_deleted_notes_in_batch() {
        init_notes_db().expect("notes db should initialize before batch delete test");
        let token = unique_test_token("batch_delete");
        let now = Utc::now();

        let deleted_note = Note {
            id: NoteId::new(),
            title: format!("{token} deleted"),
            content: format!("{token} deleted content"),
            created_at: now,
            updated_at: now,
            deleted_at: Some(now),
            is_pinned: false,
            sort_order: 0,
        };
        save_note(&deleted_note).expect("failed to save soft-deleted note");

        let active_note = Note {
            id: NoteId::new(),
            title: format!("{token} active"),
            content: format!("{token} active content"),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_pinned: false,
            sort_order: 1,
        };
        save_note(&active_note).expect("failed to save active note");

        delete_all_deleted_notes().expect("batch delete should succeed");

        let deleted_result = get_note(deleted_note.id).expect("query deleted note should succeed");
        let active_result = get_note(active_note.id).expect("query active note should succeed");

        delete_note_permanently(active_note.id).expect("cleanup failed for active note");

        assert!(
            deleted_result.is_none(),
            "soft-deleted note should be permanently removed by batch delete"
        );
        assert!(
            active_result.is_some(),
            "active note should not be removed by batch delete"
        );
    }

    #[test]
    fn test_rebuild_notes_search_index_recovers_desynced_rows() {
        init_notes_db().expect("notes db should initialize before FTS rebuild test");
        let token = unique_test_token("fts_rebuild");
        let now = Utc::now();

        let note = Note {
            id: NoteId::new(),
            title: format!("{token} title"),
            content: format!("{token} content"),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        };

        save_note(&note).expect("failed to save note for fts rebuild test");

        // Manually remove the FTS row to simulate a desynced index
        let db = get_db().expect("notes db should be initialized");
        let conn = db.lock().expect("notes db lock should succeed");

        conn.execute(
            r#"
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES(
                'delete',
                (SELECT rowid FROM notes WHERE id = ?1),
                ?2,
                ?3
            )
            "#,
            params![note.id.as_str(), note.title.clone(), note.content.clone()],
        )
        .expect("failed to desync notes_fts row");
        drop(conn);

        // The note should NOT be searchable while desynced
        let missing = search_notes(&token).expect("search before rebuild should succeed");
        assert!(
            missing.iter().all(|candidate| candidate.id != note.id),
            "desynced note should not be searchable before rebuild"
        );

        // Rebuild should restore the index
        rebuild_notes_search_index().expect("fts rebuild should succeed");

        let rebuilt = search_notes(&token).expect("search after rebuild should succeed");
        delete_note_permanently(note.id).expect("cleanup failed for fts rebuild test");

        assert!(
            rebuilt.iter().any(|candidate| candidate.id == note.id),
            "fts rebuild should restore existing rows into notes_fts"
        );
    }

    #[test]
    fn test_search_notes_returns_matching_note_for_special_character_content() {
        init_notes_db().expect("notes db should initialize before special-character match test");
        let token = unique_test_token("search_special_match");
        let query = format!("{token}@example.com");
        let now = Utc::now();

        let note = Note {
            id: NoteId::new(),
            title: format!("Contact {query}"),
            content: format!("Reach me at {query}"),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        };

        save_note(&note).expect("failed to save note for special character search test");

        // FTS5 index updates may lag under concurrent writes (nextest parallelism).
        // Retry briefly so the test is not flaky.
        let mut results = Vec::new();
        for _ in 0..5 {
            results = search_notes(&query).expect("search should succeed");
            if results.iter().any(|c| c.id == note.id) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        delete_note_permanently(note.id).expect("cleanup failed for special character search test");

        assert!(
            results.iter().any(|candidate| candidate.id == note.id),
            "search should return the note that contains the special-character query"
        );
    }
}
