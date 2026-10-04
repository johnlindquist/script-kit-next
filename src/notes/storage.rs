//! Notes Storage Layer
//!
//! Markdown files under `brain/notes/` are canonical; `notes.sqlite` is a
//! derived, rebuildable index (FTS, tags, aliases, backlinks). See ADR 0003.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::brain::substrate::{io as brain_io, BrainFrontmatter, BrainSubstrate};
use crate::scripts::root_search_contract::{
    RootOwnedProviderRefresh, RootOwnedProviderRefreshLifecycle,
};

use super::file_names;
use super::metadata;
use super::model::{Note, NoteId};

/// SQLite index schema generation — bump when index shape changes.
const NOTES_INDEX_SCHEMA_VERSION: i32 = 2;
/// Root-level source filters may request the whole bounded launcher result set.
const MAX_ROOT_NOTES_SEARCH_RESULTS: usize = 24;

/// Global database connection for notes
static NOTES_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();
static NOTES_SUBSTRATE: OnceLock<Arc<BrainSubstrate>> = OnceLock::new();
static NOTE_CONTENT_HASHES: OnceLock<Mutex<HashMap<NoteId, String>>> = OnceLock::new();
static ROOT_NOTES_SEARCH_CACHE: OnceLock<Mutex<RootNotesSearchCache>> = OnceLock::new();
static ROOT_NOTES_SEARCH_CACHE_GENERATION: AtomicU64 = AtomicU64::new(0);
static NOTES_STORAGE_GENERATION: AtomicU64 = AtomicU64::new(0);
static NOTES_DIR_WATCHER_STARTED: AtomicBool = AtomicBool::new(false);

fn db_lock_err(e: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!("DB lock error: {e}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootNotesSectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub search_content: bool,
}

impl Default for RootNotesSectionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            max_results: 3,
            min_query_chars: 3,
            search_content: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RootNoteSearchHit {
    pub id: NoteId,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub is_pinned: bool,
    pub char_count: usize,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NoteBacklinkSummary {
    pub id: NoteId,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RootNotesSearchCacheKey {
    query: String,
    enabled: bool,
    max_results: usize,
    min_query_chars: usize,
    search_content: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RootNotesSearchFlightKey {
    generation: u64,
    search: RootNotesSearchCacheKey,
}

#[derive(Clone, Debug)]
pub(crate) struct RootNotesSearchRefresh {
    owner: RootOwnedProviderRefresh,
    flight: RootNotesSearchFlightKey,
    options: RootNotesSectionOptions,
}

pub(crate) struct RootNotesSearchSnapshot {
    flight: RootNotesSearchFlightKey,
    hits: Result<Vec<RootNoteSearchHit>>,
}

impl RootNotesSearchSnapshot {
    pub(crate) fn read_outcome(&self) -> Result<usize, &anyhow::Error> {
        self.hits.as_ref().map(Vec::len)
    }
}

struct RootNotesCachedHits {
    generation: u64,
    hits: Vec<RootNoteSearchHit>,
}

#[derive(Default)]
struct RootNotesSearchCache {
    hits_by_query: HashMap<RootNotesSearchCacheKey, RootNotesCachedHits>,
    in_flight: HashSet<RootNotesSearchFlightKey>,
    refresh_lifecycle: RootOwnedProviderRefreshLifecycle,
}

fn root_notes_search_cache() -> &'static Mutex<RootNotesSearchCache> {
    ROOT_NOTES_SEARCH_CACHE.get_or_init(|| Mutex::new(RootNotesSearchCache::default()))
}

fn root_notes_search_cache_key(
    query: &str,
    options: RootNotesSectionOptions,
) -> RootNotesSearchCacheKey {
    RootNotesSearchCacheKey {
        query: query.trim().to_string(),
        enabled: options.enabled,
        max_results: options.max_results,
        min_query_chars: options.min_query_chars,
        search_content: options.search_content,
    }
}

pub(crate) fn invalidate_root_notes_search_cache() {
    ROOT_NOTES_SEARCH_CACHE_GENERATION.fetch_add(1, Ordering::Relaxed);
    NOTES_STORAGE_GENERATION.fetch_add(1, Ordering::Relaxed);
    if let Some(cache) = ROOT_NOTES_SEARCH_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.hits_by_query.clear();
        }
    }
}

fn fingerprint_path(path: &std::path::Path) -> (String, usize) {
    let path_text = path.to_string_lossy();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path_text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (format!("fnv1a64:{hash:016x}"), path_text.chars().count())
}

/// Redacted storage identity for automation receipts. Never returns raw
/// paths.
///
/// The database and the canonical Brain markdown substrate are sandboxed by
/// SEPARATE environment variables (`SCRIPT_KIT_TEST_NOTES_DB_PATH` and
/// `SCRIPT_KIT_TEST_NOTES_BRAIN_PATH`); a DB-only sandbox can still point
/// canonical note writes at the real Brain directory. `fullySandboxed` is
/// therefore the conjunction, and mutation probes must gate on it — not on
/// the legacy `testSandbox` (kept for older readers: DB-only signal).
pub(crate) fn automation_storage_identity() -> serde_json::Value {
    let (db_fingerprint, db_len) = fingerprint_path(&get_notes_db_path());
    let (brain_fingerprint, brain_len) = fingerprint_path(&get_notes_brain_base_path());
    let owned = crate::runtime_policy::is_owned_evaluation();
    let db_sandbox =
        owned || std::env::var_os("SCRIPT_KIT_TEST_NOTES_DB_PATH").is_some() || cfg!(test);
    let brain_sandbox =
        owned || std::env::var_os("SCRIPT_KIT_TEST_NOTES_BRAIN_PATH").is_some() || cfg!(test);

    serde_json::json!({
        "schemaVersion": 2,
        "redacted": true,
        "generation": NOTES_STORAGE_GENERATION.load(Ordering::Relaxed),
        "rootSearchCacheGeneration": ROOT_NOTES_SEARCH_CACHE_GENERATION.load(Ordering::Relaxed),
        "dbPathFingerprint": db_fingerprint,
        "dbPathLength": db_len,
        "brainPathFingerprint": brain_fingerprint,
        "brainPathLength": brain_len,
        "dbSandbox": db_sandbox,
        "brainSandbox": brain_sandbox,
        "fullySandboxed": db_sandbox && brain_sandbox,
        "testSandbox": db_sandbox,
    })
}

pub(crate) fn root_notes_query_is_eligible(query: &str, options: RootNotesSectionOptions) -> bool {
    let query = query.trim();
    options.enabled
        && !query.contains('\n')
        && crate::scripts::search::query_meets_min_query_chars(query, options.min_query_chars)
}

/// Get the path to the notes database
fn get_notes_db_path() -> PathBuf {
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        return policy.root().join("notes/db/notes.sqlite");
    }
    if let Ok(path) = std::env::var("SCRIPT_KIT_TEST_NOTES_DB_PATH") {
        return PathBuf::from(path);
    }

    if cfg!(test) {
        return std::env::temp_dir()
            .join("script-kit-gpui-tests")
            .join(std::process::id().to_string())
            .join("db")
            .join("notes.sqlite");
    }

    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit"));

    kit_dir.join("db").join("notes.sqlite")
}

fn get_notes_brain_base_path() -> PathBuf {
    if crate::runtime_policy::is_owned_evaluation() {
        return crate::brain::substrate::BrainPaths::default_kit()
            .base()
            .to_path_buf();
    }
    if let Ok(path) = std::env::var("SCRIPT_KIT_TEST_NOTES_BRAIN_PATH") {
        return PathBuf::from(path);
    }

    if cfg!(test) {
        return std::env::temp_dir()
            .join("script-kit-gpui-tests")
            .join(std::process::id().to_string())
            .join("brain");
    }

    crate::notes::folder::notes_folder()
}

/// Days directory under the notes brain root. The Notes Cmd+P switcher
/// lists day pages read-through from here; day files are never copied into
/// the notes database.
pub(crate) fn notes_brain_days_dir() -> PathBuf {
    get_notes_brain_base_path().join("days")
}

pub(crate) fn note_file_path(id: NoteId) -> Result<Option<PathBuf>> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let slug = lookup_note_slug(&conn, id)?;
    slug.map(|slug| notes_substrate().map(|substrate| substrate.paths().note_file(&slug)))
        .transpose()
}

pub(crate) fn notes_substrate() -> Result<Arc<BrainSubstrate>> {
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy.require_owned_path(&get_notes_brain_base_path())?;
        if let Some(substrate) = NOTES_SUBSTRATE.get() {
            policy.require_owned_path(substrate.paths().base())?;
        }
    }
    Ok(NOTES_SUBSTRATE
        .get_or_init(|| Arc::new(BrainSubstrate::new(get_notes_brain_base_path())))
        .clone())
}

fn note_content_hashes() -> &'static Mutex<HashMap<NoteId, String>> {
    NOTE_CONTENT_HASHES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn content_hash(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    format!("{:x}", digest)
}

fn slug_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
        .filter(|slug| !slug.is_empty())
}

/// A note file's name relative to the notes folder, without `.md`: `Lisbon`, or `Travel/Lisbon`
/// for a note filed in a folder. Files outside the notes folder (the trash) go by their name.
fn note_slug_for_path(substrate: &BrainSubstrate, path: &Path) -> Option<String> {
    let Ok(relative) = path.strip_prefix(substrate.paths().notes_dir()) else {
        return slug_from_path(path);
    };
    let relative = relative.with_extension("");
    let parts = relative
        .components()
        .map(|component| match component {
            std::path::Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let slug = parts.join("/");
    (!slug.is_empty()).then_some(slug)
}

fn is_conflict_copy_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".conflict-"))
}

/// Whether `path` is a note the notes folder lists: a `.md` file under it, in any folder that
/// is not hidden (`.obsidian`, `.trash`), and not a kept conflict copy.
fn is_listed_note_path(notes_dir: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(notes_dir) else {
        return false;
    };
    path.extension().and_then(|ext| ext.to_str()) == Some("md")
        && !is_conflict_copy_path(path)
        && relative.components().all(|component| {
            matches!(component, std::path::Component::Normal(part)
                if !part.to_string_lossy().starts_with('.'))
        })
}

