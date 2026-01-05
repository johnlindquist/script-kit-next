//! Database worker thread
//!
//! Single-threaded SQLite access via message passing.
//! Eliminates global Mutex contention and enables proper WAL concurrency.
//!
//! This module provides infrastructure for migrating from the global
//! `Arc<Mutex<Connection>>` pattern to a dedicated DB worker thread.
//! The migration will be done incrementally - currently this is unused
//! but provides the architecture for the fix.

#![allow(dead_code)] // Infrastructure module - wired up incrementally

mod db_impl;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info};

use super::types::{ClipboardEntry, ClipboardEntryMeta, ContentType};
use db_impl::*;

/// Global sender to the DB worker thread
static DB_SENDER: OnceLock<Sender<DbRequest>> = OnceLock::new();

/// Guard to ensure worker is started only once
static WORKER_STARTED: OnceLock<JoinHandle<()>> = OnceLock::new();

/// Request types for the DB worker
pub enum DbRequest {
    /// Add or update an entry (dedup by content hash)
    AddOrTouch {
        content: String,
        content_type: ContentType,
        content_hash: String,
        text_preview: Option<String>,
        image_width: Option<u32>,
        image_height: Option<u32>,
        byte_size: usize,
        reply: SyncSender<Result<String>>,
    },
    /// Get entry content by ID
    GetContent {
        id: String,
        reply: SyncSender<Option<String>>,
    },
    /// Get entry by ID (full entry including content)
    GetEntry {
        id: String,
        reply: SyncSender<Option<ClipboardEntry>>,
    },
    /// Get paginated entry metadata (no content payload)
    GetMeta {
        limit: usize,
        offset: usize,
        reply: SyncSender<Vec<ClipboardEntryMeta>>,
    },
    /// Get paginated full entries
    GetPage {
        limit: usize,
        offset: usize,
        reply: SyncSender<Vec<ClipboardEntry>>,
    },
    /// Get total entry count
    GetCount { reply: SyncSender<usize> },
    /// Pin an entry
    Pin { id: String, reply: SyncSender<Result<()>> },
    /// Unpin an entry
    Unpin { id: String, reply: SyncSender<Result<()>> },
    /// Remove an entry
    Remove { id: String, reply: SyncSender<Result<()>> },
    /// Clear all history
    Clear { reply: SyncSender<Result<()>> },
    /// Prune old entries (returns count deleted)
    Prune { cutoff_timestamp_ms: i64, reply: SyncSender<Result<usize>> },
    /// Trim oversized text entries (returns count deleted)
    TrimOversized { max_len: usize, reply: SyncSender<Result<usize>> },
    /// Update OCR text for an entry
    UpdateOcr { id: String, text: String, reply: SyncSender<Result<()>> },
    /// Run incremental vacuum
    IncrementalVacuum { reply: SyncSender<Result<()>> },
    /// Run WAL checkpoint
    WalCheckpoint { reply: SyncSender<Result<()>> },
    /// Shutdown the worker
    Shutdown,
}

/// Get the database path (~/.scriptkit/db/clipboard-history.sqlite)
pub fn get_db_path() -> Result<PathBuf> {
    let kit_dir = PathBuf::from(shellexpand::tilde("~/.scriptkit").as_ref());
    let db_dir = kit_dir.join("db");
    if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir).context("Failed to create ~/.scriptkit/db directory")?;
    }
    Ok(db_dir.join("clipboard-history.sqlite"))
}

/// Start the database worker thread
pub fn start_db_worker() -> Result<()> {
    if WORKER_STARTED.get().is_some() {
        debug!("DB worker already started");
        return Ok(());
    }

    let (tx, rx): (Sender<DbRequest>, Receiver<DbRequest>) = mpsc::channel();
    if DB_SENDER.set(tx).is_err() {
        debug!("DB sender already set");
        return Ok(());
    }

    let handle = thread::spawn(move || match init_connection() {
        Ok(conn) => db_worker_loop(conn, rx),
        Err(e) => error!(error = %e, "Failed to initialize DB worker connection"),
    });

    let _ = WORKER_STARTED.set(handle);
    info!("DB worker thread started");
    Ok(())
}

/// Get the sender to the DB worker
pub fn get_db_sender() -> Option<&'static Sender<DbRequest>> {
    DB_SENDER.get()
}

