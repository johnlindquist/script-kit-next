//! AI Chat Storage Layer
//!
//! SQLite-backed persistence for AI chats with CRUD operations and FTS5 search.
//! Follows the same patterns as src/notes/storage.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Chat, ChatId, ChatSource, Message, MessageRole};

/// Global database connection for AI chats
static AI_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the AI chats database (~/.scriptkit/db/ai-chats.sqlite)
fn get_ai_db_path() -> PathBuf {
    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit"));

    kit_dir.join("db").join("ai-chats.sqlite")
}

/// Initialize the AI chats database
///
/// This function is idempotent - calling it multiple times is safe and will
/// succeed if the database is already initialized.
pub fn init_ai_db() -> Result<()> {
    // Check if already initialized - return early (idempotent behavior)
    if AI_DB.get().is_some() {
        debug!("AI database already initialized, skipping");
        return Ok(());
    }

    let db_path = get_ai_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create AI db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open AI chats database")?;

    // Enable WAL mode for better write performance and concurrency
    // This matches the pattern used in notes/storage.rs
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .context("Failed to enable WAL mode for AI database")?;

    // Set synchronous=NORMAL for good performance with WAL safety
    conn.execute_batch("PRAGMA synchronous=NORMAL;")
        .context("Failed to set synchronous mode for AI database")?;

    // Enable foreign keys enforcement - required for CASCADE to work
    // Without this, FOREIGN KEY constraints are parsed but not enforced!
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .context("Failed to enable foreign_keys for AI database")?;

    // Set busy timeout to avoid "database is locked" errors under concurrent access
    // 1000ms is a reasonable default for a desktop app
    conn.busy_timeout(std::time::Duration::from_millis(1000))
        .context("Failed to set busy_timeout for AI database")?;

    debug!(db_path = %db_path.display(), "AI database PRAGMAs configured: WAL, synchronous=NORMAL, foreign_keys=ON, busy_timeout=1000ms");

    // Create tables
    conn.execute_batch(
        r#"
        -- Chats table
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT 'New Chat',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            model_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'ai_window'
        );

        CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_chats_deleted_at ON chats(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_chats_provider ON chats(provider);
        -- Note: idx_chats_source is created in the migration below to handle existing DBs

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            tokens_used INTEGER,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);

        -- Full-text search support for chats (searches titles and message content)
        CREATE VIRTUAL TABLE IF NOT EXISTS chats_fts USING fts5(
            title,
            content='chats',
            content_rowid='rowid'
        );

        -- Full-text search for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            content='messages',
            content_rowid='rowid'
        );

        -- Recreate triggers atomically to avoid race conditions in concurrent test processes
        -- Using IMMEDIATE transaction to prevent concurrent DROP+CREATE conflicts
        BEGIN IMMEDIATE;

        -- Drop old triggers first (IF NOT EXISTS won't update existing triggers)
        -- This is needed to migrate from AFTER UPDATE to AFTER UPDATE OF column
        DROP TRIGGER IF EXISTS chats_ai;
        DROP TRIGGER IF EXISTS chats_ad;
        DROP TRIGGER IF EXISTS chats_au;
        DROP TRIGGER IF EXISTS messages_ai;
        DROP TRIGGER IF EXISTS messages_ad;
        DROP TRIGGER IF EXISTS messages_au;

        -- Triggers to keep chat FTS in sync
        CREATE TRIGGER chats_ai AFTER INSERT ON chats BEGIN
            INSERT INTO chats_fts(rowid, title)
            VALUES (NEW.rowid, NEW.title);
        END;

        CREATE TRIGGER chats_ad AFTER DELETE ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title)
            VALUES('delete', OLD.rowid, OLD.title);
        END;

        -- CRITICAL: Only trigger on title changes (not updated_at changes)
        -- save_message_internal updates chats.updated_at on every message,
        -- which would cause unnecessary DELETE+INSERT into chats_fts without this fix
        CREATE TRIGGER chats_au AFTER UPDATE OF title ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title)
            VALUES('delete', OLD.rowid, OLD.title);
            INSERT INTO chats_fts(rowid, title)
            VALUES (NEW.rowid, NEW.title);
        END;

        -- Triggers to keep message FTS in sync
        CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
            INSERT INTO messages_fts(rowid, content)
            VALUES (NEW.rowid, NEW.content);
        END;

        CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content)
            VALUES('delete', OLD.rowid, OLD.content);
        END;

        -- CRITICAL: Only trigger on content changes (not tokens_used changes etc.)
        CREATE TRIGGER messages_au AFTER UPDATE OF content ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content)
            VALUES('delete', OLD.rowid, OLD.content);
            INSERT INTO messages_fts(rowid, content)
            VALUES (NEW.rowid, NEW.content);
        END;

        COMMIT;
        "#,
    )
    .context("Failed to create AI tables")?;

    // Migration: Add source column if it doesn't exist (for existing databases)
    // SQLite's ADD COLUMN is simple and doesn't require complex migrations
    let has_source: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('chats') WHERE name = 'source'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_source {
        conn.execute(
            "ALTER TABLE chats ADD COLUMN source TEXT NOT NULL DEFAULT 'ai_window'",
            [],
        )
        .context("Failed to add source column to chats table")?;
        info!("Migrated AI database: added source column to chats table");
    }

    // Always ensure the source index exists (handles both new and migrated DBs)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chats_source ON chats(source)",
        [],
    )
    .context("Failed to create source index")?;

    info!(db_path = %db_path.display(), "AI chats database initialized");

    // Try to set the global connection. If another thread beat us to it
    // (race condition), that's fine - just return success (idempotent).
    if AI_DB.set(Arc::new(Mutex::new(conn))).is_err() {
        debug!("AI database was initialized by another thread, using existing connection");
    }

    Ok(())
}

/// Get a reference to the AI database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    AI_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("AI database not initialized"))
}

// ============================================================================
// Chat Operations
// ============================================================================

/// Create a new chat
pub fn create_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO chats (id, title, created_at, updated_at, deleted_at, model_id, provider, source)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.created_at.to_rfc3339(),
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
            chat.source.as_str(),
        ],
    )
    .context("Failed to create chat")?;

    debug!(chat_id = %chat.id, title = %chat.title, "Chat created");
    Ok(())
}