/// Every note file under the notes folder, folders included, in a stable order. Hidden folders
/// and links to elsewhere are left alone.
fn note_files(notes_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut folders = vec![notes_dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("reading notes dir {}", folder.display()))
            }
        };
        for entry in entries {
            let entry = entry.context("reading notes dir entry")?;
            let path = entry.path();
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let file_type = entry.file_type().context("reading notes dir entry type")?;
            if file_type.is_dir() {
                folders.push(path);
            } else if file_type.is_file() && is_listed_note_path(notes_dir, &path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// The body written below the store's front matter. A front matter block the person wrote that
/// isn't YAML stays in the body exactly as written, so it reads back unchanged.
fn note_body_for_file(content: &str) -> String {
    if crate::brain::substrate::frontmatter_is_unreadable(content) {
        return content.to_string();
    }
    metadata::strip_frontmatter(content).to_string()
}

fn user_facing_content(frontmatter: &BrainFrontmatter, body: &str) -> Result<String> {
    frontmatter.merge_into_body(body)
}

/// What the editor shows for a note file read from disk: its words, without the store's own
/// front matter (id, dates, pinned), and keeping any front matter the person wrote. A file
/// another app wrote, without the store's front matter, is shown exactly as written.
pub(crate) fn note_editor_content_from_file(raw: &str) -> Result<String> {
    match notes_substrate()?.parse_document(raw) {
        Ok((frontmatter, body)) => user_facing_content(&frontmatter, &body),
        Err(_) => Ok(raw.to_string()),
    }
}

fn brain_frontmatter_from_note(
    note: &Note,
    preserved: Option<BrainFrontmatter>,
) -> Result<BrainFrontmatter> {
    if crate::brain::substrate::frontmatter_is_unreadable(&note.content) {
        // The person's own block is unreadable and stays in the body as written; the store adds
        // nothing that would have to be merged into it.
        return Ok(BrainFrontmatter {
            id: note.id,
            created: note.created_at,
            updated: note.updated_at,
            tags: Vec::new(),
            aliases: Vec::new(),
            pinned: note.is_pinned,
            source: None,
            why: None,
            extra: Default::default(),
        });
    }
    let parsed = metadata::parse_note_metadata(&note.title, &note.content);
    let (preserved_source, extra) = preserved
        .map(|frontmatter| (frontmatter.source, frontmatter.extra))
        .unwrap_or_default();
    let source = preserved_source.or_else(|| source_from_note_content(&note.content));
    let mut frontmatter = BrainFrontmatter {
        id: note.id,
        created: note.created_at,
        updated: note.updated_at,
        tags: parsed.tags.into_iter().map(|tag| tag.display).collect(),
        aliases: parsed
            .aliases
            .into_iter()
            .filter(|alias| alias.source != "title")
            .map(|alias| alias.alias)
            .collect(),
        pinned: note.is_pinned,
        source,
        why: None,
        extra,
    };
    frontmatter.merge_extra_from_content(&note.content)?;
    Ok(frontmatter)
}

fn source_from_note_content(content: &str) -> Option<String> {
    if let Some(source) = metadata::parse_frontmatter_source(content) {
        return Some(source);
    }
    if let Ok(substrate) = notes_substrate() {
        if let Ok((frontmatter, _)) = substrate.parse_document(content) {
            return frontmatter.source;
        }
    }
    None
}

fn note_from_brain_document(
    frontmatter: BrainFrontmatter,
    body: &str,
    deleted_at: Option<DateTime<Utc>>,
    sort_order: i32,
) -> Result<Note> {
    let content = user_facing_content(&frontmatter, body)?;
    let title = Note::with_content(&content).title;
    Ok(Note {
        id: frontmatter.id,
        title,
        content,
        created_at: frontmatter.created,
        updated_at: frontmatter.updated,
        deleted_at,
        is_pinned: frontmatter.pinned,
        sort_order,
    })
}

fn load_note_from_file(
    substrate: &BrainSubstrate,
    path: &Path,
    deleted_at: Option<DateTime<Utc>>,
    sort_order: i32,
) -> Result<(Note, String, String)> {
    let raw = brain_io::read_private_document(path)
        .with_context(|| format!("reading note file {}", path.display()))?;
    let hash = content_hash(&raw);
    let slug = note_slug_for_path(substrate, path).context("note file missing slug stem")?;
    // A note another app wrote has none of the store's front matter: it is still a note, with
    // an identity from its name and its words exactly as written.
    let (frontmatter, body) = match substrate.parse_document(&raw) {
        Ok(parsed) => parsed,
        Err(_) => (unmanaged_note_frontmatter(path, &slug), raw.clone()),
    };
    let mut note = note_from_brain_document(frontmatter, &body, deleted_at, sort_order)?;
    note.title = file_names::title_for_file(&note.content, file_names::split_folder(&slug).1);
    Ok((note, slug, hash))
}

fn unmanaged_note_frontmatter(path: &Path, stem: &str) -> BrainFrontmatter {
    let metadata = fs::metadata(path).ok();
    let updated: DateTime<Utc> = metadata
        .as_ref()
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(Utc::now);
    let created = metadata
        .as_ref()
        .and_then(|metadata| metadata.created().ok())
        .map(DateTime::<Utc>::from)
        .unwrap_or(updated);
    BrainFrontmatter::new(
        file_names::note_id_for_unmanaged_file(stem),
        created,
        updated,
    )
}

/// A free file name for a new note, after its title.
fn allocate_note_stem(substrate: &BrainSubstrate, title: &str) -> Result<String> {
    let notes_dir = substrate.paths().notes_dir();
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy.require_owned_path(&notes_dir)?;
    }
    Ok(file_names::unused_stem(
        &notes_dir,
        &file_names::stem_for_title(title),
        None,
    ))
}

/// Renames a note's file when its title changed and the file was named after the old one. A
/// file someone named themselves keeps its name, and a note filed in a folder stays in it.
/// Returns the name the file has now.
fn name_note_file_after_title(
    substrate: &BrainSubstrate,
    stem: &str,
    previous_title: &str,
    title: &str,
) -> Result<String> {
    let (folder, own) = file_names::split_folder(stem);
    if file_names::stem_follows_title(own, title)
        || !file_names::stem_follows_title(own, previous_title)
    {
        return Ok(stem.to_string());
    }
    let folder_dir = substrate.paths().notes_dir().join(folder);
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy.require_owned_path(&folder_dir)?;
    }
    let next = format!(
        "{folder}{}",
        file_names::unused_stem(&folder_dir, &file_names::stem_for_title(title), Some(own))
    );
    if next == stem {
        return Ok(next);
    }
    let from = substrate.paths().note_file(stem);
    let to = substrate.paths().note_file(&next);
    if from.exists() {
        brain_io::with_brain_write_lock(|| fs::rename(&from, &to))
            .with_context(|| format!("renaming note file to {}", to.display()))?;
    }
    Ok(next)
}

/// Notes named the way they used to be (`launch-plan.md`) take their title as their name
/// (`Launch plan.md`), so `[[Launch plan]]` finds them in other Markdown apps too.
fn name_legacy_note_files_by_title(substrate: &BrainSubstrate) -> Result<usize> {
    let notes_dir = substrate.paths().notes_dir();
    if !notes_dir.exists() {
        return Ok(0);
    }
    let mut renamed = 0;
    for entry in fs::read_dir(&notes_dir)
        .with_context(|| format!("reading notes dir {}", notes_dir.display()))?
    {
        let path = entry.context("reading notes dir entry")?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md")
            || is_conflict_copy_path(&path)
        {
            continue;
        }
        let Some(stem) = slug_from_path(&path) else {
            continue;
        };
        let Ok(raw) = brain_io::read_private_document(&path) else {
            continue;
        };
        if substrate.parse_document(&raw).is_err() {
            continue;
        }
        let title = Note::title_from_content(&raw);
        let slug = crate::brain::substrate::slugify(&title);
        let legacy = !slug.is_empty()
            && (stem == slug
                || stem
                    .strip_prefix(slug.as_str())
                    .and_then(|rest| rest.strip_prefix('-'))
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())));
        if !legacy || stem == file_names::stem_for_title(&title) {
            continue;
        }
        let next = allocate_note_stem(substrate, &title)?;
        let to = substrate.paths().note_file(&next);
        brain_io::with_brain_write_lock(|| fs::rename(&path, &to))
            .with_context(|| format!("naming note file {}", to.display()))?;
        renamed += 1;
    }
    Ok(renamed)
}

fn lookup_note_slug(conn: &Connection, note_id: NoteId) -> Result<Option<String>> {
    conn.query_row(
        "SELECT file_slug FROM notes WHERE id = ?1",
        params![note_id.as_str()],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .context("Failed to look up note slug")
    .map(|row| row.flatten())
}

/// Where a save writes: the note's file name, whether the save brings it back from the trash,
/// and the title it had before (`None` for a new note).
fn resolve_note_save_target(
    conn: &Connection,
    note: &Note,
) -> Result<(String, bool, Option<String>)> {
    let existing = conn
        .query_row(
            "SELECT file_slug, deleted_at IS NOT NULL, title FROM notes WHERE id = ?1",
            params![note.id.as_str()],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .context("Failed to look up note save target")?;
    let (slug, was_deleted, previous_title) = match existing {
        Some((slug, was_deleted, title)) => (slug, was_deleted, Some(title)),
        None => (None, false, None),
    };
    let slug = match slug {
        Some(slug) => slug,
        None => allocate_note_stem(notes_substrate()?.as_ref(), &note.title)?,
    };
    Ok((
        slug,
        was_deleted && note.deleted_at.is_none(),
        previous_title,
    ))
}

fn write_conflict_copy(path: &Path, contents: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let Some(conflict_path) = write_conflict_copy_at(path, contents, &timestamp)? else {
        return Ok(());
    };
    let original = crate::logging::log_private_user_value(&path.display().to_string());
    let conflict = crate::logging::log_private_user_value(&conflict_path.display().to_string());
    warn!(
        original_bytes = original.raw_bytes,
        original_sha256 = %original.sha256,
        conflict_bytes = conflict.raw_bytes,
        conflict_sha256 = %conflict.sha256,
        "External note edit conflict preserved as conflict copy"
    );
    Ok(())
}

fn write_conflict_copy_at(path: &Path, contents: &str, timestamp: &str) -> Result<Option<PathBuf>> {
    use std::io::Write as _;

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(None);
    };
    let parent = path.parent().context("conflict copy path missing parent")?;
    for attempt in 1..=1024 {
        let suffix = if attempt == 1 {
            String::new()
        } else {
            format!("-{attempt}")
        };
        let conflict_path = parent.join(format!("{stem}.conflict-{timestamp}{suffix}.md"));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        match options.open(&conflict_path) {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(contents.as_bytes())
                    .and_then(|_| file.sync_all())
                {
                    let _ = fs::remove_file(&conflict_path);
                    return Err(error).with_context(|| {
                        format!("writing private conflict copy {}", conflict_path.display())
                    });
                }
                return Ok(Some(conflict_path));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("creating private conflict copy {}", conflict_path.display())
                });
            }
        }
    }
    anyhow::bail!("no unused private note conflict filename remained")
}

fn guard_external_edit_before_write(path: &Path, note_id: NoteId) -> Result<()> {
    let Some(disk) = brain_io::read_private_document_if_present(path)? else {
        return Ok(());
    };
    let disk_hash = content_hash(&disk);
    let known_hash = note_content_hashes()
        .lock()
        .map_err(db_lock_err)?
        .get(&note_id)
        .cloned();
    if let Some(known_hash) = known_hash {
        if known_hash != disk_hash {
            write_conflict_copy(path, &disk)?;
        }
    }
    Ok(())
}

fn remember_note_hash(note_id: NoteId, hash: String) {
    if let Ok(mut guard) = note_content_hashes().lock() {
        guard.insert(note_id, hash);
    }
}

fn forget_note_hash(note_id: NoteId) {
    if let Ok(mut guard) = note_content_hashes().lock() {
        guard.remove(&note_id);
    }
}

fn deleted_at_from_trash_path(path: &Path) -> DateTime<Utc> {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| Utc::now())
}

fn preserve_note_cart_items_for_rebuild(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS temp.note_cart_items_rebuild_snapshot;
        CREATE TEMP TABLE note_cart_items_rebuild_snapshot AS
        SELECT id, note_id, label, payload_json, created_at, updated_at, sort_order
        FROM note_cart_items;
        "#,
    )
    .context("Failed to preserve note cart items before notes index rebuild")?;
    Ok(())
}

fn clear_index_tables(conn: &Connection) -> Result<()> {
    preserve_note_cart_items_for_rebuild(conn)?;
    conn.execute_batch(
        r#"
        DELETE FROM note_links;
        DELETE FROM note_aliases;
        DELETE FROM note_tags;
        DELETE FROM notes;
        "#,
    )
    .context("Failed to clear notes index tables")?;
    Ok(())
}

