//! Brain store: the unified memory substrate at `~/.scriptkit/db/brain.sqlite`.
//!
//! Documents from every source (notes, chat turns, future: clipboard
//! promotions, browser captures) are normalized into `brain_docs` with their
//! own FTS5 index, and embedded into `brain_embeddings` by the background
//! indexer. `brain_signals` is the append-only attention log: what John
//! searches, asks, and selects — used to boost ranking toward what currently
//! matters.
//!
//! Design notes:
//! - Separate sqlite file (NOT notes.sqlite): the brain indexes many sources
//!   and must never contend with or complicate the notes schema.
//! - Vectors are BLOB f32 arrays, brute-force cosine in memory. At the scale
//!   of a personal knowledge base (thousands of docs) this is single-digit
//!   milliseconds and avoids shipping a vector-extension dependency.

use anyhow::{anyhow, Context as _, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

static BRAIN_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// A document source. Stable string keys — stored in sqlite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocSource {
    Note,
    ChatTurn,
    Clipboard,
}

impl DocSource {
    pub fn as_str(self) -> &'static str {
        match self {
            DocSource::Note => "note",
            DocSource::ChatTurn => "chat_turn",
            DocSource::Clipboard => "clipboard",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "note" => Some(DocSource::Note),
            "chat_turn" => Some(DocSource::ChatTurn),
            "clipboard" => Some(DocSource::Clipboard),
            _ => None,
        }
    }

    /// Human label used when rendering retrieved context for the agent.
    pub fn label(self) -> &'static str {
        match self {
            DocSource::Note => "Note",
            DocSource::ChatTurn => "Past conversation",
            DocSource::Clipboard => "Clipboard",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrainDoc {
    pub id: i64,
    pub source: DocSource,
    pub source_id: String,
    pub title: String,
    pub content: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct BrainSignal {
    pub topic: String,
    pub weight: i64,
    pub source: String,
    pub created_at: i64,
}

fn brain_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("SCRIPT_KIT_TEST_BRAIN_DB_PATH") {
        return PathBuf::from(path);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".scriptkit").join("db").join("brain.sqlite")
}

fn ensure_brain_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS brain_docs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            source_id TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            UNIQUE(source, source_id)
        );
        CREATE INDEX IF NOT EXISTS idx_brain_docs_source ON brain_docs(source);
        CREATE INDEX IF NOT EXISTS idx_brain_docs_updated ON brain_docs(updated_at DESC);

        CREATE VIRTUAL TABLE IF NOT EXISTS brain_docs_fts USING fts5(
            title, content, content='brain_docs', content_rowid='id'
        );

        CREATE TRIGGER IF NOT EXISTS brain_docs_ai AFTER INSERT ON brain_docs BEGIN
            INSERT INTO brain_docs_fts(rowid, title, content)
            VALUES (new.id, new.title, new.content);
        END;
        CREATE TRIGGER IF NOT EXISTS brain_docs_ad AFTER DELETE ON brain_docs BEGIN
            INSERT INTO brain_docs_fts(brain_docs_fts, rowid, title, content)
            VALUES ('delete', old.id, old.title, old.content);
        END;
        CREATE TRIGGER IF NOT EXISTS brain_docs_au AFTER UPDATE ON brain_docs BEGIN
            INSERT INTO brain_docs_fts(brain_docs_fts, rowid, title, content)
            VALUES ('delete', old.id, old.title, old.content);
            INSERT INTO brain_docs_fts(rowid, title, content)
            VALUES (new.id, new.title, new.content);
        END;

        CREATE TABLE IF NOT EXISTS brain_embeddings (
            doc_id INTEGER PRIMARY KEY REFERENCES brain_docs(id) ON DELETE CASCADE,
            model_id TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            dim INTEGER NOT NULL,
            vec BLOB NOT NULL,
            embedded_at INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE TABLE IF NOT EXISTS brain_signals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            topic TEXT NOT NULL,
            weight INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT (unixepoch())
        );
        CREATE INDEX IF NOT EXISTS idx_brain_signals_created ON brain_signals(created_at DESC);

        CREATE TABLE IF NOT EXISTS brain_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .context("ensure brain schema")
}