/// Update an existing chat
pub fn update_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        UPDATE chats
        SET title = ?2, updated_at = ?3, deleted_at = ?4, model_id = ?5, provider = ?6, source = ?7
        WHERE id = ?1
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
            chat.source.as_str(),
        ],
    )
    .context("Failed to update chat")?;

    debug!(chat_id = %chat.id, "Chat updated");
    Ok(())
}

/// Update chat title
pub fn update_chat_title(chat_id: &ChatId, title: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![chat_id.as_str(), title, now],
    )
    .context("Failed to update chat title")?;

    debug!(chat_id = %chat_id, title = %title, "Chat title updated");
    Ok(())
}

/// Get a chat by ID
pub fn get_chat(id: &ChatId) -> Result<Option<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider, source
            FROM chats
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_chat query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_chat)
        .optional()
        .context("Failed to get chat")?;

    Ok(result)
}

/// Get all active chats (not deleted), sorted by updated_at desc
pub fn get_all_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider, source
            FROM chats
            WHERE deleted_at IS NULL
            ORDER BY updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect chats")?;

    debug!(count = chats.len(), "Retrieved all chats");
    Ok(chats)
}

/// Get chats in trash (soft-deleted)
pub fn get_deleted_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider, source
            FROM chats
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query deleted chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted chats")?;

    debug!(count = chats.len(), "Retrieved deleted chats");
    Ok(chats)
}

/// Soft delete a chat
pub fn delete_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to soft delete chat")?;

    info!(chat_id = %chat_id, "Chat soft deleted");
    Ok(())
}

/// Restore a chat from trash
pub fn restore_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = NULL, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to restore chat")?;

    info!(chat_id = %chat_id, "Chat restored from trash");
    Ok(())
}

/// Permanently delete a chat and all its messages
pub fn delete_chat_permanently(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Delete messages first (foreign key constraint)
    conn.execute(
        "DELETE FROM messages WHERE chat_id = ?1",
        params![chat_id.as_str()],
    )
    .context("Failed to delete chat messages")?;

    conn.execute("DELETE FROM chats WHERE id = ?1", params![chat_id.as_str()])
        .context("Failed to delete chat")?;

    info!(chat_id = %chat_id, "Chat permanently deleted");
    Ok(())
}

/// Sanitize a query string for FTS5 MATCH.
/// FTS5 has special characters that can cause parse errors.
/// This escapes or removes problematic characters.
fn sanitize_fts_query(query: &str) -> String {
    // FTS5 special characters that need escaping: * " ' ( ) : - ^
    // For simplicity, we'll wrap the query in double quotes for phrase matching
    // and escape any internal double quotes.
    let escaped = query.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Search chats by title or message content
///
/// Searches both chat titles and message content using FTS5 when possible,
/// with a fallback to LIKE queries for robustness.
pub fn search_chats(query: &str) -> Result<Vec<Chat>> {
    if query.trim().is_empty() {
        return get_all_chats();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Try FTS search first, fall back to LIKE on error
    let sanitized_query = sanitize_fts_query(query);

    // Attempt FTS search with corrected aliases
    let fts_result: rusqlite::Result<Vec<Chat>> = (|| {
        let mut stmt = conn.prepare(
            r#"
            SELECT DISTINCT c.id, c.title, c.created_at, c.updated_at,
                   c.deleted_at, c.model_id, c.provider, c.source
            FROM chats c
            LEFT JOIN chats_fts fts ON c.rowid = fts.rowid
            LEFT JOIN messages m ON c.id = m.chat_id
            LEFT JOIN messages_fts mfts ON m.rowid = mfts.rowid
            WHERE c.deleted_at IS NULL
              AND (fts MATCH ?1 OR mfts MATCH ?1)
            ORDER BY c.updated_at DESC
            "#,
        )?;

        let chats = stmt
            .query_map(params![sanitized_query], row_to_chat)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(chats)
    })();

    match fts_result {
        Ok(chats) => {
            debug!(query = %query, count = chats.len(), method = "fts", "Chat search completed");
            Ok(chats)
        }
        Err(e) => {
            // FTS failed (possibly due to special characters or other issues)
            // Fall back to simple LIKE search on title
            debug!(error = %e, query = %query, "FTS search failed, falling back to LIKE");

            let like_pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT DISTINCT c.id, c.title, c.created_at, c.updated_at,
                           c.deleted_at, c.model_id, c.provider, c.source
                    FROM chats c
                    LEFT JOIN messages m ON c.id = m.chat_id
                    WHERE c.deleted_at IS NULL
                      AND (c.title LIKE ?1 OR m.content LIKE ?1)
                    ORDER BY c.updated_at DESC
                    "#,
                )
                .context("Failed to prepare LIKE search query")?;

            let chats = stmt
                .query_map(params![like_pattern], row_to_chat)
                .context("Failed to execute LIKE search")?
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to collect LIKE search results")?;

            debug!(query = %query, count = chats.len(), method = "like", "Chat search completed (fallback)");
            Ok(chats)
        }
    }
}

// ============================================================================
// Message Operations
// ============================================================================

/// Save a message
pub fn save_message(message: &Message) -> Result<()> {
    save_message_internal(message, true)
}

/// Save a message without updating the chat's updated_at timestamp.
/// Used for mock data insertion where we want to preserve historical dates.
fn save_message_without_update(message: &Message) -> Result<()> {
    save_message_internal(message, false)
}

/// Internal message save with optional chat timestamp update
///
/// Uses a transaction to ensure atomicity - either both the message insert
/// and chat timestamp update succeed, or both are rolled back.
/// This also reduces fsync overhead by committing once instead of twice.
fn save_message_internal(message: &Message, update_chat_timestamp: bool) -> Result<()> {
    let db = get_db()?;
    let mut conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Wrap both operations in a single transaction for:
    // 1. Atomicity: both succeed or both fail
    // 2. Performance: one fsync instead of two autocommit fsyncs
    let tx = conn.transaction().context("Failed to start transaction")?;

    tx.execute(
        r#"
        INSERT INTO messages (id, chat_id, role, content, created_at, tokens_used)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            tokens_used = excluded.tokens_used
        "#,
        params![
            message.id,
            message.chat_id.as_str(),
            message.role.as_str(),
            message.content,
            message.created_at.to_rfc3339(),
            message.tokens_used,
        ],
    )
    .context("Failed to save message")?;

    // Update the chat's updated_at timestamp (unless explicitly skipped for mock data)
    if update_chat_timestamp {
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "UPDATE chats SET updated_at = ?2 WHERE id = ?1",
            params![message.chat_id.as_str(), now],
        )
        .context("Failed to update chat timestamp")?;
    }

    tx.commit()
        .context("Failed to commit message transaction")?;

    debug!(
        message_id = %message.id,
        chat_id = %message.chat_id,
        role = %message.role,
        "Message saved"
    );
    Ok(())
}