fn restore_note_cart_items_after_rebuild(conn: &Connection) -> Result<()> {
    let restored = conn
        .execute(
            r#"
            INSERT OR REPLACE INTO note_cart_items
                (id, note_id, label, payload_json, created_at, updated_at, sort_order)
            SELECT
                snapshot.id,
                snapshot.note_id,
                snapshot.label,
                snapshot.payload_json,
                snapshot.created_at,
                snapshot.updated_at,
                snapshot.sort_order
            FROM temp.note_cart_items_rebuild_snapshot snapshot
            WHERE EXISTS (
                SELECT 1 FROM notes WHERE notes.id = snapshot.note_id
            )
            "#,
            [],
        )
        .context("Failed to restore note cart items after notes index rebuild")?;
    let pruned = conn
        .execute(
            r#"
            DELETE FROM note_cart_items
            WHERE NOT EXISTS (
                SELECT 1 FROM notes WHERE notes.id = note_cart_items.note_id
            )
            "#,
            [],
        )
        .context("Failed to prune orphaned note cart items after notes index rebuild")?;
    conn.execute_batch("DROP TABLE IF EXISTS temp.note_cart_items_rebuild_snapshot;")
        .context("Failed to drop note cart item rebuild snapshot")?;
    debug!(
        restored,
        pruned, "Restored note cart items after notes index rebuild"
    );
    Ok(())
}

fn schema_needs_rebuild(conn: &Connection) -> Result<bool> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);
    if version != NOTES_INDEX_SCHEMA_VERSION {
        return Ok(true);
    }

    let has_file_slug: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('notes') WHERE name = 'file_slug'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(has_file_slug == 0)
}

/// Rebuild the sqlite index from canonical markdown files.
///
/// Contract: delete the DB, rebuild from files, nothing user-visible is lost.
pub fn rebuild_index_from_files() -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    rebuild_index_from_files_with_conn(&conn)
}

fn rebuild_index_from_files_with_conn(conn: &Connection) -> Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;")
        .context("Failed to begin notes index rebuild transaction")?;
    let result = rebuild_index_from_files_with_conn_inner(conn);
    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .context("Failed to commit notes index rebuild transaction")?;
            invalidate_root_notes_search_cache();
            info!("Rebuilt notes sqlite index from brain files");
            Ok(())
        }
        Err(error) => {
            if let Err(rollback_error) = conn.execute_batch("ROLLBACK;") {
                warn!(
                    %rollback_error,
                    "Failed to roll back notes index rebuild transaction"
                );
            }
            Err(error)
        }
    }
}

fn indexed_note_slugs_with_conn(conn: &Connection) -> Result<HashMap<NoteId, String>> {
    let mut stmt = conn
        .prepare("SELECT id, file_slug FROM notes WHERE file_slug IS NOT NULL")
        .context("Failed to prepare indexed note files query")?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .context("Failed to query indexed note files")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to read indexed note files")?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, slug)| NoteId::parse(&id).map(|id| (id, slug)))
        .collect())
}

/// Files that carry the same note id — a copy made in Finder or Obsidian keeps the original's
/// front matter — each become a note of their own. The file the index already knew keeps the
/// id (else the first by name); every other copy gets a new id written into its front matter.
/// A copy whose new id can't be written is left out of the index rather than replacing the
/// original.
fn give_duplicates_their_own_ids(
    substrate: &BrainSubstrate,
    loaded: &mut Vec<(PathBuf, (Note, String, String))>,
    indexed_slugs: &HashMap<NoteId, String>,
) {
    let mut holders: HashMap<NoteId, usize> = loaded
        .iter()
        .enumerate()
        .filter(|(_, (_, (note, slug, _)))| indexed_slugs.get(&note.id) == Some(slug))
        .map(|(index, (_, (note, _, _)))| (note.id, index))
        .collect();
    let mut unwritable = Vec::new();
    for index in 0..loaded.len() {
        let id = loaded[index].1 .0.id;
        match holders.get(&id) {
            None => {
                holders.insert(id, index);
            }
            Some(&holder) if holder == index => {}
            Some(_) => match give_note_file_its_own_id(substrate, &loaded[index].0) {
                Ok(reloaded) => {
                    holders.insert(reloaded.0.id, index);
                    loaded[index].1 = reloaded;
                }
                Err(error) => {
                    warn!(path = %loaded[index].0.display(), %error, "Could not give a copied note its own id");
                    unwritable.push(index);
                }
            },
        }
    }
    for index in unwritable.into_iter().rev() {
        loaded.remove(index);
    }
}

fn give_note_file_its_own_id(
    substrate: &BrainSubstrate,
    path: &Path,
) -> Result<(Note, String, String)> {
    brain_io::with_brain_write_lock(|| -> Result<()> {
        let raw = brain_io::read_private_document(path)?;
        let (mut frontmatter, body) = substrate.parse_document(&raw)?;
        frontmatter.id = NoteId::new();
        substrate.write_document(path, &frontmatter, &body)
    })?;
    load_note_from_file(substrate, path, None, 0)
}

fn rebuild_index_from_files_with_conn_inner(conn: &Connection) -> Result<()> {
    let indexed_slugs = indexed_note_slugs_with_conn(conn)?;
    clear_index_tables(conn)?;

    let substrate = notes_substrate()?;
    match name_legacy_note_files_by_title(&substrate) {
        Ok(0) => {}
        Ok(renamed) => info!(renamed, "Named note files after their titles"),
        Err(error) => warn!(%error, "Could not name every note file after its title"),
    }
    let mut loaded = Vec::new();
    for path in note_files(&substrate.paths().notes_dir())? {
        // Skip-and-log a single malformed/newer note file rather than
        // aborting the whole rebuild — one hand-edited or version-skewed
        // file must not be able to wedge the entire notes index (which
        // rebuilds on init after a corrupt-DB recovery).
        match load_note_from_file(&substrate, &path, None, 0) {
            Ok(note) => loaded.push((path, note)),
            Err(error) => warn!(
                path = %path.display(),
                %error,
                "Skipping unreadable note during index rebuild"
            ),
        }
    }
    give_duplicates_their_own_ids(&substrate, &mut loaded, &indexed_slugs);
    for (_, (note, slug, hash)) in loaded {
        upsert_note_index_with_conn(conn, &note, &slug, &hash)?;
        remember_note_hash(note.id, hash);
    }

    let trash_dir = substrate.paths().trash_dir();
    if trash_dir.exists() {
        for entry in fs::read_dir(&trash_dir)
            .with_context(|| format!("reading trash dir {}", trash_dir.display()))?
        {
            let entry = entry.context("reading trash dir entry")?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if is_conflict_copy_path(&path) {
                continue;
            }
            let deleted_at = Some(deleted_at_from_trash_path(&path));
            let (note, slug, hash) = match load_note_from_file(&substrate, &path, deleted_at, 0) {
                Ok(loaded) => loaded,
                Err(error) => {
                    warn!(
                        path = %path.display(),
                        %error,
                        "Skipping unreadable trashed note during index rebuild"
                    );
                    continue;
                }
            };
            upsert_note_index_with_conn(conn, &note, &slug, &hash)?;
            remember_note_hash(note.id, hash);
        }
    }

    restore_note_cart_items_after_rebuild(conn)?;
    recompute_all_note_link_targets_with_conn(conn)?;
    rebuild_notes_search_index_with_conn(conn)?;
    conn.execute(
        &format!("PRAGMA user_version = {NOTES_INDEX_SCHEMA_VERSION}"),
        [],
    )
    .context("Failed to set notes index schema version")?;
    Ok(())
}

fn upsert_note_index_with_conn(
    conn: &Connection,
    note: &Note,
    slug: &str,
    hash: &str,
) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO notes (
            id, title, content, created_at, updated_at, deleted_at,
            is_pinned, sort_order, file_slug, content_hash
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            is_pinned = excluded.is_pinned,
            sort_order = excluded.sort_order,
            file_slug = excluded.file_slug,
            content_hash = excluded.content_hash
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
            slug,
            hash,
        ],
    )
    .context("Failed to upsert note index row")?;
    replace_note_metadata_with_conn(conn, note)?;
    Ok(())
}

fn write_canonical_note_file(
    substrate: &BrainSubstrate,
    note: &Note,
    slug: &str,
) -> Result<String> {
    let path = substrate.paths().note_file(slug);
    let preserved_frontmatter = brain_io::read_private_document_if_present(&path)?
        .and_then(|raw| substrate.parse_document(&raw).ok())
        .map(|(frontmatter, _)| frontmatter);

    guard_external_edit_before_write(&path, note.id)?;

    let frontmatter = brain_frontmatter_from_note(note, preserved_frontmatter)?;
    let body = note_body_for_file(&note.content);
    substrate.write_document(&path, &frontmatter, &body)?;

    let raw = brain_io::read_private_document(&path)
        .with_context(|| format!("reading note file after write {}", path.display()))?;
    Ok(content_hash(&raw))
}

fn trash_canonical_note_file(substrate: &BrainSubstrate, slug: &str) -> Result<()> {
    let path = substrate.paths().note_file(slug);
    if path.exists() {
        substrate.trash(&path)?;
    }
    Ok(())
}

fn restore_canonical_note_file(
    substrate: &BrainSubstrate,
    note_id: NoteId,
    slug: &str,
) -> Result<()> {
    let destination = substrate.paths().note_file(slug);
    let trash_dir = substrate.paths().trash_dir();
    if !trash_dir.exists() {
        return Ok(());
    }
    crate::atomic_file::ensure_private_directory(&trash_dir)
        .context("Failed to prepare private Notes trash directory")?;

    let mut matching_path = None;
    for entry in fs::read_dir(&trash_dir)
        .with_context(|| format!("reading trash dir {}", trash_dir.display()))?
    {
        let entry = entry.context("reading trash entry")?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") {
            continue;
        }
        let raw = brain_io::read_private_document(&path)
            .with_context(|| format!("reading trash document {}", path.display()))?;
        // Day pages share the trash but have no Notes frontmatter identity.
        if metadata::strip_frontmatter(&raw).len() == raw.len() {
            continue;
        }
        let Ok((frontmatter, _)) = substrate.parse_document(&raw) else {
            continue;
        };
        if frontmatter.id == note_id {
            anyhow::ensure!(
                matching_path.is_none(),
                "multiple trash documents match the note identity"
            );
            matching_path = Some(path);
        }
    }
    if let Some(path) = matching_path {
        substrate.restore(&path, &destination)?;
    }
    Ok(())
}

fn delete_trashed_note_file(substrate: &BrainSubstrate, slug: &str) -> Result<()> {
    let trash_dir = substrate.paths().trash_dir();
    if !trash_dir.exists() {
        return Ok(());
    }
    crate::atomic_file::ensure_private_directory(&trash_dir)
        .context("Failed to prepare private Notes trash directory")?;
    let slug = file_names::split_folder(slug).1;
    let suffix = format!("{slug}.md");
    for entry in fs::read_dir(&trash_dir)
        .with_context(|| format!("reading trash dir {}", trash_dir.display()))?
    {
        let entry = entry.context("reading trash entry")?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name == suffix || name.starts_with(&format!("{slug}-")) {
            fs::remove_file(&path)
                .with_context(|| format!("removing trashed note {}", path.display()))?;
            return Ok(());
        }
    }
    Ok(())
}