fn init_connection() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database at {:?}", db_path))?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; \
         PRAGMA busy_timeout = 5000; PRAGMA auto_vacuum = INCREMENTAL;",
    ).context("Failed to set database pragmas")?;

    create_schema(&conn)?;
    run_migrations(&conn)?;
    create_indexes(&conn)?;

    info!("Database worker initialized");
    Ok(conn)
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY, content TEXT NOT NULL, content_hash TEXT,
            content_type TEXT NOT NULL DEFAULT 'text', timestamp INTEGER NOT NULL,
            pinned INTEGER DEFAULT 0, ocr_text TEXT
        )", [],
    ).context("Failed to create history table")?;
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<()> {
    add_column_if_missing(conn, "ocr_text", "TEXT")?;
    add_column_if_missing(conn, "content_hash", "TEXT")?;
    add_column_if_missing(conn, "text_preview", "TEXT")?;
    add_column_if_missing(conn, "image_width", "INTEGER")?;
    add_column_if_missing(conn, "image_height", "INTEGER")?;
    add_column_if_missing(conn, "byte_size", "INTEGER DEFAULT 0")?;

    let needs_ts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM history WHERE timestamp < 100000000000 AND timestamp > 0",
        [], |row| row.get(0),
    ).unwrap_or(0);

    if needs_ts > 0 {
        conn.execute(
            "UPDATE history SET timestamp = timestamp * 1000 WHERE timestamp < 100000000000 AND timestamp > 0",
            [],
        )?;
        info!(count = needs_ts, "Migrated timestamps to milliseconds");
    }
    Ok(())
}

fn add_column_if_missing(conn: &Connection, name: &str, col_type: &str) -> Result<()> {
    let has: bool = conn.query_row(
        &format!("SELECT COUNT(*) FROM pragma_table_info('history') WHERE name='{}'", name),
        [], |row| row.get::<_, i32>(0),
    ).map(|c| c > 0).unwrap_or(false);

    if !has {
        conn.execute(&format!("ALTER TABLE history ADD COLUMN {} {}", name, col_type), [])?;
        info!(column = name, "Added column to history table");
    }
    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    conn.execute("CREATE INDEX IF NOT EXISTS idx_timestamp ON history(timestamp DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_pinned_timestamp ON history(pinned DESC, timestamp DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_dedup ON history(content_type, content_hash)", [])?;
    Ok(())
}

fn db_worker_loop(conn: Connection, rx: Receiver<DbRequest>) {
    info!("DB worker loop started");
    for request in rx {
        if !handle_request(&conn, request) { break; }
    }
    info!("DB worker loop ended");
}

fn handle_request(conn: &Connection, req: DbRequest) -> bool {
    match req {
        DbRequest::AddOrTouch { content, content_type, content_hash, text_preview, image_width, image_height, byte_size, reply } => {
            let _ = reply.send(add_or_touch_impl(conn, &content, content_type, &content_hash, text_preview, image_width, image_height, byte_size));
        }
        DbRequest::GetContent { id, reply } => { let _ = reply.send(get_content_impl(conn, &id)); }
        DbRequest::GetEntry { id, reply } => { let _ = reply.send(get_entry_impl(conn, &id)); }
        DbRequest::GetMeta { limit, offset, reply } => { let _ = reply.send(get_meta_impl(conn, limit, offset)); }
        DbRequest::GetPage { limit, offset, reply } => { let _ = reply.send(get_page_impl(conn, limit, offset)); }
        DbRequest::GetCount { reply } => { let _ = reply.send(get_count_impl(conn)); }
        DbRequest::Pin { id, reply } => { let _ = reply.send(pin_impl(conn, &id)); }
        DbRequest::Unpin { id, reply } => { let _ = reply.send(unpin_impl(conn, &id)); }
        DbRequest::Remove { id, reply } => { let _ = reply.send(remove_impl(conn, &id)); }
        DbRequest::Clear { reply } => { let _ = reply.send(clear_impl(conn)); }
        DbRequest::Prune { cutoff_timestamp_ms, reply } => { let _ = reply.send(prune_impl(conn, cutoff_timestamp_ms)); }
        DbRequest::TrimOversized { max_len, reply } => { let _ = reply.send(trim_oversized_impl(conn, max_len)); }
        DbRequest::UpdateOcr { id, text, reply } => { let _ = reply.send(update_ocr_impl(conn, &id, &text)); }
        DbRequest::IncrementalVacuum { reply } => { let _ = reply.send(vacuum_impl(conn)); }
        DbRequest::WalCheckpoint { reply } => { let _ = reply.send(checkpoint_impl(conn)); }
        DbRequest::Shutdown => { info!("DB worker shutdown"); return false; }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path_format() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("clipboard-history.sqlite"));
    }
}