/// Initialize (or return) the global brain DB connection. Idempotent.
pub fn init_brain_db() -> Result<()> {
    if BRAIN_DB.get().is_some() {
        return Ok(());
    }
    let path = brain_db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create brain db dir")?;
    }
    let conn = Connection::open(&path).context("open brain.sqlite")?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("brain WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .context("brain busy_timeout")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("brain foreign_keys")?;
    ensure_brain_schema(&conn)?;
    let _ = BRAIN_DB.set(Arc::new(Mutex::new(conn)));
    Ok(())
}

fn get_db() -> Result<Arc<Mutex<Connection>>> {
    init_brain_db()?;
    BRAIN_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("brain db not initialized"))
}

fn content_hash(title: &str, content: &str) -> String {
    // FNV-1a 64: fast, stable, no new dependency; collisions only cost a
    // redundant re-embed.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in title
        .as_bytes()
        .iter()
        .chain([0u8].iter())
        .chain(content.as_bytes())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("{hash:016x}")
}

/// Insert or update a document. Returns the doc id. The embedding is
/// invalidated automatically when content changes (hash mismatch leaves the
/// stale row for the indexer to refresh).
pub fn upsert_doc(
    source: DocSource,
    source_id: &str,
    title: &str,
    content: &str,
    updated_at: i64,
) -> Result<i64> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let hash = content_hash(title, content);
    conn.execute(
        "INSERT INTO brain_docs (source, source_id, title, content, content_hash, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(source, source_id) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            content_hash = excluded.content_hash,
            updated_at = excluded.updated_at
         WHERE brain_docs.content_hash != excluded.content_hash
            OR brain_docs.updated_at != excluded.updated_at",
        params![source.as_str(), source_id, title, content, hash, updated_at],
    )
    .context("upsert brain doc")?;
    let id: i64 = conn
        .query_row(
            "SELECT id FROM brain_docs WHERE source = ?1 AND source_id = ?2",
            params![source.as_str(), source_id],
            |row| row.get(0),
        )
        .context("read brain doc id")?;
    Ok(id)
}

/// Remove a document (e.g. when its source note is deleted).
pub fn remove_doc(source: DocSource, source_id: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    conn.execute(
        "DELETE FROM brain_docs WHERE source = ?1 AND source_id = ?2",
        params![source.as_str(), source_id],
    )
    .context("remove brain doc")?;
    Ok(())
}

/// Docs whose embedding is missing or stale for the given model. Ordered by
/// recency so fresh material becomes searchable first.
pub fn docs_needing_embedding(model_id: &str, limit: usize) -> Result<Vec<BrainDoc>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let mut stmt = conn.prepare(
        "SELECT d.id, d.source, d.source_id, d.title, d.content, d.updated_at
         FROM brain_docs d
         LEFT JOIN brain_embeddings e ON e.doc_id = d.id
         WHERE e.doc_id IS NULL
            OR e.model_id != ?1
            OR e.content_hash != d.content_hash
         ORDER BY d.updated_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![model_id, limit as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, source, source_id, title, content, updated_at)| {
            DocSource::parse(&source).map(|source| BrainDoc {
                id,
                source,
                source_id,
                title,
                content,
                updated_at,
            })
        })
        .collect())
}

pub fn store_embedding(
    doc_id: i64,
    model_id: &str,
    title: &str,
    content: &str,
    vec: &[f32],
) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let hash = content_hash(title, content);
    let bytes: Vec<u8> = vec.iter().flat_map(|f| f.to_le_bytes()).collect();
    conn.execute(
        "INSERT INTO brain_embeddings (doc_id, model_id, content_hash, dim, vec)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(doc_id) DO UPDATE SET
            model_id = excluded.model_id,
            content_hash = excluded.content_hash,
            dim = excluded.dim,
            vec = excluded.vec,
            embedded_at = unixepoch()",
        params![doc_id, model_id, hash, vec.len() as i64, bytes],
    )
    .context("store brain embedding")?;
    Ok(())
}