fn reindex_external_note_file(path: &Path) -> Result<()> {
    let substrate = notes_substrate()?;
    let notes_dir = substrate.paths().notes_dir();
    if !substrate.paths().contains(path) || !is_listed_note_path(&notes_dir, path) {
        return Ok(());
    }

    let Some(raw) = brain_io::read_private_document_if_present(path)? else {
        if let Some(slug) = note_slug_for_path(&substrate, path) {
            let db = get_db()?;
            let conn = db.lock().map_err(db_lock_err)?;
            if let Some(note_id) = conn
                .query_row(
                    "SELECT id FROM notes WHERE file_slug = ?1",
                    params![slug],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .context("Failed to look up note id for deleted file")?
            {
                if let Some(id) = NoteId::parse(&note_id) {
                    conn.execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
                        .context("Failed to remove deleted note from index")?;
                    forget_note_hash(id);
                    invalidate_root_notes_search_cache();
                }
            }
        }
        return Ok(());
    };
    let mut hash = content_hash(&raw);
    let (mut note, mut slug, _) = load_note_from_file(&substrate, path, None, 0)?;

    // A copy made in Finder or Obsidian carries the original's id. While the original's file
    // is still there, the copy becomes a note of its own; when it isn't, the note moved.
    let indexed_slug = {
        let db = get_db()?;
        let conn = db.lock().map_err(db_lock_err)?;
        indexed_note_slugs_with_conn(&conn)?.remove(&note.id)
    };
    if indexed_slug.as_ref().is_some_and(|indexed| {
        indexed != &slug
            && file_has_note_id(&substrate, &substrate.paths().note_file(indexed), note.id)
    }) {
        let (copy, copy_slug, copy_hash) = give_note_file_its_own_id(&substrate, path)?;
        (note, slug, hash) = (copy, copy_slug, copy_hash);
    }

    let known_hash = note_content_hashes()
        .lock()
        .map_err(db_lock_err)?
        .get(&note.id)
        .cloned();
    if known_hash.as_deref() == Some(hash.as_str()) && indexed_slug.as_deref() == Some(&slug) {
        return Ok(());
    }

    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let sort_order = conn
        .query_row(
            "SELECT sort_order FROM notes WHERE id = ?1",
            params![note.id.as_str()],
            |row| row.get::<_, i32>(0),
        )
        .unwrap_or(0);
    let mut indexed = note;
    indexed.sort_order = sort_order;
    upsert_note_index_with_conn(&conn, &indexed, &slug, &hash)?;
    // Indexing an external version does not mean a writer adopted it: a save that starts from
    // older text still keeps this version (the editor merges it; others keep a copy).
    invalidate_root_notes_search_cache();
    debug!(note_id = %indexed.id, file = %path.display(), "Reindexed externally edited note");
    Ok(())
}

fn file_has_note_id(substrate: &BrainSubstrate, path: &Path, id: NoteId) -> bool {
    brain_io::read_private_document_if_present(path)
        .ok()
        .flatten()
        .and_then(|raw| substrate.parse_document(&raw).ok())
        .is_some_and(|(frontmatter, _)| frontmatter.id == id)
}

/// The file that holds note `id` now, wherever it went: the indexed file when it still has the
/// note, else any note file carrying that id (renamed or moved in Finder or Obsidian before the
/// index caught up). `None` when no file holds it.
fn locate_note_file(substrate: &BrainSubstrate, id: NoteId) -> Result<Option<PathBuf>> {
    if let Some(path) = note_file_path(id)? {
        if file_has_note_id(substrate, &path, id) {
            return Ok(Some(path));
        }
    }
    Ok(note_files(&substrate.paths().notes_dir())?
        .into_iter()
        .find(|path| file_has_note_id(substrate, path, id)))
}

/// What an editor save left on disk.
#[derive(Debug, Clone)]
pub(crate) struct SavedNote {
    pub(crate) path: PathBuf,
    pub(crate) title: String,
    /// The note's text now, which differs from the editor's when changes made elsewhere were
    /// merged in.
    pub(crate) content: String,
    /// Lines changed both in the editor and elsewhere were kept in both versions.
    pub(crate) kept_both: bool,
}

/// Saves the editor's text for note `id`. `base` is the text the editor last loaded or saved.
///
/// The save follows the note's file if another app renamed or moved it, recreates it if it was
/// deleted, and merges line by line with any change made to it elsewhere since `base`: edits
/// to different lines all land, and lines both sides changed are kept in both versions.
pub(crate) fn save_note_from_editor(id: NoteId, base: &str, content: &str) -> Result<SavedNote> {
    let substrate = notes_substrate()?;
    let located = locate_note_file(&substrate, id)?;
    if let Some(path) = &located {
        if note_file_path(id)?.as_deref() != Some(path.as_path()) {
            reindex_external_note_file(path)?;
        }
    }

    let mut text = content.to_string();
    let mut kept_both = false;
    if let Some(path) = &located {
        if let Some(raw) = brain_io::read_private_document_if_present(path)? {
            let theirs = note_editor_content_from_file(&raw)?;
            if theirs != base && theirs != content {
                let merged = super::merge::merge_lines(base, content, &theirs);
                text = merged.text;
                kept_both = merged.kept_both;
            }
            // The file's current version is part of what gets written.
            remember_note_hash(id, content_hash(&raw));
        }
    }

    let mut note = match get_note(id)? {
        Some(note) => note,
        // Deleted elsewhere and already dropped from the index: bring it back under its id.
        None => {
            let mut note = Note::with_content(text.clone());
            note.id = id;
            note
        }
    };
    note.set_content(&text);
    save_note(&note)?;
    verify_saved_note_content(id, &text)?;
    let saved = get_note(id)?.context("saved note missing from the index")?;
    let path = note_file_path(id)?.context("saved note file missing")?;
    Ok(SavedNote {
        path,
        title: saved.title,
        content: text,
        kept_both,
    })
}

fn start_notes_dir_watcher() {
    // Owned fixtures use synchronous canonical saves/searches, not an unbounded watcher.
    if crate::runtime_policy::is_owned_evaluation() {
        return;
    }
    if NOTES_DIR_WATCHER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    let Ok(substrate) = notes_substrate() else {
        return;
    };
    let notes_dir = substrate.paths().notes_dir();
    if let Err(error) = crate::atomic_file::ensure_private_directory(&notes_dir) {
        warn!(%error, "Failed to prepare private notes watcher directory");
        NOTES_DIR_WATCHER_STARTED.store(false, Ordering::SeqCst);
        return;
    }

    let spawn_result = std::thread::Builder::new()
        .name("notes-brain-watcher".to_string())
        .spawn(move || notes_dir_watcher_loop(notes_dir));

    if let Err(error) = spawn_result {
        warn!(%error, "Failed to start notes brain directory watcher");
        NOTES_DIR_WATCHER_STARTED.store(false, Ordering::SeqCst);
    }
}

fn notes_dir_watcher_loop(notes_dir: PathBuf) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = match recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(watcher) => watcher,
        Err(error) => {
            warn!(%error, "Failed to create notes brain watcher");
            NOTES_DIR_WATCHER_STARTED.store(false, Ordering::SeqCst);
            return;
        }
    };

    if let Err(error) = watcher.watch(&notes_dir, RecursiveMode::Recursive) {
        warn!(
            %error,
            dir = %notes_dir.display(),
            "Failed to watch notes brain directory"
        );
        NOTES_DIR_WATCHER_STARTED.store(false, Ordering::SeqCst);
        return;
    }

    let debounce = Duration::from_millis(crate::config::defaults::DEFAULT_WATCHER_DEBOUNCE_MS);
    let mut pending: HashMap<PathBuf, Instant> = HashMap::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    // A folder moved, renamed or removed in Finder reports only the folder.
                    let is_note = path.extension().and_then(|ext| ext.to_str()) == Some("md");
                    if is_note || path.extension().is_none() {
                        pending.insert(path, Instant::now() + debounce);
                    }
                }
            }
            Ok(Err(error)) => {
                warn!(%error, "Notes brain watcher notify error");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        let ready: Vec<PathBuf> = pending
            .iter()
            .filter_map(|(path, deadline)| (*deadline <= now).then_some(path.clone()))
            .collect();
        for path in ready {
            pending.remove(&path);
            let result = if path.extension().is_none() {
                reindex_external_note_folder(&notes_dir, &path)
            } else {
                reindex_external_note_file(&path)
            };
            if let Err(error) = result {
                warn!(%error, file = %path.display(), "Failed to reindex external note edit");
            }
        }
    }
}

/// Reindexes every note in a folder that appeared, moved or disappeared under the notes folder.
fn reindex_external_note_folder(notes_dir: &Path, folder: &Path) -> Result<()> {
    let Ok(relative) = folder.strip_prefix(notes_dir) else {
        return Ok(());
    };
    if relative.as_os_str().is_empty() {
        return Ok(());
    }
    if folder.is_dir() {
        for path in note_files(folder)? {
            if is_listed_note_path(notes_dir, &path) {
                reindex_external_note_file(&path)?;
            }
        }
        return Ok(());
    }
    let prefix = format!("{}/", relative.to_string_lossy());
    let gone: Vec<String> = {
        let db = get_db()?;
        let conn = db.lock().map_err(db_lock_err)?;
        indexed_note_slugs_with_conn(&conn)?
            .into_values()
            .filter(|slug| slug.starts_with(&prefix))
            .collect()
    };
    for slug in gone {
        reindex_external_note_file(&notes_dir.join(format!("{slug}.md")))?;
    }
    Ok(())
}

include!("storage_schema.rs");

/// Initialize the notes database
///
/// This function is idempotent - it's safe to call multiple times.
/// If the database is already initialized, it verifies schema and triggers
/// are up-to-date on the existing connection.
pub fn init_notes_db() -> Result<()> {
    let _init_guard = NOTES_DB_INIT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let substrate = notes_substrate()?;
    crate::atomic_file::ensure_private_directory(substrate.paths().base())
        .context("Failed to prepare private Notes brain directory")?;
    crate::atomic_file::ensure_private_directory(&substrate.paths().notes_dir())
        .context("Failed to prepare private Notes document directory")?;
    crate::atomic_file::ensure_private_directory(&substrate.paths().trash_dir())
        .context("Failed to prepare private Notes trash directory")?;

    if let Some(db) = NOTES_DB.get() {
        let conn = db.lock().map_err(db_lock_err)?;

        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .context("Failed to enable notes foreign keys")?;
        ensure_notes_schema(&conn)?;
        if schema_needs_rebuild(&conn)? {
            rebuild_index_from_files_with_conn(&conn)
                .context("Failed to rebuild notes index from brain files")?;
        } else {
            backfill_note_metadata_with_conn(&conn)
                .context("Failed to backfill notes metadata schema")?;
        }
        start_notes_dir_watcher();
        debug!("Notes database already initialized, schema verified");
        return Ok(());
    }

    let db_path = get_notes_db_path();

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).context("Failed to create notes db directory")?;
    }

    let (conn, _) = open_or_recover_notes_db(&db_path)?;

    // Markdown is canonical. Reconcile on every cold start so offline edits
    // replace stale index rows and seed the save-conflict baselines.
    rebuild_index_from_files_with_conn(&conn)
        .context("Failed to rebuild notes index from brain files")?;

    info!(db_path = %db_path.display(), "Notes database initialized");

    let _ = NOTES_DB.get_or_init(|| Arc::new(Mutex::new(conn)));

    start_notes_dir_watcher();
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
    let conn = db.lock().map_err(db_lock_err)?;
    rebuild_notes_search_index_with_conn(&conn)
}

/// Get a reference to the notes database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    NOTES_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Notes database not initialized"))
}