/// Delete a single message by ID
pub fn delete_message(message_id: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM messages WHERE id = ?1", params![message_id])
        .context("Failed to delete message")?;

    debug!(message_id = %message_id, "Message deleted");
    Ok(())
}

/// Get all messages for a chat, ordered by creation time
pub fn get_chat_messages(chat_id: &ChatId) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .context("Failed to prepare get_chat_messages query")?;

    let messages = stmt
        .query_map(params![chat_id.as_str()], row_to_message)
        .context("Failed to query messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect messages")?;

    debug!(chat_id = %chat_id, count = messages.len(), "Retrieved chat messages");
    Ok(messages)
}

/// Get the last N messages for a chat
pub fn get_recent_messages(chat_id: &ChatId, limit: usize) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .context("Failed to prepare get_recent_messages query")?;

    let mut messages: Vec<Message> = stmt
        .query_map(params![chat_id.as_str(), limit as i64], row_to_message)
        .context("Failed to query recent messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect recent messages")?;

    // Reverse to get chronological order
    messages.reverse();

    Ok(messages)
}

/// Get total token usage for a chat
pub fn get_chat_token_usage(chat_id: &ChatId) -> Result<u64> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens_used), 0) FROM messages WHERE chat_id = ?1",
            params![chat_id.as_str()],
            |row| row.get(0),
        )
        .context("Failed to get token usage")?;

    Ok(total as u64)
}

/// Get chat count (active only)
pub fn get_chat_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chats WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("Failed to count chats")?;

    Ok(count as usize)
}

/// Prune chats deleted more than `days` ago
pub fn prune_old_deleted_chats(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    // Get IDs of chats to delete
    let chat_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1")
            .context("Failed to prepare prune query")?;

        let results = stmt
            .query_map(params![cutoff.to_rfc3339()], |row| row.get(0))
            .context("Failed to query chats to prune")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect chat IDs")?;
        results
    };

    // Delete messages for these chats
    for chat_id in &chat_ids {
        conn.execute("DELETE FROM messages WHERE chat_id = ?1", params![chat_id])
            .context("Failed to delete messages for pruned chat")?;
    }

    // Delete the chats
    let count = conn
        .execute(
            "DELETE FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted chats")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted chats");
    }

    Ok(count)
}

// ============================================================================
// Row Converters
// ============================================================================

/// Convert a database row to a Chat
fn row_to_chat(row: &rusqlite::Row) -> rusqlite::Result<Chat> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let created_at_str: String = row.get(2)?;
    let updated_at_str: String = row.get(3)?;
    let deleted_at_str: Option<String> = row.get(4)?;
    let model_id: String = row.get(5)?;
    let provider: String = row.get(6)?;
    let source_str: String = row.get(7)?;

    let id = ChatId::parse(&id_str).unwrap_or_default();

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

    let source = ChatSource::parse(&source_str);

    Ok(Chat {
        id,
        title,
        created_at,
        updated_at,
        deleted_at,
        model_id,
        provider,
        source,
    })
}

/// Convert a database row to a Message
fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let chat_id_str: String = row.get(1)?;
    let role_str: String = row.get(2)?;
    let content: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let tokens_used: Option<u32> = row.get(5)?;

    let chat_id = ChatId::parse(&chat_id_str).unwrap_or_default();

    let role = MessageRole::parse(&role_str).unwrap_or(MessageRole::User);

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Message {
        id,
        chat_id,
        role,
        content,
        created_at,
        tokens_used,
        images: Vec::new(),
    })
}

// ============================================================================
// Mock Data for Testing
// ============================================================================

