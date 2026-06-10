//! Brain inbox: things the brain noticed that deserve a human glance.
//!
//! The curator's daily pass files small, factual items here — commitments
//! made in chats, questions that never got answered, topics drifting out of
//! focus, pinned notes going stale. Each row is provenance-linked (same
//! source vocabulary as [`super::store::DocSource`]) and deduped by a stable
//! hash of `kind|title`, so a re-run of the curator can't double-file the
//! same observation. Stage B surfaces open items in the launcher.
//!
//! Retention: open items live until resolved; resolved items age out of
//! `prune_ambient_data` after 30 days (see store.rs).

use super::store;
use anyhow::{Context as _, Result};
use rusqlite::params;

/// An inbox item category. Stable string keys — stored in sqlite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxKind {
    Commitment,
    Question,
    Drift,
    StalePin,
}

impl InboxKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InboxKind::Commitment => "commitment",
            InboxKind::Question => "question",
            InboxKind::Drift => "drift",
            InboxKind::StalePin => "stale_pin",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "commitment" => Some(InboxKind::Commitment),
            "question" => Some(InboxKind::Question),
            "drift" => Some(InboxKind::Drift),
            "stale_pin" => Some(InboxKind::StalePin),
            _ => None,
        }
    }

    /// Human label used when rendering inbox rows.
    pub fn label(self) -> &'static str {
        match self {
            InboxKind::Commitment => "Commitment",
            InboxKind::Question => "Open Question",
            InboxKind::Drift => "Drifting",
            InboxKind::StalePin => "Stale Pin",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InboxItem {
    pub id: i64,
    pub kind: InboxKind,
    pub title: String,
    pub detail: String,
    pub source: String,
    pub source_id: String,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

/// Stable dedupe key: FNV-1a 64 hex of `kind|title`, with the title
/// lowercased and whitespace-collapsed so trivial rephrasings ("Ship  X "
/// vs "ship x") can't duplicate a row.
fn dedupe_hash(kind: InboxKind, title: &str) -> String {
    let normalized = title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in kind
        .as_str()
        .as_bytes()
        .iter()
        .chain([b'|'].iter())
        .chain(normalized.as_bytes())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("{hash:016x}")
}

/// File an inbox item. Returns whether a new row was inserted (false when an
/// item with the same kind + normalized title already exists, or the title is
/// blank).
pub fn insert_inbox_item(
    kind: InboxKind,
    title: &str,
    detail: &str,
    source: &str,
    source_id: &str,
) -> Result<bool> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(false);
    }
    let hash = dedupe_hash(kind, title);
    store::with_conn(|conn| {
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO brain_inbox
                    (kind, title, detail, source, source_id, dedupe_hash, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, unixepoch())",
                params![kind.as_str(), title, detail.trim(), source, source_id, hash],
            )
            .context("insert brain inbox item")?;
        Ok(inserted > 0)
    })
}

/// Open (unresolved) items, newest first.
pub fn open_inbox_items(limit: usize) -> Result<Vec<InboxItem>> {
    store::with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, kind, title, detail, source, source_id, created_at, resolved_at
             FROM brain_inbox WHERE resolved_at IS NULL
             ORDER BY created_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows
            .into_iter()
            .filter_map(
                |(id, kind, title, detail, source, source_id, created_at, resolved_at)| {
                    InboxKind::parse(&kind).map(|kind| InboxItem {
                        id,
                        kind,
                        title,
                        detail,
                        source,
                        source_id,
                        created_at,
                        resolved_at,
                    })
                },
            )
            .collect())
    })
}

/// Mark an item resolved. Returns false when the id is unknown or already
/// resolved.
pub fn resolve_inbox_item(id: i64) -> Result<bool> {
    resolve_inbox_item_at(id, chrono::Utc::now().timestamp())
}

/// Testable core of [`resolve_inbox_item`] — `now` is injectable.
pub(crate) fn resolve_inbox_item_at(id: i64, now: i64) -> Result<bool> {
    store::with_conn(|conn| {
        let updated = conn
            .execute(
                "UPDATE brain_inbox SET resolved_at = ?1
                 WHERE id = ?2 AND resolved_at IS NULL",
                params![now, id],
            )
            .context("resolve brain inbox item")?;
        Ok(updated > 0)
    })
}

/// Count of open items, for badges and the health surface.
pub fn count_open_inbox() -> Result<i64> {
    store::with_conn(|conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM brain_inbox WHERE resolved_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("count open brain inbox items")
    })
}