/// Opens the notes store if nothing has yet. Cheap once it is open: saves call this on every
/// keystroke's autosave, so it never repeats `init_notes_db`'s schema checks.
pub(crate) fn ensure_notes_db_open() -> Result<()> {
    if NOTES_DB.get().is_some() {
        return Ok(());
    }
    init_notes_db()
}

/// Save a note (insert or update)
pub fn save_note(note: &Note) -> Result<()> {
    let substrate = notes_substrate()?;
    let db = get_db()?;
    let mut conn = db.lock().map_err(db_lock_err)?;

    // A note's title is its `# Heading`, else its file name; the file is named after it.
    let heading = file_names::heading_title(&note.content);
    let mut titled = note.clone();
    if let Some(heading) = &heading {
        titled.title = heading.clone();
    }
    let (mut slug, is_restoring, previous_title) = resolve_note_save_target(&conn, &titled)?;
    if is_restoring {
        let destination = substrate.paths().note_file(&slug);
        if let Some(raw) = brain_io::read_private_document_if_present(&destination)? {
            let other_note = substrate
                .parse_document(&raw)
                .map_or(true, |(existing, _)| existing.id != note.id);
            if other_note {
                slug = allocate_note_stem(&substrate, &titled.title)?;
            }
        }
    } else if note.deleted_at.is_none() {
        if let (Some(heading), Some(previous)) = (heading.as_deref(), previous_title.as_deref()) {
            slug = name_note_file_after_title(&substrate, &slug, previous, heading)?;
        }
    }
    if heading.is_none() && previous_title.is_some() {
        titled.title = slug.clone();
    }
    let note = &titled;
    let hash = if note.deleted_at.is_some() {
        trash_canonical_note_file(&substrate, &slug)?;
        String::new()
    } else {
        if is_restoring {
            restore_canonical_note_file(&substrate, note.id, &slug)?;
        }
        write_canonical_note_file(&substrate, note, &slug)?
    };

    let tx = conn
        .transaction()
        .context("Failed to start note save transaction")?;

    upsert_note_index_with_conn(&tx, note, &slug, &hash)?;
    tx.commit()
        .context("Failed to commit note save transaction")?;

    if !hash.is_empty() {
        remember_note_hash(note.id, hash);
    } else if note.deleted_at.is_some() {
        forget_note_hash(note.id);
    }

    debug!(note_id = %note.id, title = %note.title, slug = %slug, "Note saved to brain file");
    crate::dev_marker::log_marker_note_explanation_if_ready(&note.id, &note.content);
    invalidate_root_notes_search_cache();
    Ok(())
}

/// After a note's file is renamed from `old` to `new`, points the links in other notes and in
/// the day pages in `days_dir` at the new name, as other Markdown apps do. Links stay as they
/// are when another note still answers to `old`. Returns how many files changed.
pub(crate) fn rewrite_note_links(
    renamed: NoteId,
    old: &str,
    new: &str,
    days_dir: &Path,
) -> Result<usize> {
    if metadata::slugify_note_ref(old) == metadata::slugify_note_ref(new)
        || resolve_note_ref(old)? != NoteRefResolution::NotFound
    {
        return Ok(0);
    }
    let mut changed = 0;
    for mut note in get_all_notes()? {
        if note.id == renamed {
            continue;
        }
        if let Some(content) = file_names::rewrite_wiki_links(&note.content, old, new) {
            note.set_content(&content);
            save_note(&note)?;
            changed += 1;
        }
    }
    if let Some(policy) = crate::runtime_policy::owned_evaluation() {
        policy.require_owned_path(days_dir)?;
    }
    let Ok(entries) = fs::read_dir(days_dir) else {
        return Ok(changed);
    };
    for entry in entries {
        let path = entry.context("reading days dir entry")?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let rewrote = brain_io::with_brain_write_lock(|| -> Result<bool> {
            let Some(content) = brain_io::read_private_document_if_present(&path)? else {
                return Ok(false);
            };
            let Some(rewritten) = file_names::rewrite_wiki_links(&content, old, new) else {
                return Ok(false);
            };
            brain_io::atomic_write(&path, &rewritten)?;
            Ok(true)
        })
        .with_context(|| format!("updating links in {}", path.display()))?;
        changed += usize::from(rewrote);
    }
    invalidate_root_notes_search_cache();
    Ok(changed)
}

/// Read back the canonical file and index after a delivery save.
pub(crate) fn verify_saved_note_content(id: NoteId, expected: &str) -> Result<()> {
    let note = get_note(id)?.context("saved Notes index entry missing")?;
    anyhow::ensure!(
        note.content == expected && note.deleted_at.is_none(),
        "saved Notes index content differs"
    );
    let path = note_file_path(id)?.context("saved Notes canonical path missing")?;
    let raw = brain_io::read_private_document(&path)?;
    let (frontmatter, _) = notes_substrate()?.parse_document(&raw)?;
    anyhow::ensure!(
        frontmatter.id == id,
        "saved Notes canonical identity differs"
    );
    let expected_raw = brain_frontmatter_from_note(&note, Some(frontmatter))?
        .render(&note_body_for_file(expected))?;
    anyhow::ensure!(raw == expected_raw, "saved Notes canonical content differs");
    Ok(())
}

fn replace_note_metadata_with_conn(conn: &Connection, note: &Note) -> Result<()> {
    let mut parsed = metadata::parse_note_metadata(&note.title, &note.content);
    // Other Markdown apps link a note by its file name, which can differ from its title
    // (`Trip ideas 2.md`), so `[[Trip ideas 2]]` finds it too.
    if let Some(stem) = lookup_note_slug(conn, note.id)? {
        let slug = metadata::slugify_note_ref(&stem);
        if !slug.is_empty() && parsed.aliases.iter().all(|alias| alias.slug != slug) {
            parsed.aliases.push(metadata::ParsedAlias {
                alias: stem,
                slug,
                source: "file",
            });
        }
    }
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "DELETE FROM note_tags WHERE note_id = ?1",
        params![note.id.as_str()],
    )
    .context("Failed to clear note tags")?;
    conn.execute(
        "DELETE FROM note_aliases WHERE note_id = ?1",
        params![note.id.as_str()],
    )
    .context("Failed to clear note aliases")?;
    conn.execute(
        "DELETE FROM note_links WHERE source_note_id = ?1",
        params![note.id.as_str()],
    )
    .context("Failed to clear note links")?;

    for tag in parsed.tags {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO note_tags (note_id, tag, normalized_tag, source, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                note.id.as_str(),
                tag.display,
                tag.normalized,
                tag.source,
                now,
            ],
        )
        .context("Failed to insert note tag")?;
    }

    for alias in parsed.aliases {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO note_aliases (note_id, alias, slug, source, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![note.id.as_str(), alias.alias, alias.slug, alias.source, now,],
        )
        .context("Failed to insert note alias")?;
    }

    for link in parsed.links {
        let target_note_id = resolve_note_link_target(conn, &link.target_slug)?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO note_links
                (source_note_id, target_note_id, target_ref, target_slug, label, kind, byte_start, byte_end, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                note.id.as_str(),
                target_note_id.map(|id| id.as_str()),
                link.target_ref,
                link.target_slug,
                link.label,
                link.kind,
                link.byte_start as i64,
                link.byte_end as i64,
                now,
            ],
        )
        .context("Failed to insert note link")?;
    }

    recompute_all_note_link_targets_with_conn(conn)?;
    Ok(())
}

fn resolve_note_link_target(conn: &Connection, target_slug: &str) -> Result<Option<NoteId>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT note_id
            FROM note_aliases
            WHERE slug = ?1
            ORDER BY source = 'title' DESC, updated_at DESC
            LIMIT 2
            "#,
        )
        .context("Failed to prepare note link resolution query")?;
    let matches = stmt
        .query_map(params![target_slug], |row| row.get::<_, String>(0))
        .context("Failed to query note aliases for link resolution")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note alias matches")?;

    if matches.len() == 1 {
        Ok(NoteId::parse(&matches[0]))
    } else {
        Ok(None)
    }
}

fn resolve_unresolved_links_with_conn(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT target_slug
            FROM note_links
            WHERE target_note_id IS NULL
            "#,
        )
        .context("Failed to prepare unresolved note links query")?;
    let slugs = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .context("Failed to query unresolved note links")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect unresolved note links")?;
    drop(stmt);

    for slug in slugs {
        if let Some(target_id) = resolve_note_link_target(conn, &slug)? {
            conn.execute(
                "UPDATE note_links SET target_note_id = ?1 WHERE target_slug = ?2 AND target_note_id IS NULL",
                params![target_id.as_str(), slug],
            )
            .context("Failed to resolve note links")?;
        }
    }

    Ok(())
}

fn recompute_all_note_link_targets_with_conn(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE note_links SET target_note_id = NULL", [])
        .context("Failed to clear note link targets")?;
    resolve_unresolved_links_with_conn(conn)
}

fn backfill_note_metadata_with_conn(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            "#,
        )
        .context("Failed to prepare notes metadata backfill query")?;
    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query notes for metadata backfill")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect notes for metadata backfill")?;
    drop(stmt);

    for note in notes {
        replace_note_metadata_with_conn(conn, &note)?;
    }

    recompute_all_note_link_targets_with_conn(conn)?;
    Ok(())
}

/// Get a note by ID
/// Count active (non-deleted) notes carrying the given normalized tag.
///
/// Used to decide whether instruction notes should be staged on new Agent
/// Chat threads without paying for a full list read.
pub(crate) fn count_active_notes_with_tag(tag: &str) -> Result<u64> {
    let Some(normalized) = metadata::normalize_tag(tag) else {
        return Ok(0);
    };

    ensure_notes_db_open()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    let count: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(DISTINCT t.note_id)
            FROM note_tags t
            JOIN notes n ON n.id = t.note_id
            WHERE t.normalized_tag = ?1 AND n.deleted_at IS NULL
            "#,
            params![normalized],
            |row| row.get(0),
        )
        .context("Failed to count notes with tag")?;

    Ok(count.max(0) as u64)
}

/// Result of resolving a wiki-link target reference against note aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoteRefResolution {
    Unique(NoteId),
    Ambiguous,
    NotFound,
}

/// Resolve a `[[wiki link]]` target to a note: a note's file name first (`Lisbon`, or
/// `Travel/Lisbon` for one in a folder), as Obsidian does, then a title or alias.
pub(crate) fn resolve_note_ref(target: &str) -> Result<NoteRefResolution> {
    let slug = metadata::slugify_note_ref(target);
    if slug.is_empty() {
        return Ok(NoteRefResolution::NotFound);
    }

    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    let mut by_file = conn
        .prepare(
            "SELECT id, file_slug FROM notes WHERE deleted_at IS NULL AND file_slug IS NOT NULL",
        )
        .context("Failed to prepare note file name query")?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .context("Failed to query note file names")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note file names")?
        .into_iter()
        .filter(|(_, file)| {
            metadata::slugify_note_ref(file) == slug
                || metadata::slugify_note_ref(file_names::split_folder(file).1) == slug
        })
        .filter_map(|(id, _)| NoteId::parse(&id));
    if let (Some(id), None) = (by_file.next(), by_file.next()) {
        return Ok(NoteRefResolution::Unique(id));
    }

    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT a.note_id
            FROM note_aliases a
            JOIN notes n ON n.id = a.note_id
            WHERE a.slug = ?1 AND n.deleted_at IS NULL
            "#,
        )
        .context("Failed to prepare note ref resolution query")?;
    let matches = stmt
        .query_map(params![slug], |row| row.get::<_, String>(0))
        .context("Failed to query note aliases for ref resolution")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note ref matches")?;

    match matches.len() {
        0 => Ok(NoteRefResolution::NotFound),
        1 => Ok(NoteId::parse(&matches[0])
            .map(NoteRefResolution::Unique)
            .unwrap_or(NoteRefResolution::NotFound)),
        _ => Ok(NoteRefResolution::Ambiguous),
    }
}