/// Insert comprehensive mock chat data for visual testing.
/// Creates many chats across different time periods with long conversations
/// to verify search functionality and scrolling behavior.
#[allow(clippy::useless_vec)] // vec! is more readable for mock data definitions
pub fn insert_mock_data() -> Result<()> {
    use chrono::Duration;

    // First, ensure the database is initialized
    if AI_DB.get().is_none() {
        init_ai_db()?;
    }

    let now = Utc::now();

    // Helper to create a chat with messages
    let mut total_chats = 0;
    let mut total_messages = 0;

    // ========================================================================
    // TODAY - 5 chats
    // ========================================================================

    // Chat 1: Long conversation about Python (TODAY - scrollable)
    let chat1_id = ChatId::new();
    create_chat(&Chat {
        id: chat1_id,
        title: "Python Best Practices Discussion".to_string(),
        created_at: now - Duration::hours(2),
        updated_at: now - Duration::minutes(10),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    let python_messages = vec![
        (MessageRole::User, "What are some Python best practices I should follow?"),
        (MessageRole::Assistant, "Great question! Here are the key Python best practices:\n\n## 1. Follow PEP 8 Style Guide\n\n```python\n# Good\ndef calculate_total(items: list[Item]) -> float:\n    return sum(item.price for item in items)\n\n# Bad\ndef calculateTotal(Items):\n    total=0\n    for i in Items:total+=i.price\n    return total\n```\n\n## 2. Use Type Hints\n\nType hints improve code readability and enable better IDE support:\n\n```python\nfrom typing import Optional, List\n\ndef find_user(user_id: int) -> Optional[User]:\n    return users.get(user_id)\n```\n\n## 3. Use Context Managers\n\n```python\nwith open('file.txt', 'r') as f:\n    content = f.read()\n```\n\nWould you like me to elaborate on any of these?"),
        (MessageRole::User, "Yes, tell me more about type hints"),
        (MessageRole::Assistant, "## Deep Dive into Type Hints\n\nType hints were introduced in Python 3.5 and have become increasingly important.\n\n### Basic Types\n\n```python\nname: str = \"Alice\"\nage: int = 30\nheight: float = 5.9\nis_active: bool = True\n```\n\n### Collection Types\n\n```python\nfrom typing import List, Dict, Set, Tuple\n\nnames: List[str] = [\"Alice\", \"Bob\"]\nages: Dict[str, int] = {\"Alice\": 30}\nunique_ids: Set[int] = {1, 2, 3}\ncoordinates: Tuple[float, float] = (10.5, 20.3)\n```\n\n### Modern Syntax (Python 3.9+)\n\n```python\n# You can use built-in types directly\nnames: list[str] = [\"Alice\", \"Bob\"]\nages: dict[str, int] = {\"Alice\": 30}\n```\n\n### Optional and Union\n\n```python\nfrom typing import Optional, Union\n\ndef find_user(id: int) -> Optional[User]:\n    return None  # or User instance\n\n# Union for multiple types\ndef process(data: Union[str, bytes]) -> str:\n    if isinstance(data, bytes):\n        return data.decode()\n    return data\n```\n\n### Generic Types\n\n```python\nfrom typing import TypeVar, Generic\n\nT = TypeVar('T')\n\nclass Stack(Generic[T]):\n    def __init__(self) -> None:\n        self._items: list[T] = []\n    \n    def push(self, item: T) -> None:\n        self._items.append(item)\n    \n    def pop(self) -> T:\n        return self._items.pop()\n```"),
        (MessageRole::User, "What about dataclasses?"),
        (MessageRole::Assistant, "## Python Dataclasses\n\nDataclasses reduce boilerplate for classes that primarily store data.\n\n### Basic Usage\n\n```python\nfrom dataclasses import dataclass\n\n@dataclass\nclass User:\n    name: str\n    age: int\n    email: str = \"\"\n```\n\nThis automatically generates:\n- `__init__`\n- `__repr__`\n- `__eq__`\n\n### Frozen (Immutable) Dataclasses\n\n```python\n@dataclass(frozen=True)\nclass Point:\n    x: float\n    y: float\n```\n\n### With Default Factory\n\n```python\nfrom dataclasses import dataclass, field\nfrom typing import List\n\n@dataclass\nclass Team:\n    name: str\n    members: List[str] = field(default_factory=list)\n```\n\n### Post-Init Processing\n\n```python\n@dataclass\nclass Rectangle:\n    width: float\n    height: float\n    area: float = field(init=False)\n    \n    def __post_init__(self):\n        self.area = self.width * self.height\n```"),
        (MessageRole::User, "Thanks! One more question - what about async/await?"),
        (MessageRole::Assistant, "## Async/Await in Python\n\nAsynchronous programming allows concurrent execution without threads.\n\n### Basic Async Function\n\n```python\nimport asyncio\n\nasync def fetch_data(url: str) -> dict:\n    # Simulate network delay\n    await asyncio.sleep(1)\n    return {\"url\": url, \"data\": \"...\"}\n```\n\n### Running Async Code\n\n```python\nasync def main():\n    result = await fetch_data(\"https://api.example.com\")\n    print(result)\n\nasyncio.run(main())\n```\n\n### Concurrent Execution\n\n```python\nasync def fetch_all(urls: list[str]) -> list[dict]:\n    tasks = [fetch_data(url) for url in urls]\n    results = await asyncio.gather(*tasks)\n    return results\n```\n\n### Async Context Managers\n\n```python\nclass AsyncDatabase:\n    async def __aenter__(self):\n        await self.connect()\n        return self\n    \n    async def __aexit__(self, *args):\n        await self.disconnect()\n\nasync def main():\n    async with AsyncDatabase() as db:\n        await db.query(\"SELECT * FROM users\")\n```\n\n### Async Iterators\n\n```python\nasync def read_lines(filename: str):\n    async with aiofiles.open(filename) as f:\n        async for line in f:\n            yield line.strip()\n\nasync def main():\n    async for line in read_lines(\"data.txt\"):\n        print(line)\n```"),
    ];

    for (i, (role, content)) in python_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat1_id,
            role: *role,
            content: content.to_string(),
            created_at: now - Duration::minutes(120 - i as i64 * 10),
            tokens_used: Some(content.len() as u32 / 4),
            images: Vec::new(),
        })?;
        total_messages += 1;
    }

    // Chat 2: Quick question (TODAY)
    let chat2_id = ChatId::new();
    create_chat(&Chat {
        id: chat2_id,
        title: "Git Rebase vs Merge".to_string(),
        created_at: now - Duration::hours(1),
        updated_at: now - Duration::minutes(30),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat2_id,
        role: MessageRole::User,
        content: "What's the difference between git rebase and merge?".to_string(),
        created_at: now - Duration::minutes(35),
        tokens_used: Some(12),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat2_id,
        role: MessageRole::Assistant,
        content: "## Git Merge vs Rebase\n\n**Merge** creates a new commit that combines two branches:\n```\n  A---B---C feature\n /         \\\nD---E---F---G main (merge commit)\n```\n\n**Rebase** replays your commits on top of another branch:\n```\n          A'--B'--C' feature\n         /\nD---E---F main\n```\n\n### When to use each:\n- **Merge**: Preserves history, good for shared branches\n- **Rebase**: Cleaner history, good for local/feature branches".to_string(),
        created_at: now - Duration::minutes(34),
        tokens_used: Some(85),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 3: Another today chat (searchable keyword: "typescript")
    let chat3_id = ChatId::new();
    create_chat(&Chat {
        id: chat3_id,
        title: "TypeScript Generics Tutorial".to_string(),
        created_at: now - Duration::hours(3),
        updated_at: now - Duration::hours(2),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat3_id,
        role: MessageRole::User,
        content: "Explain TypeScript generics with examples".to_string(),
        created_at: now - Duration::hours(3),
        tokens_used: Some(8),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat3_id,
        role: MessageRole::Assistant,
        content: "## TypeScript Generics\n\nGenerics allow you to write reusable, type-safe code.\n\n```typescript\nfunction identity<T>(arg: T): T {\n    return arg;\n}\n\nconst num = identity<number>(42);\nconst str = identity<string>(\"hello\");\n```\n\n### Generic Interfaces\n\n```typescript\ninterface Container<T> {\n    value: T;\n    getValue(): T;\n}\n```\n\n### Generic Constraints\n\n```typescript\ninterface Lengthwise {\n    length: number;\n}\n\nfunction logLength<T extends Lengthwise>(arg: T): void {\n    console.log(arg.length);\n}\n```".to_string(),
        created_at: now - Duration::hours(3) + Duration::minutes(1),
        tokens_used: Some(95),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 4: Docker question (TODAY)
    let chat4_id = ChatId::new();
    create_chat(&Chat {
        id: chat4_id,
        title: "Docker Compose Setup".to_string(),
        created_at: now - Duration::hours(4),
        updated_at: now - Duration::hours(3),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat4_id,
        role: MessageRole::User,
        content: "How do I set up Docker Compose for a Node.js app with PostgreSQL?".to_string(),
        created_at: now - Duration::hours(4),
        tokens_used: Some(15),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat4_id,
        role: MessageRole::Assistant,
        content: "Here's a complete Docker Compose setup:\n\n```yaml\nversion: '3.8'\n\nservices:\n  app:\n    build: .\n    ports:\n      - \"3000:3000\"\n    environment:\n      - DATABASE_URL=postgresql://user:pass@db:5432/mydb\n    depends_on:\n      - db\n\n  db:\n    image: postgres:15\n    environment:\n      - POSTGRES_USER=user\n      - POSTGRES_PASSWORD=pass\n      - POSTGRES_DB=mydb\n    volumes:\n      - postgres_data:/var/lib/postgresql/data\n\nvolumes:\n  postgres_data:\n```\n\nAnd your Dockerfile:\n\n```dockerfile\nFROM node:20-alpine\nWORKDIR /app\nCOPY package*.json ./\nRUN npm install\nCOPY . .\nEXPOSE 3000\nCMD [\"npm\", \"start\"]\n```".to_string(),
        created_at: now - Duration::hours(4) + Duration::minutes(1),
        tokens_used: Some(120),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 5: React hooks (TODAY)
    let chat5_id = ChatId::new();
    create_chat(&Chat {
        id: chat5_id,
        title: "React useEffect Cleanup".to_string(),
        created_at: now - Duration::hours(5),
        updated_at: now - Duration::hours(4),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat5_id,
        role: MessageRole::User,
        content: "When do I need cleanup in useEffect?".to_string(),
        created_at: now - Duration::hours(5),
        tokens_used: Some(10),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat5_id,
        role: MessageRole::Assistant,
        content: "## useEffect Cleanup\n\nYou need cleanup when your effect:\n\n1. **Subscribes to events**\n```jsx\nuseEffect(() => {\n  window.addEventListener('resize', handler);\n  return () => window.removeEventListener('resize', handler);\n}, []);\n```\n\n2. **Sets up timers**\n```jsx\nuseEffect(() => {\n  const id = setInterval(tick, 1000);\n  return () => clearInterval(id);\n}, []);\n```\n\n3. **Opens connections**\n```jsx\nuseEffect(() => {\n  const ws = new WebSocket(url);\n  return () => ws.close();\n}, [url]);\n```".to_string(),
        created_at: now - Duration::hours(5) + Duration::minutes(1),
        tokens_used: Some(80),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // ========================================================================
    // YESTERDAY - 4 chats
    // ========================================================================
    let yesterday = now - Duration::days(1);

    // Chat 6: API Design (YESTERDAY) - Long conversation
    let chat6_id = ChatId::new();
    create_chat(&Chat {
        id: chat6_id,
        title: "RESTful API Design Principles".to_string(),
        created_at: yesterday - Duration::hours(5),
        updated_at: yesterday,
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    let api_messages = vec![
        (MessageRole::User, "What are the best practices for designing a REST API?"),
        (MessageRole::Assistant, "## REST API Design Best Practices\n\n### 1. Use Nouns for Resources\n```\nGET /users          # List users\nGET /users/123      # Get user 123\nPOST /users         # Create user\nPUT /users/123      # Update user 123\nDELETE /users/123   # Delete user 123\n```\n\n### 2. Use HTTP Methods Correctly\n- **GET**: Read (idempotent)\n- **POST**: Create\n- **PUT**: Full update (idempotent)\n- **PATCH**: Partial update\n- **DELETE**: Remove (idempotent)\n\n### 3. Use Proper Status Codes\n- 200 OK\n- 201 Created\n- 204 No Content\n- 400 Bad Request\n- 401 Unauthorized\n- 403 Forbidden\n- 404 Not Found\n- 500 Internal Server Error"),
        (MessageRole::User, "What about pagination?"),
        (MessageRole::Assistant, "## API Pagination\n\n### Offset-Based (Simple)\n```\nGET /users?page=2&limit=20\n```\n\nResponse:\n```json\n{\n  \"data\": [...],\n  \"pagination\": {\n    \"page\": 2,\n    \"limit\": 20,\n    \"total\": 150,\n    \"pages\": 8\n  }\n}\n```\n\n### Cursor-Based (Scalable)\n```\nGET /users?cursor=abc123&limit=20\n```\n\nResponse:\n```json\n{\n  \"data\": [...],\n  \"next_cursor\": \"xyz789\",\n  \"has_more\": true\n}\n```\n\n### Link Headers (RFC 5988)\n```\nLink: </users?page=3>; rel=\"next\",\n      </users?page=1>; rel=\"prev\""),
        (MessageRole::User, "How do I handle errors?"),
        (MessageRole::Assistant, "## API Error Handling\n\n### Standard Error Response Format\n\n```json\n{\n  \"error\": {\n    \"code\": \"VALIDATION_ERROR\",\n    \"message\": \"Invalid input data\",\n    \"details\": [\n      {\n        \"field\": \"email\",\n        \"message\": \"Invalid email format\"\n      }\n    ],\n    \"request_id\": \"req_abc123\"\n  }\n}\n```\n\n### Error Codes by Category\n\n**Client Errors (4xx)**\n- 400: Validation failed\n- 401: Missing/invalid auth\n- 403: Insufficient permissions\n- 404: Resource not found\n- 409: Conflict (duplicate)\n- 422: Unprocessable entity\n- 429: Rate limited\n\n**Server Errors (5xx)**\n- 500: Internal error\n- 502: Bad gateway\n- 503: Service unavailable\n- 504: Gateway timeout"),
    ];

    for (i, (role, content)) in api_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat6_id,
            role: *role,
            content: content.to_string(),
            created_at: yesterday - Duration::hours(5) + Duration::minutes(i as i64 * 5),
            tokens_used: Some(content.len() as u32 / 4),
            images: Vec::new(),
        })?;
        total_messages += 1;
    }

    // Chat 7: SQL Query (YESTERDAY)
    let chat7_id = ChatId::new();
    create_chat(&Chat {
        id: chat7_id,
        title: "Complex SQL JOIN Query".to_string(),
        created_at: yesterday - Duration::hours(8),
        updated_at: yesterday - Duration::hours(7),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat7_id,
        role: MessageRole::User,
        content: "Help me write a SQL query to get users with their orders".to_string(),
        created_at: yesterday - Duration::hours(8),
        tokens_used: Some(15),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat7_id,
        role: MessageRole::Assistant,
        content: "```sql\nSELECT \n    u.id,\n    u.name,\n    u.email,\n    COUNT(o.id) as order_count,\n    COALESCE(SUM(o.total), 0) as total_spent\nFROM users u\nLEFT JOIN orders o ON u.id = o.user_id\nWHERE u.active = true\nGROUP BY u.id, u.name, u.email\nHAVING COUNT(o.id) > 0\nORDER BY total_spent DESC\nLIMIT 100;\n```".to_string(),
        created_at: yesterday - Duration::hours(8) + Duration::minutes(1),
        tokens_used: Some(65),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 8: Kubernetes (YESTERDAY)
    let chat8_id = ChatId::new();
    create_chat(&Chat {
        id: chat8_id,
        title: "Kubernetes Deployment YAML".to_string(),
        created_at: yesterday - Duration::hours(10),
        updated_at: yesterday - Duration::hours(9),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat8_id,
        role: MessageRole::User,
        content: "Create a Kubernetes deployment for a web app".to_string(),
        created_at: yesterday - Duration::hours(10),
        tokens_used: Some(12),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat8_id,
        role: MessageRole::Assistant,
        content: "```yaml\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web-app\n  labels:\n    app: web-app\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: web-app\n  template:\n    metadata:\n      labels:\n        app: web-app\n    spec:\n      containers:\n      - name: web-app\n        image: myapp:latest\n        ports:\n        - containerPort: 8080\n        resources:\n          limits:\n            memory: \"256Mi\"\n            cpu: \"500m\"\n        livenessProbe:\n          httpGet:\n            path: /health\n            port: 8080\n          initialDelaySeconds: 30\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: web-app-service\nspec:\n  selector:\n    app: web-app\n  ports:\n  - port: 80\n    targetPort: 8080\n  type: LoadBalancer\n```".to_string(),
        created_at: yesterday - Duration::hours(10) + Duration::minutes(1),
        tokens_used: Some(120),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 9: CSS Grid (YESTERDAY)
    let chat9_id = ChatId::new();
    create_chat(&Chat {
        id: chat9_id,
        title: "CSS Grid Layout Examples".to_string(),
        created_at: yesterday - Duration::hours(12),
        updated_at: yesterday - Duration::hours(11),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat9_id,
        role: MessageRole::User,
        content: "Show me CSS Grid examples".to_string(),
        created_at: yesterday - Duration::hours(12),
        tokens_used: Some(8),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat9_id,
        role: MessageRole::Assistant,
        content: "## CSS Grid Examples\n\n### Basic Grid\n```css\n.container {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: 20px;\n}\n```\n\n### Named Areas\n```css\n.layout {\n  display: grid;\n  grid-template-areas:\n    \"header header header\"\n    \"sidebar main main\"\n    \"footer footer footer\";\n  grid-template-columns: 200px 1fr 1fr;\n}\n\n.header { grid-area: header; }\n.sidebar { grid-area: sidebar; }\n.main { grid-area: main; }\n```".to_string(),
        created_at: yesterday - Duration::hours(12) + Duration::minutes(1),
        tokens_used: Some(75),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // ========================================================================
    // THIS WEEK - 5 chats (2-6 days ago)
    // ========================================================================

    // Chat 10: Rust Ownership (3 days ago) - Very long conversation for scrolling
    let three_days_ago = now - Duration::days(3);
    let chat10_id = ChatId::new();
    create_chat(&Chat {
        id: chat10_id,
        title: "Understanding Rust Ownership".to_string(),
        created_at: three_days_ago - Duration::hours(2),
        updated_at: three_days_ago,
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    let rust_messages = vec![
        (MessageRole::User, "Explain Rust ownership to me"),
        (MessageRole::Assistant, "## Rust Ownership\n\nOwnership is Rust's most unique feature and enables memory safety without garbage collection.\n\n### The Three Rules\n\n1. Each value has an owner\n2. Only one owner at a time\n3. Value is dropped when owner goes out of scope\n\n```rust\nfn main() {\n    let s1 = String::from(\"hello\"); // s1 owns the string\n    let s2 = s1;                     // ownership moves to s2\n    // println!(\"{}\", s1);          // ERROR! s1 no longer valid\n    println!(\"{}\", s2);              // OK\n}\n```"),
        (MessageRole::User, "What about borrowing?"),
        (MessageRole::Assistant, "## Borrowing in Rust\n\nBorrowing lets you reference data without taking ownership.\n\n### Immutable Borrows\n\n```rust\nfn calculate_length(s: &String) -> usize {\n    s.len()\n}\n\nfn main() {\n    let s = String::from(\"hello\");\n    let len = calculate_length(&s); // borrow s\n    println!(\"Length of '{}' is {}\", s, len); // s still valid\n}\n```\n\n### Mutable Borrows\n\n```rust\nfn change(s: &mut String) {\n    s.push_str(\", world\");\n}\n\nfn main() {\n    let mut s = String::from(\"hello\");\n    change(&mut s);\n    println!(\"{}\", s); // \"hello, world\"\n}\n```\n\n### Borrowing Rules\n\n1. You can have either ONE mutable reference OR any number of immutable references\n2. References must always be valid"),
        (MessageRole::User, "Can you explain lifetimes?"),
        (MessageRole::Assistant, "## Rust Lifetimes\n\nLifetimes ensure references are valid for as long as they're used.\n\n### Why Lifetimes?\n\n```rust\n// This won't compile - dangling reference\nfn dangling() -> &String {\n    let s = String::from(\"hello\");\n    &s  // s goes out of scope, reference would be invalid\n}\n```\n\n### Lifetime Annotations\n\n```rust\n// 'a is a lifetime parameter\nfn longest<'a>(x: &'a str, y: &'a str) -> &'a str {\n    if x.len() > y.len() { x } else { y }\n}\n```\n\nThis says: \"The returned reference will live at least as long as the shorter of x and y.\"\n\n### Lifetime Elision\n\nRust can often infer lifetimes:\n\n```rust\n// These are equivalent\nfn first_word(s: &str) -> &str { ... }\nfn first_word<'a>(s: &'a str) -> &'a str { ... }\n```\n\n### Static Lifetime\n\n```rust\nlet s: &'static str = \"I live forever!\";\n```"),
        (MessageRole::User, "What are smart pointers?"),
        (MessageRole::Assistant, "## Smart Pointers in Rust\n\n### Box<T> - Heap Allocation\n\n```rust\nlet b = Box::new(5);\nprintln!(\"b = {}\", b);\n\n// Useful for recursive types\nenum List {\n    Cons(i32, Box<List>),\n    Nil,\n}\n```\n\n### Rc<T> - Reference Counting\n\n```rust\nuse std::rc::Rc;\n\nlet a = Rc::new(5);\nlet b = Rc::clone(&a); // Increases ref count\nlet c = Rc::clone(&a);\n\nprintln!(\"count = {}\", Rc::strong_count(&a)); // 3\n```\n\n### Arc<T> - Thread-Safe Rc\n\n```rust\nuse std::sync::Arc;\nuse std::thread;\n\nlet data = Arc::new(vec![1, 2, 3]);\n\nfor _ in 0..3 {\n    let data = Arc::clone(&data);\n    thread::spawn(move || {\n        println!(\"{:?}\", data);\n    });\n}\n```\n\n### RefCell<T> - Interior Mutability\n\n```rust\nuse std::cell::RefCell;\n\nlet data = RefCell::new(5);\n\n*data.borrow_mut() += 1;\nprintln!(\"{}\", data.borrow()); // 6\n```"),
        (MessageRole::User, "How do I handle errors in Rust?"),
        (MessageRole::Assistant, "## Error Handling in Rust\n\n### Result<T, E>\n\n```rust\nuse std::fs::File;\nuse std::io::{self, Read};\n\nfn read_file(path: &str) -> Result<String, io::Error> {\n    let mut file = File::open(path)?;\n    let mut contents = String::new();\n    file.read_to_string(&mut contents)?;\n    Ok(contents)\n}\n```\n\n### The ? Operator\n\nPropagates errors automatically:\n\n```rust\nfn get_user(id: u32) -> Result<User, Error> {\n    let conn = database::connect()?;\n    let user = conn.query_user(id)?;\n    Ok(user)\n}\n```\n\n### Custom Error Types\n\n```rust\n#[derive(Debug)]\nenum AppError {\n    IoError(io::Error),\n    ParseError(String),\n    NotFound,\n}\n\nimpl From<io::Error> for AppError {\n    fn from(err: io::Error) -> Self {\n        AppError::IoError(err)\n    }\n}\n```\n\n### anyhow for Applications\n\n```rust\nuse anyhow::{Context, Result};\n\nfn main() -> Result<()> {\n    let config = read_config()\n        .context(\"Failed to read config\")?;\n    Ok(())\n}\n```\n\n### thiserror for Libraries\n\n```rust\nuse thiserror::Error;\n\n#[derive(Error, Debug)]\npub enum DataError {\n    #[error(\"database error: {0}\")]\n    Database(#[from] sqlx::Error),\n    #[error(\"not found: {0}\")]\n    NotFound(String),\n}\n```"),
    ];

    for (i, (role, content)) in rust_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat10_id,
            role: *role,
            content: content.to_string(),
            created_at: three_days_ago - Duration::hours(2) + Duration::minutes(i as i64 * 8),
            tokens_used: Some(content.len() as u32 / 4),
            images: Vec::new(),
        })?;
        total_messages += 1;
    }

    // Chat 11: Authentication (4 days ago)
    let four_days_ago = now - Duration::days(4);
    let chat11_id = ChatId::new();
    create_chat(&Chat {
        id: chat11_id,
        title: "JWT Authentication Setup".to_string(),
        created_at: four_days_ago,
        updated_at: four_days_ago,
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
        source: ChatSource::default(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat11_id,
        role: MessageRole::User,
        content: "How do I implement JWT authentication?".to_string(),
        created_at: four_days_ago,
        tokens_used: Some(10),
        images: Vec::new(),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat11_id,
        role: MessageRole::Assistant,
        content: "## JWT Authentication\n\n```javascript\nconst jwt = require('jsonwebtoken');\n\n// Generate token\nconst token = jwt.sign(\n  { userId: user.id },\n  process.env.JWT_SECRET,\n  { expiresIn: '24h' }\n);\n\n// Verify token middleware\nfunction authenticate(req, res, next) {\n  const token = req.headers.authorization?.split(' ')[1];\n  try {\n    const decoded = jwt.verify(token, process.env.JWT_SECRET);\n    req.user = decoded;\n    next();\n  } catch (err) {\n    res.status(401).json({ error: 'Invalid token' });\n  }\n}\n```".to_string(),
        created_at: four_days_ago + Duration::minutes(1),
        tokens_used: Some(85),
            images: Vec::new(),
    })?;
    total_messages += 2;

    // Chat 12-14: More chats for variety
    for (i, (title, topic)) in [
        ("GraphQL Schema Design", "graphql"),
        ("WebSocket Implementation", "websocket"),
        ("CI/CD Pipeline Setup", "pipeline"),
    ]
    .iter()
    .enumerate()
    {
        let days_ago = now - Duration::days(5 + i as i64);
        let chat_id = ChatId::new();
        create_chat(&Chat {
            id: chat_id,
            title: title.to_string(),
            created_at: days_ago,
            updated_at: days_ago,
            deleted_at: None,
            model_id: if i % 2 == 0 {
                "claude-3-5-sonnet-20241022"
            } else {
                "gpt-4o"
            }
            .to_string(),
            provider: if i % 2 == 0 { "anthropic" } else { "openai" }.to_string(),
            source: ChatSource::default(),
        })?;
        total_chats += 1;

        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::User,
            content: format!("Tell me about {}", topic),
            created_at: days_ago,
            tokens_used: Some(6),
            images: Vec::new(),
        })?;
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::Assistant,
            content: format!(
                "Here's an overview of {}...\n\n(This is mock content for testing)",
                topic
            ),
            created_at: days_ago + Duration::minutes(1),
            tokens_used: Some(20),
            images: Vec::new(),
        })?;
        total_messages += 2;
    }

    // ========================================================================
    // OLDER - 5+ chats (8+ days ago)
    // ========================================================================

    for (i, title) in [
        "Machine Learning Basics",
        "Database Optimization",
        "Security Best Practices",
        "Microservices Architecture",
        "Testing Strategies",
        "Performance Tuning",
        "Code Review Guidelines",
    ]
    .iter()
    .enumerate()
    {
        let days_ago = now - Duration::days(10 + i as i64 * 3);
        let chat_id = ChatId::new();
        create_chat(&Chat {
            id: chat_id,
            title: title.to_string(),
            created_at: days_ago,
            updated_at: days_ago,
            deleted_at: None,
            model_id: "claude-3-5-sonnet-20241022".to_string(),
            provider: "anthropic".to_string(),
            source: ChatSource::default(),
        })?;
        total_chats += 1;

        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::User,
            content: format!("Explain {} in detail", title.to_lowercase()),
            created_at: days_ago,
            tokens_used: Some(8),
            images: Vec::new(),
        })?;
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::Assistant,
            content: format!("## {}\n\nThis is a comprehensive topic...\n\n(Mock content for testing the older section)", title),
            created_at: days_ago + Duration::minutes(1),
            tokens_used: Some(25),
            images: Vec::new(),
        })?;
        total_messages += 2;
    }

    info!(
        chat_count = total_chats,
        message_count = total_messages,
        "Comprehensive mock data inserted for AI visual testing"
    );

    Ok(())
}

/// Clear all mock data (for test cleanup)
pub fn clear_all_chats() -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM messages", [])
        .context("Failed to delete all messages")?;
    conn.execute("DELETE FROM chats", [])
        .context("Failed to delete all chats")?;

    info!("All chats and messages cleared");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_ai_db_path();
        assert!(path.to_string_lossy().contains("ai-chats.sqlite"));
        assert!(path.to_string_lossy().contains(".scriptkit/db"));
    }

    #[test]
    fn test_init_ai_db_is_idempotent() {
        // First call should succeed (may already be initialized from other tests)
        let result1 = init_ai_db();
        // If it fails, it should only be because it's already initialized (which is OK)
        if let Err(ref e) = result1 {
            // This is the old behavior we're fixing - it shouldn't error
            panic!("init_ai_db() should be idempotent but got error: {}", e);
        }

        // Second call should also succeed (idempotent)
        let result2 = init_ai_db();
        assert!(
            result2.is_ok(),
            "init_ai_db() should be idempotent, second call failed: {:?}",
            result2.err()
        );

        // Third call for good measure
        let result3 = init_ai_db();
        assert!(
            result3.is_ok(),
            "init_ai_db() should be idempotent, third call failed: {:?}",
            result3.err()
        );
    }

    #[test]
    fn test_search_chats_does_not_error() {
        // Ensure DB is initialized
        let _ = init_ai_db();

        // Empty search should return all chats (not error)
        let result = search_chats("");
        assert!(
            result.is_ok(),
            "Empty search should not error: {:?}",
            result.err()
        );

        // Simple text search should not error (even if no results)
        let result = search_chats("test");
        assert!(
            result.is_ok(),
            "Simple text search should not error: {:?}",
            result.err()
        );

        // Search with special characters should not crash
        // (FTS MATCH is fragile with special characters - should fall back gracefully)
        let result = search_chats("test@example.com");
        assert!(
            result.is_ok(),
            "Search with @ should not error: {:?}",
            result.err()
        );

        let result = search_chats("foo*bar");
        assert!(
            result.is_ok(),
            "Search with * should not error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fts_triggers_use_update_of_column() {
        // Ensure DB is initialized
        let _ = init_ai_db();

        let db = get_db().expect("Should get db connection");
        let conn = db.lock().expect("Should lock connection");

        // Query the trigger SQL to verify it uses "UPDATE OF" syntax
        let chat_trigger_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='chats_au'",
                [],
                |row| row.get(0),
            )
            .expect("Should find chats_au trigger");

        // The trigger should only fire on UPDATE OF title, not on all updates
        assert!(
            chat_trigger_sql.to_lowercase().contains("update of title"),
            "chats_au trigger should use 'UPDATE OF title' to avoid FTS churn on updated_at changes. Got: {}",
            chat_trigger_sql
        );

        let message_trigger_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='messages_au'",
                [],
                |row| row.get(0),
            )
            .expect("Should find messages_au trigger");

        // The trigger should only fire on UPDATE OF content, not on all updates
        assert!(
            message_trigger_sql
                .to_lowercase()
                .contains("update of content"),
            "messages_au trigger should use 'UPDATE OF content' to avoid FTS churn. Got: {}",
            message_trigger_sql
        );
    }

    #[test]
    fn test_ai_db_has_required_pragmas() {
        // Ensure DB is initialized
        let _ = init_ai_db();

        let db = get_db().expect("Should get db connection");
        let conn = db.lock().expect("Should lock connection");

        // Verify WAL mode is enabled
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("Should query journal_mode");
        assert_eq!(
            journal_mode.to_lowercase(),
            "wal",
            "AI DB should use WAL mode for better concurrency"
        );

        // Verify foreign keys are enabled
        let foreign_keys: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("Should query foreign_keys");
        assert_eq!(
            foreign_keys, 1,
            "AI DB should have foreign_keys=ON for CASCADE to work"
        );

        // Verify busy_timeout is set (should be > 0)
        let busy_timeout: i32 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("Should query busy_timeout");
        assert!(
            busy_timeout >= 1000,
            "AI DB should have busy_timeout >= 1000ms, got {}",
            busy_timeout
        );
    }
}