/// All embeddings for the given model: (doc_id, vector). Loaded into memory
/// for brute-force cosine — see module docs for why this is fine.
pub fn load_embeddings(model_id: &str) -> Result<Vec<(i64, Vec<f32>)>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let mut stmt = conn.prepare("SELECT doc_id, vec FROM brain_embeddings WHERE model_id = ?1")?;
    let rows = stmt
        .query_map(params![model_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .map(|(id, bytes)| {
            let vec = bytes
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            (id, vec)
        })
        .collect())
}

/// FTS5 BM25 search over all brain docs. Returns (doc_id, rank) best-first.
pub fn fts_search(query: &str, limit: usize) -> Result<Vec<i64>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let fts_query = sanitize_fts_query(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT rowid FROM brain_docs_fts WHERE brain_docs_fts MATCH ?1
         ORDER BY bm25(brain_docs_fts, 2.0, 1.0) LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![fts_query, limit as i64], |row| row.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Quote each term (so punctuation can't break FTS5 syntax) and join with OR:
/// recall queries are natural-language questions ("what branch does bluefin
/// deploy from"), and FTS5's implicit AND would require every filler word to
/// appear in a document. OR + BM25 ranking keeps precision: documents
/// matching more terms rank higher.
fn sanitize_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|term| term.replace('"', ""))
        .filter(|term| term.len() > 3)
        .map(|term| format!("\"{term}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub fn get_docs_by_ids(ids: &[i64]) -> Result<Vec<BrainDoc>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, source, source_id, title, content, updated_at
         FROM brain_docs WHERE id IN ({placeholders})"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, source, source_id, title, content, updated_at)| {
            DocSource::parse(&source).map(|source| BrainDoc {
                id,
                source,
                source_id,
                title,
                content,
                updated_at,
            })
        })
        .collect())
}

/// Append an attention signal. Topics are free-form lowercase strings.
pub fn record_signal(topic: &str, weight: i64, source: &str) -> Result<()> {
    let topic = topic.trim().to_lowercase();
    if topic.is_empty() {
        return Ok(());
    }
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    conn.execute(
        "INSERT INTO brain_signals (topic, weight, source) VALUES (?1, ?2, ?3)",
        params![topic, weight, source],
    )
    .context("record brain signal")?;
    Ok(())
}

/// Recent signals (newest first), for ranking boosts and the focus view.
pub fn recent_signals(limit: usize) -> Result<Vec<BrainSignal>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let mut stmt = conn.prepare(
        "SELECT topic, weight, source, created_at FROM brain_signals
         ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(BrainSignal {
                topic: row.get(0)?,
                weight: row.get(1)?,
                source: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn meta_get(key: &str) -> Result<Option<String>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    conn.query_row(
        "SELECT value FROM brain_meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .context("brain meta get")
}

pub fn meta_set(key: &str, value: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    conn.execute(
        "INSERT INTO brain_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .context("brain meta set")?;
    Ok(())
}

/// Stats for the brain status surface and doctor checks.
pub fn doc_stats() -> Result<(i64, i64, i64)> {
    let db = get_db()?;
    let conn = db.lock().map_err(|_| anyhow!("brain db lock poisoned"))?;
    let docs: i64 = conn.query_row("SELECT COUNT(*) FROM brain_docs", [], |r| r.get(0))?;
    let embedded: i64 =
        conn.query_row("SELECT COUNT(*) FROM brain_embeddings", [], |r| r.get(0))?;
    let signals: i64 = conn.query_row("SELECT COUNT(*) FROM brain_signals", [], |r| r.get(0))?;
    Ok((docs, embedded, signals))
}