pub fn get_note(id: NoteId) -> Result<Option<Note>> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

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
    let conn = db.lock().map_err(db_lock_err)?;

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
    let conn = db.lock().map_err(db_lock_err)?;

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

/// Words that phrase a request ("find my note about the …") rather than say what a note holds.
/// They're dropped only when every typed word together finds nothing.
const NOTE_QUERY_FILLER: &[&str] = &[
    "a", "about", "an", "and", "any", "called", "find", "for", "from", "get", "i", "in", "is",
    "look", "me", "my", "named", "note", "notes", "of", "on", "open", "our", "please", "search",
    "show", "that", "the", "titled", "to", "where", "which", "with",
];

/// The words of a query as the search index splits them: runs of letters and digits, lowercased.
fn note_query_words(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Every word must appear somewhere in the note; the last one may still be half typed.
fn every_word_fts_query(words: &[String]) -> String {
    let last = words.len().saturating_sub(1);
    words
        .iter()
        .enumerate()
        .map(|(at, word)| {
            if at == last {
                format!("\"{word}\"*")
            } else {
                format!("\"{word}\"")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The full-text matches tried in order: the exact phrase, then every word in any order (the
/// last as a prefix), then the same without request phrasing such as "find my note about".
fn note_fts_queries(query: &str) -> Vec<String> {
    let words = note_query_words(query);
    if words.is_empty() {
        return Vec::new();
    }
    let mut queries = vec![sanitize_fts_query(query.trim())];
    queries.push(every_word_fts_query(&words));
    let content: Vec<String> = words
        .iter()
        .filter(|word| !NOTE_QUERY_FILLER.contains(&word.as_str()))
        .cloned()
        .collect();
    if !content.is_empty() && content.len() < words.len() {
        queries.push(every_word_fts_query(&content));
    }
    queries
}

/// Where `needle`'s letters appear in order in `title`, how many letters that run spans.
fn title_subsequence_span(needle: &[char], title: &str) -> Option<usize> {
    let letters: Vec<char> = title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    (0..letters.len())
        .filter(|start| letters[*start] == needle[0])
        .filter_map(|start| {
            let mut wanted = needle.iter();
            let mut next = wanted.next();
            for (at, letter) in letters[start..].iter().enumerate() {
                if Some(letter) == next {
                    next = wanted.next();
                    if next.is_none() {
                        return Some(at + 1);
                    }
                }
            }
            None
        })
        .min()
}

/// Notes whose title holds the typed letters in order, closest runs first, so a slip like
/// "lisbn" still finds "Lisbon trip". Only consulted when nothing else matched.
fn note_ids_by_title_letters(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>> {
    let needle: Vec<char> = query
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if needle.len() < 3 {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare("SELECT id, title FROM notes WHERE deleted_at IS NULL")
        .context("Failed to prepare note title scan")?;
    let mut scored: Vec<(usize, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .context("Failed to scan note titles")?
        .filter_map(|row| row.ok())
        .filter_map(|(id, title)| {
            title_subsequence_span(&needle, &title)
                .filter(|span| *span <= needle.len() * 2)
                .map(|span| (span, id))
        })
        .collect();
    scored.sort();
    Ok(scored.into_iter().take(limit).map(|(_, id)| id).collect())
}

/// Notes whose title contains the query as typed, for queries the word index can't express.
fn note_ids_by_title_text(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>> {
    let trimmed = query.trim().to_lowercase();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let escaped = trimmed
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id FROM notes
            WHERE deleted_at IS NULL AND lower(title) LIKE ?1 ESCAPE '\'
            ORDER BY
                CASE WHEN lower(title) = ?2 THEN 0 WHEN lower(title) LIKE ?3 ESCAPE '\' THEN 1 ELSE 2 END,
                is_pinned DESC,
                updated_at DESC
            LIMIT ?4
            "#,
        )
        .context("Failed to prepare note title text query")?;
    let ids = stmt
        .query_map(
            params![
                format!("%{escaped}%"),
                trimmed,
                format!("{escaped}%"),
                limit as i64
            ],
            |row| row.get::<_, String>(0),
        )
        .context("Failed to run note title text query")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note title text matches")?;
    Ok(ids)
}

/// Ids of the notes that match `query`, best first, at most `limit`.
///
/// `#tag`, `tag:`, `alias:` and `link:` read note metadata. Otherwise exact-phrase hits lead;
/// then notes holding every typed word in any order (the last word may be half typed, and the
/// index stems words so "booking" finds "booked"); then, if that found nothing, the words left
/// after request phrasing like "find my note about". Within each, titles that start with the
/// query come first. When the words find nothing at all, titles containing the query as typed
/// and finally titles holding its letters in order are tried.
fn ranked_note_ids(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>> {
    if let Some(notes) = search_notes_metadata_only(conn, query)? {
        return Ok(notes
            .into_iter()
            .take(limit)
            .map(|note| note.id.as_str())
            .collect());
    }
    let title_prefix = format!(
        "{}%",
        query
            .trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let title_exact = query.trim().to_lowercase();
    let mut ids: Vec<String> = Vec::new();
    {
        // The word index answers first and the notes table is read only for its hits; joining
        // the other way round re-runs the match once per note.
        let mut stmt = conn
            .prepare(
                r#"
                WITH hits AS MATERIALIZED (
                    SELECT rowid AS hit, bm25(notes_fts, 8.0, 1.0) AS score
                    FROM notes_fts WHERE notes_fts MATCH ?1
                )
                SELECT n.id
                FROM hits INNER JOIN notes n ON n.rowid = hits.hit
                WHERE n.deleted_at IS NULL
                ORDER BY lower(n.title) = ?4 DESC, lower(n.title) LIKE ?3 ESCAPE '\' DESC,
                    n.is_pinned DESC, hits.score, n.updated_at DESC
                LIMIT ?2
                "#,
            )
            .context("Failed to prepare notes word search")?;
        for (tier, fts_query) in note_fts_queries(query).iter().enumerate() {
            let found = stmt
                .query_map(
                    params![fts_query, limit as i64, title_prefix, title_exact],
                    |row| row.get::<_, String>(0),
                )
                .context("Failed to run notes word search")?
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to collect notes word search")?;
            for id in found {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            if tier > 0 && !ids.is_empty() {
                break;
            }
        }
    }
    ids.truncate(limit);
    if ids.is_empty() {
        ids = note_ids_by_title_text(conn, query, limit)?;
    }
    if ids.is_empty() {
        ids = note_ids_by_title_letters(conn, query, limit)?;
    }
    Ok(ids)
}

/// The ids of the notes `query` finds, best first, at most `limit`, without loading the
/// notes themselves: the same ranking as [`search_notes`], for lists that already hold
/// their rows.
pub(crate) fn search_note_ids(query: &str, limit: usize) -> Result<Vec<NoteId>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    Ok(ranked_note_ids(&conn, query, limit)?
        .iter()
        .filter_map(|id| NoteId::parse(id))
        .collect())
}

/// Loads rows for `ids` with `select` (whose `?` list is filled in) and keeps `ids`' order.
fn rows_for_note_ids<T>(
    conn: &Connection,
    ids: &[String],
    select: &str,
    map: impl FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    id_of: impl Fn(&T) -> NoteId,
) -> Result<Vec<T>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = select.replace("{ids}", &placeholders);
    let mut stmt = conn
        .prepare(&sql)
        .context("Failed to prepare matched notes query")?;
    let mut rows = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), map)
        .context("Failed to load matched notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect matched notes")?;
    let rank: HashMap<String, usize> = ids
        .iter()
        .enumerate()
        .map(|(at, id)| (id.clone(), at))
        .collect();
    rows.sort_by_key(|row| {
        rank.get(&id_of(row).as_str())
            .copied()
            .unwrap_or(usize::MAX)
    });
    Ok(rows)
}

fn log_notes_search_completed(query: &str, count: usize, method: &'static str) {
    let safe_query = crate::logging::log_private_user_value(query);
    debug!(
        query_bytes = safe_query.raw_bytes,
        query_sha256 = %safe_query.sha256,
        count,
        method,
        "Note search completed"
    );
}

fn log_notes_search_fts_fallback(query: &str, error: &impl std::fmt::Display) {
    let safe_query = crate::logging::log_private_user_value(query);
    let safe_error = crate::logging::log_private_user_value(&error.to_string());
    debug!(
        query_bytes = safe_query.raw_bytes,
        query_sha256 = %safe_query.sha256,
        error_bytes = safe_error.raw_bytes,
        error_sha256 = %safe_error.sha256,
        method = "like_fallback",
        "FTS search failed, using LIKE fallback"
    );
}

/// Search notes using full-text search
///
/// Matching follows `ranked_note_ids`; a LIKE scan over titles and bodies stands in if the
/// index itself fails.
pub fn search_notes(query: &str) -> Result<Vec<Note>> {
    if query.trim().is_empty() {
        return get_all_notes();
    }

    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    if let Some(metadata_notes) = search_notes_metadata_only(&conn, query)? {
        log_notes_search_completed(query, metadata_notes.len(), "metadata_only");
        return Ok(metadata_notes);
    }

    let fts_result = ranked_note_ids(&conn, query, 200).and_then(|ids| {
        rows_for_note_ids(
            &conn,
            &ids,
            r#"
            SELECT id, title, content, created_at, updated_at,
                   deleted_at, is_pinned, sort_order
            FROM notes WHERE id IN ({ids})
            "#,
            row_to_note,
            |note| note.id,
        )
    });

    match fts_result {
        Ok(notes) => {
            log_notes_search_completed(query, notes.len(), "fts");
            Ok(notes)
        }
        Err(e) => {
            // FTS failed (possibly due to special characters), fall back to LIKE search
            log_notes_search_fts_fallback(query, &e);

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

            log_notes_search_completed(query, notes.len(), "like_fallback");
            Ok(notes)
        }
    }
}

fn search_notes_metadata_only(conn: &Connection, query: &str) -> Result<Option<Vec<Note>>> {
    let trimmed = query.trim();
    if let Some(tag) = trimmed
        .strip_prefix("tag:")
        .or_else(|| trimmed.strip_prefix('#'))
    {
        return search_notes_by_metadata(conn, "tag", tag).map(Some);
    }
    if let Some(alias) = trimmed.strip_prefix("alias:") {
        return search_notes_by_metadata(conn, "alias", alias).map(Some);
    }
    if let Some(link) = trimmed.strip_prefix("link:") {
        return search_notes_by_metadata(conn, "link", link).map(Some);
    }
    Ok(None)
}

fn search_notes_by_metadata(conn: &Connection, mode: &str, query: &str) -> Result<Vec<Note>> {
    let normalized = match mode {
        "tag" => metadata::normalize_tag(query),
        "alias" | "link" => {
            let slug = metadata::slugify_note_ref(query);
            (!slug.is_empty()).then_some(slug)
        }
        _ => metadata::normalize_tag(query).or_else(|| {
            let slug = metadata::slugify_note_ref(query);
            (!slug.is_empty()).then_some(slug)
        }),
    };
    let Some(normalized) = normalized else {
        return Ok(Vec::new());
    };
    let pattern = format!("{}%", normalized);

    let condition = match mode {
        "tag" => "t.normalized_tag LIKE ?1",
        "alias" => "a.slug LIKE ?1",
        "link" => "l.target_slug LIKE ?1",
        _ => "t.normalized_tag LIKE ?1 OR a.slug LIKE ?1 OR l.target_slug LIKE ?1",
    };
    let sql = format!(
        r#"
        SELECT DISTINCT n.id, n.title, n.content, n.created_at, n.updated_at,
               n.deleted_at, n.is_pinned, n.sort_order
        FROM notes n
        LEFT JOIN note_tags t ON t.note_id = n.id
        LEFT JOIN note_aliases a ON a.note_id = n.id
        LEFT JOIN note_links l ON l.source_note_id = n.id
        WHERE n.deleted_at IS NULL AND ({condition})
        ORDER BY n.is_pinned DESC, n.updated_at DESC
        LIMIT 200
        "#
    );

    let mut stmt = conn
        .prepare(&sql)
        .context("Failed to prepare notes metadata search query")?;
    let notes = stmt
        .query_map(params![pattern], row_to_note)
        .context("Failed to execute notes metadata search")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect notes metadata search results")?;
    Ok(notes)
}

pub(crate) fn get_note_tags(note_id: NoteId) -> Result<Vec<String>> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT tag
            FROM note_tags
            WHERE note_id = ?1
            ORDER BY normalized_tag ASC
            "#,
        )
        .context("Failed to prepare note tags query")?;
    let tags = stmt
        .query_map(params![note_id.as_str()], |row| row.get::<_, String>(0))
        .context("Failed to query note tags")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note tags")?;
    Ok(tags)
}

pub(crate) fn get_note_aliases(note_id: NoteId) -> Result<Vec<String>> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT alias
            FROM note_aliases
            WHERE note_id = ?1
            ORDER BY slug ASC
            "#,
        )
        .context("Failed to prepare note aliases query")?;
    let aliases = stmt
        .query_map(params![note_id.as_str()], |row| row.get::<_, String>(0))
        .context("Failed to query note aliases")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note aliases")?;
    Ok(aliases)
}

pub(crate) fn get_note_outbound_link_count(note_id: NoteId) -> Result<usize> {
    count_note_links(
        "SELECT COUNT(*) FROM note_links WHERE source_note_id = ?1",
        note_id,
    )
}

pub(crate) fn get_note_backlink_count(note_id: NoteId) -> Result<usize> {
    count_note_links(
        r#"
        SELECT COUNT(DISTINCT l.source_note_id)
        FROM note_links l
        JOIN notes n ON n.id = l.source_note_id
        WHERE l.target_note_id = ?1
          AND n.deleted_at IS NULL
        "#,
        note_id,
    )
}

pub(crate) fn get_note_backlinks(note_id: NoteId) -> Result<Vec<NoteBacklinkSummary>> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT n.id, n.title, n.updated_at
            FROM note_links l
            JOIN notes n ON n.id = l.source_note_id
            WHERE l.target_note_id = ?1 AND n.deleted_at IS NULL
            ORDER BY n.updated_at DESC
            "#,
        )
        .context("Failed to prepare note backlinks query")?;
    let backlinks = stmt
        .query_map(params![note_id.as_str()], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let updated_at_str: String = row.get(2)?;
            let id = NoteId::parse(&id).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    format!("Invalid backlink source note UUID: {id}").into(),
                )
            })?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(NoteBacklinkSummary {
                id,
                title,
                updated_at,
            })
        })
        .context("Failed to query note backlinks")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect note backlinks")?;
    Ok(backlinks)
}

fn count_note_links(sql: &str, note_id: NoteId) -> Result<usize> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let count: i64 = conn
        .query_row(sql, params![note_id.as_str()], |row| row.get(0))
        .context("Failed to count note links")?;
    Ok(count.max(0) as usize)
}

/// Search notes for root launcher rows without returning note body content.
pub(crate) fn search_root_notes_meta(
    query: &str,
    options: RootNotesSectionOptions,
) -> Vec<RootNoteSearchHit> {
    if !root_notes_query_is_eligible(query, options) {
        return Vec::new();
    }

    match search_root_notes_meta_result(query.trim(), options) {
        Ok(hits) => hits,
        Err(error) => {
            log_root_notes_search_failure(query, &error);
            Vec::new()
        }
    }
}

fn log_root_notes_search_failure(query: &str, error: &anyhow::Error) {
    let safe_query = crate::logging::log_private_user_value(query);
    let safe_error = crate::logging::log_private_user_value(&error.to_string());
    tracing::warn!(
        query_bytes = safe_query.raw_bytes,
        query_sha256 = %safe_query.sha256,
        error_bytes = safe_error.raw_bytes,
        error_sha256 = %safe_error.sha256,
        "root_notes_search_failed"
    );
}

pub(crate) fn search_root_notes_meta_direct(
    query: &str,
    options: RootNotesSectionOptions,
) -> Vec<RootNoteSearchHit> {
    search_root_notes_meta(query, options)
}

/// Cache-only root notes lookup for the launcher foreground search path.
///
/// A cold query returns no hits without opening SQLite or starting a worker.
/// The launcher owns refresh scheduling and only publishes an exact live-query
/// snapshot after validating its source, generation, and cache epoch.
pub(crate) fn search_root_notes_meta_cached(
    query: &str,
    options: RootNotesSectionOptions,
) -> Vec<RootNoteSearchHit> {
    if !root_notes_query_is_eligible(query, options) {
        return Vec::new();
    }

    root_notes_search_cache()
        .lock()
        .ok()
        .and_then(|cache| {
            cache
                .hits_by_query
                .get(&root_notes_search_cache_key(query, options))
                .map(|cached| cached.hits.clone())
        })
        .unwrap_or_default()
}

pub(crate) fn root_notes_search_cache_is_fresh(
    query: &str,
    options: RootNotesSectionOptions,
) -> bool {
    root_notes_query_is_eligible(query, options)
        && root_notes_search_cache().lock().is_ok_and(|cache| {
            cache
                .hits_by_query
                .get(&root_notes_search_cache_key(query, options))
                .is_some_and(|cached| {
                    cached.generation == ROOT_NOTES_SEARCH_CACHE_GENERATION.load(Ordering::Relaxed)
                })
        })
}

fn fresh_root_notes_search_cache_status(
    cache: &RootNotesSearchCache,
    query: &str,
    options: RootNotesSectionOptions,
    generation: u64,
) -> Option<(u64, usize)> {
    if !root_notes_query_is_eligible(query, options)
        || cache.refresh_lifecycle.in_flight.is_some()
        || !cache.in_flight.is_empty()
    {
        return None;
    }
    let cached = cache
        .hits_by_query
        .get(&root_notes_search_cache_key(query, options))?;
    (cached.generation == generation).then_some((cached.generation, cached.hits.len()))
}

/// Current exact-query cache epoch and row count; no source work is admitted.
pub(crate) fn root_notes_search_fresh_cache_status(
    query: &str,
    options: RootNotesSectionOptions,
) -> Option<(u64, usize)> {
    let cache = root_notes_search_cache().try_lock().ok()?;
    fresh_root_notes_search_cache_status(
        &cache,
        query,
        options,
        ROOT_NOTES_SEARCH_CACHE_GENERATION.load(Ordering::Relaxed),
    )
}

fn try_begin_root_notes_search_refresh_in_cache(
    cache: &mut RootNotesSearchCache,
    query: &str,
    options: RootNotesSectionOptions,
    cache_generation: u64,
) -> Option<RootNotesSearchRefresh> {
    if !root_notes_query_is_eligible(query, options) {
        return None;
    }

    let search = root_notes_search_cache_key(query, options);
    let owner = cache.refresh_lifecycle.begin(
        sk_protocol::command_contract::CommandSource::Note,
        cache
            .hits_by_query
            .get(&search)
            .is_some_and(|cached| cached.generation == cache_generation),
    )?;
    let flight = RootNotesSearchFlightKey {
        generation: cache_generation,
        search,
    };
    if !cache.in_flight.insert(flight.clone()) {
        cache.refresh_lifecycle.finish(owner);
        return None;
    }

    tracing::debug!(
        target: "script_kit::search",
        source = "root-notes-search-cache",
        generation = owner.generation,
        cache_generation,
        "Started owned Notes snapshot refresh"
    );
    Some(RootNotesSearchRefresh {
        owner,
        flight,
        options,
    })
}

pub(crate) fn try_begin_root_notes_search_refresh(
    query: &str,
    options: RootNotesSectionOptions,
) -> Option<RootNotesSearchRefresh> {
    let cache_generation = ROOT_NOTES_SEARCH_CACHE_GENERATION.load(Ordering::Relaxed);
    let mut cache = root_notes_search_cache().lock().ok()?;
    try_begin_root_notes_search_refresh_in_cache(&mut cache, query, options, cache_generation)
}

pub(crate) fn read_root_notes_search_snapshot(
    refresh: &RootNotesSearchRefresh,
) -> RootNotesSearchSnapshot {
    assert!(
        !crate::runtime_policy::is_owned_evaluation(),
        "owned_source_snapshot_required"
    );
    RootNotesSearchSnapshot {
        flight: refresh.flight.clone(),
        hits: search_root_notes_meta_result(&refresh.flight.search.query, refresh.options),
    }
}

pub(crate) fn owned_root_notes_search_snapshot(
    refresh: &RootNotesSearchRefresh,
    result: Result<Vec<RootNoteSearchHit>>,
) -> Result<RootNotesSearchSnapshot> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    Ok(RootNotesSearchSnapshot {
        flight: refresh.flight.clone(),
        hits: result,
    })
}

pub(crate) fn reset_owned_root_notes_search() -> Result<()> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    let mut cache = root_notes_search_cache().lock().map_err(db_lock_err)?;
    let generation = cache
        .refresh_lifecycle
        .next_generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("notes_refresh_generation_exhausted"))?;
    ROOT_NOTES_SEARCH_CACHE_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| anyhow::anyhow!("notes_cache_generation_exhausted"))?;
    cache.refresh_lifecycle.next_generation = generation;
    cache.refresh_lifecycle.in_flight = None;
    cache.in_flight.clear();
    cache.hits_by_query.clear();
    Ok(())
}

pub(crate) fn invalidate_owned_root_notes_search_freshness() -> Result<()> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    let mut cache = root_notes_search_cache().lock().map_err(db_lock_err)?;
    let generation = cache
        .refresh_lifecycle
        .next_generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("notes_refresh_generation_exhausted"))?;
    ROOT_NOTES_SEARCH_CACHE_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| anyhow::anyhow!("notes_cache_generation_exhausted"))?;
    cache.refresh_lifecycle.next_generation = generation;
    cache.refresh_lifecycle.in_flight = None;
    cache.in_flight.clear();
    Ok(())
}

fn finish_root_notes_search_refresh_in_cache(
    cache: &mut RootNotesSearchCache,
    refresh: RootNotesSearchRefresh,
    snapshot: RootNotesSearchSnapshot,
    current_cache_generation: u64,
) -> bool {
    if refresh.owner.source != sk_protocol::command_contract::CommandSource::Note
        || snapshot.flight != refresh.flight
        || cache.refresh_lifecycle.in_flight != Some(refresh.owner)
        || !cache.in_flight.contains(&refresh.flight)
    {
        return false;
    }
    cache.in_flight.remove(&refresh.flight);
    if !cache.refresh_lifecycle.finish(refresh.owner)
        || refresh.flight.generation != current_cache_generation
    {
        return false;
    }
    let Ok(hits) = snapshot.hits else {
        return false;
    };
    cache.hits_by_query.insert(
        refresh.flight.search,
        RootNotesCachedHits {
            generation: refresh.flight.generation,
            hits,
        },
    );
    true
}

pub(crate) fn finish_root_notes_search_refresh(
    refresh: RootNotesSearchRefresh,
    snapshot: RootNotesSearchSnapshot,
) -> bool {
    root_notes_search_cache().lock().is_ok_and(|mut cache| {
        finish_root_notes_search_refresh_in_cache(
            &mut cache,
            refresh,
            snapshot,
            ROOT_NOTES_SEARCH_CACHE_GENERATION.load(Ordering::Relaxed),
        )
    })
}

fn discard_root_notes_search_refresh_in_cache(
    cache: &mut RootNotesSearchCache,
    refresh: RootNotesSearchRefresh,
) -> bool {
    if !cache.in_flight.contains(&refresh.flight) || !cache.refresh_lifecycle.finish(refresh.owner)
    {
        return false;
    }
    cache.in_flight.remove(&refresh.flight);
    true
}

pub(crate) fn discard_root_notes_search_refresh(refresh: RootNotesSearchRefresh) -> bool {
    root_notes_search_cache()
        .lock()
        .is_ok_and(|mut cache| discard_root_notes_search_refresh_in_cache(&mut cache, refresh))
}

fn root_notes_search_result_limit(options: RootNotesSectionOptions) -> i64 {
    options.max_results.clamp(1, MAX_ROOT_NOTES_SEARCH_RESULTS) as i64
}

fn search_root_notes_meta_result(
    query: &str,
    options: RootNotesSectionOptions,
) -> Result<Vec<RootNoteSearchHit>> {
    ensure_notes_db_open()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    let limit = root_notes_search_result_limit(options);
    let hits = if query.trim().is_empty() {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, title, updated_at, is_pinned, length(content)
                FROM notes
                WHERE deleted_at IS NULL
                ORDER BY is_pinned DESC, updated_at DESC
                LIMIT ?1
                "#,
            )
            .context("Failed to prepare root notes recent query")?;

        let rows = stmt
            .query_map(params![limit], row_to_root_note_hit)
            .context("Failed to execute root notes recent query")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect root notes recent results")?;
        rows
    } else if options.search_content {
        let ids = ranked_note_ids(&conn, query, limit as usize)?;
        rows_for_note_ids(
            &conn,
            &ids,
            "SELECT id, title, updated_at, is_pinned, length(content) FROM notes WHERE id IN ({ids})",
            row_to_root_note_hit,
            |hit| hit.id,
        )?
    } else {
        search_root_notes_title_like(&conn, query, limit)?
    };

    Ok(hits
        .into_iter()
        .enumerate()
        .map(|(rank, mut hit)| {
            hit.score = i32::MAX.saturating_sub(rank as i32);
            hit
        })
        .collect())
}

/// Root search with note bodies switched off: titles containing the query, else titles
/// holding its letters in order.
fn search_root_notes_title_like(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<RootNoteSearchHit>> {
    let mut ids = note_ids_by_title_text(conn, query, limit as usize)?;
    if ids.is_empty() {
        ids = note_ids_by_title_letters(conn, query, limit as usize)?;
    }
    rows_for_note_ids(
        conn,
        &ids,
        "SELECT id, title, updated_at, is_pinned, length(content) FROM notes WHERE id IN ({ids})",
        row_to_root_note_hit,
        |hit| hit.id,
    )
}

/// Notes last changed on a local day that `on_day` accepts, most recent first, at most 60.
/// Reads only ids and dates to decide, so it stays quick with long documents.
pub(crate) fn notes_changed_on(on_day: impl Fn(chrono::NaiveDate) -> bool) -> Result<Vec<Note>> {
    ensure_notes_db_open()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let ids: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, updated_at FROM notes WHERE deleted_at IS NULL ORDER BY updated_at DESC",
            )
            .context("Failed to prepare notes-by-day query")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .context("Failed to read note change dates")?;
        rows.filter_map(|row| row.ok())
            .filter(|(_, updated_at)| {
                DateTime::parse_from_rfc3339(updated_at)
                    .is_ok_and(|at| on_day(at.with_timezone(&chrono::Local).date_naive()))
            })
            .map(|(id, _)| id)
            .take(60)
            .collect()
    };
    rows_for_note_ids(
        &conn,
        &ids,
        r#"
        SELECT id, title, content, created_at, updated_at,
               deleted_at, is_pinned, sort_order
        FROM notes WHERE id IN ({ids})
        "#,
        row_to_note,
        |note| note.id,
    )
}

/// Pinned notes, most recently changed first, for the launcher with nothing typed.
pub(crate) fn pinned_root_notes(limit: usize) -> Result<Vec<RootNoteSearchHit>> {
    ensure_notes_db_open()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, updated_at, is_pinned, length(content)
            FROM notes
            WHERE deleted_at IS NULL AND is_pinned = 1
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )
        .context("Failed to prepare pinned notes query")?;
    let hits = stmt
        .query_map(params![limit as i64], row_to_root_note_hit)
        .context("Failed to execute pinned notes query")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect pinned notes")?;
    Ok(hits)
}

/// Permanently delete a note
pub fn delete_note_permanently(id: NoteId) -> Result<()> {
    delete_note_permanently_checked(id, false)
}

/// Delete a confirmed Trash target only while its current stored state is deleted.
pub fn delete_trashed_note_permanently(id: NoteId) -> Result<()> {
    delete_note_permanently_checked(id, true)
}

fn delete_note_permanently_checked(id: NoteId, require_deleted: bool) -> Result<()> {
    let substrate = notes_substrate()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    if require_deleted {
        let is_deleted = conn
            .query_row(
                "SELECT deleted_at IS NOT NULL FROM notes WHERE id = ?1",
                params![id.as_str()],
                |row| row.get::<_, bool>(0),
            )
            .optional()?;
        anyhow::ensure!(is_deleted == Some(true), "confirmed_trash_target_changed");
    }

    let slug = lookup_note_slug(&conn, id)?;
    if let Some(slug) = slug {
        let active_path = substrate.paths().note_file(&slug);
        if active_path.exists() {
            fs::remove_file(&active_path)
                .with_context(|| format!("removing active note file {}", active_path.display()))?;
        }
        delete_trashed_note_file(&substrate, &slug)?;
    }

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
        .context("Failed to delete note")?;

    forget_note_hash(id);
    info!(note_id = %id, "Note permanently deleted");
    invalidate_root_notes_search_cache();
    Ok(())
}

/// Delete a frozen batch of Trash identities, preserving failed targets for retry.
pub fn delete_all_deleted_notes() -> Result<()> {
    let ids: Vec<NoteId> = get_deleted_notes()?
        .into_iter()
        .map(|note| note.id)
        .collect();
    let mut failed = 0;
    for id in &ids {
        if let Err(error) = delete_trashed_note_permanently(*id) {
            let safe = crate::logging::log_private_user_value(&error.to_string());
            tracing::error!(note_id = %id, diagnostic_fingerprint = %safe.sha256, "Trash target deletion failed");
            failed += 1;
        }
    }
    anyhow::ensure!(failed == 0, "trash_delete_incomplete:{failed}");
    Ok(())
}

/// Prune notes deleted more than `days` ago
pub fn prune_old_deleted_notes(days: u32) -> Result<usize> {
    let substrate = notes_substrate()?;
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    let slugs: Vec<String> = conn
        .prepare(
            "SELECT file_slug FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1 AND file_slug IS NOT NULL",
        )?
        .query_map(params![cutoff.to_rfc3339()], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let count = conn
        .execute(
            "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted notes")?;

    for slug in slugs {
        delete_trashed_note_file(&substrate, &slug)?;
    }

    if count > 0 {
        info!(count, days, "Pruned old deleted notes");
        invalidate_root_notes_search_cache();
    }

    Ok(count)
}

// ── Cart item persistence ───────────────────────────────────────────

/// Save a cart item (insert or update).
pub fn save_note_cart_item(item: &super::model::NoteCartItem) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

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
    let conn = db.lock().map_err(db_lock_err)?;

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

/// List cart items for a note, dropping duplicate payloads while preserving order.
pub fn list_note_cart_items_deduped(note_id: NoteId) -> Result<Vec<super::model::NoteCartItem>> {
    let mut items = list_note_cart_items(note_id)?;
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| seen.insert(item.dedup_key()));
    Ok(items)
}

/// Delete a cart item by ID.
pub fn delete_note_cart_item(item_id: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(db_lock_err)?;

    conn.execute(
        "DELETE FROM note_cart_items WHERE id = ?1",
        params![item_id],
    )
    .context("Failed to delete cart item")?;

    info!(cart_item_id = %item_id, "Cart item deleted");
    Ok(())
}

/// Delete multiple cart items for a note in one note-scoped transaction.
pub fn delete_note_cart_items(note_id: NoteId, item_ids: &[String]) -> Result<usize> {
    if item_ids.is_empty() {
        return Ok(0);
    }
    if std::env::var("SCRIPT_KIT_TEST_STATUS").ok().as_deref() == Some("1")
        && std::env::var("SCRIPT_KIT_TEST_NOTES_CART_DELETE_FAIL")
            .ok()
            .as_deref()
            == Some("1")
    {
        anyhow::bail!("test fixture refused note cart consumption");
    }

    let db = get_db()?;
    let mut conn = db.lock().map_err(db_lock_err)?;

    let tx = conn
        .transaction()
        .context("Failed to start note cart item delete transaction")?;

    let mut deleted = 0usize;
    for item_id in item_ids {
        deleted += tx
            .execute(
                "DELETE FROM note_cart_items WHERE note_id = ?1 AND id = ?2",
                params![note_id.as_str(), item_id],
            )
            .context("Failed to delete note-scoped cart item")?;
    }

    tx.commit()
        .context("Failed to commit note cart item delete transaction")?;

    info!(
        note_id = %note_id,
        requested = item_ids.len(),
        deleted,
        "Note cart items deleted"
    );
    Ok(deleted)
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

fn row_to_root_note_hit(row: &rusqlite::Row) -> rusqlite::Result<RootNoteSearchHit> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let updated_at_str: String = row.get(2)?;
    let is_pinned: i32 = row.get(3)?;
    let char_count: i64 = row.get(4)?;

    let id = NoteId::parse(&id_str).unwrap_or_default();
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(RootNoteSearchHit {
        id,
        title,
        updated_at,
        is_pinned: is_pinned != 0,
        char_count: char_count.max(0) as usize,
        score: 0,
    })
}

/// Serialize tests that mutate the shared per-process notes DB.
///
/// Shared with `notes::menu_syntax_capture` tests, which hit the same DB.
/// Poison-tolerant so one failing test reports its own assertion instead of
/// cascading `PoisonError` panics into unrelated tests.
#[cfg(test)]
pub(crate) fn notes_db_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[test]
fn owned_note_snapshots_and_resets_require_runtime_authority() {
    let mut cache = RootNotesSearchCache::default();
    let refresh = try_begin_root_notes_search_refresh_in_cache(
        &mut cache,
        "fixture",
        RootNotesSectionOptions {
            enabled: true,
            ..Default::default()
        },
        1,
    )
    .expect("eligible local refresh");
    assert!(owned_root_notes_search_snapshot(&refresh, Ok(Vec::new())).is_err());
    assert!(reset_owned_root_notes_search().is_err());
    assert!(invalidate_owned_root_notes_search_freshness().is_err());
}

#[cfg(test)]
include!("storage_tests.rs");
