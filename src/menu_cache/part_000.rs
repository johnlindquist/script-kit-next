use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
/// A menu bar item with its hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MenuBarItem {
    pub title: String,
    pub enabled: bool,
    pub shortcut: Option<String>,
    pub children: Vec<MenuBarItem>,
    pub menu_path: Vec<String>, // e.g., ["File", "New Window"]
}
/// Global database connection for menu cache
static MENU_CACHE_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();
/// Get the path to the menu cache database
fn get_menu_cache_db_path() -> PathBuf {
    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit"));

    kit_dir.join("db").join("menu-cache.sqlite")
}
/// Initialize the menu cache database
///
/// This function is idempotent - it's safe to call multiple times.
/// If the database is already initialized, it returns Ok(()) immediately.
pub fn init_menu_cache_db() -> Result<()> {
    // Check if already initialized - this is the common case after first init
    if MENU_CACHE_DB.get().is_some() {
        debug!("Menu cache database already initialized, skipping");
        return Ok(());
    }

    let db_path = get_menu_cache_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create menu cache db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open menu cache database")?;

    // Create tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS menu_cache (
            bundle_id TEXT PRIMARY KEY,
            menu_json TEXT NOT NULL,
            last_scanned INTEGER NOT NULL,
            app_version TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_menu_cache_last_scanned ON menu_cache(last_scanned);
        "#,
    )
    .context("Failed to create menu cache table")?;

    info!(db_path = %db_path.display(), "Menu cache database initialized");

    // Use get_or_init pattern to handle race condition where another thread
    // might have initialized the DB between our check and set
    let _ = MENU_CACHE_DB.get_or_init(|| Arc::new(Mutex::new(conn)));

    Ok(())
}
/// Get a reference to the menu cache database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    MENU_CACHE_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Menu cache database not initialized"))
}
/// Get the current timestamp as Unix epoch seconds
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
/// Get cached menu items for an application by bundle_id
pub fn get_cached_menu(bundle_id: &str) -> Result<Option<Vec<MenuBarItem>>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let result: Option<String> = conn
        .query_row(
            "SELECT menu_json FROM menu_cache WHERE bundle_id = ?1",
            params![bundle_id],
            |row| row.get(0),
        )
        .optional()
        .context("Failed to query menu cache")?;

    match result {
        Some(json) => {
            let items: Vec<MenuBarItem> =
                serde_json::from_str(&json).context("Failed to deserialize menu items")?;
            debug!(bundle_id = %bundle_id, item_count = items.len(), "Retrieved cached menu");
            Ok(Some(items))
        }
        None => {
            debug!(bundle_id = %bundle_id, "No cached menu found");
            Ok(None)
        }
    }
}
/// Set (insert or update) cached menu items for an application
pub fn set_cached_menu(
    bundle_id: &str,
    items: &[MenuBarItem],
    app_version: Option<&str>,
) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let menu_json = serde_json::to_string(items).context("Failed to serialize menu items")?;
    let timestamp = current_timestamp();

    conn.execute(
        r#"
        INSERT INTO menu_cache (bundle_id, menu_json, last_scanned, app_version)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(bundle_id) DO UPDATE SET
            menu_json = excluded.menu_json,
            last_scanned = excluded.last_scanned,
            app_version = excluded.app_version
        "#,
        params![bundle_id, menu_json, timestamp, app_version],
    )
    .context("Failed to save menu cache")?;

    debug!(
        bundle_id = %bundle_id,
        item_count = items.len(),
        app_version = app_version.unwrap_or("none"),
        "Menu cache updated"
    );
    Ok(())
}
/// Check if the cache for a bundle_id is still valid (not expired)
pub fn is_cache_valid(bundle_id: &str, max_age_secs: u64) -> Result<bool> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let result: Option<i64> = conn
        .query_row(
            "SELECT last_scanned FROM menu_cache WHERE bundle_id = ?1",
            params![bundle_id],
            |row| row.get(0),
        )
        .optional()
        .context("Failed to query cache validity")?;

    match result {
        Some(last_scanned) => {
            let now = current_timestamp();
            let age = (now - last_scanned) as u64;
            let valid = age <= max_age_secs;
            debug!(
                bundle_id = %bundle_id,
                last_scanned,
                age_secs = age,
                max_age_secs,
                valid,
                "Cache validity check"
            );
            Ok(valid)
        }
        None => {
            debug!(bundle_id = %bundle_id, "No cache entry found, treating as invalid");
            Ok(false)
        }
    }
}
/// Delete cached menu for an application (useful when app is uninstalled)
pub fn delete_cached_menu(bundle_id: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        "DELETE FROM menu_cache WHERE bundle_id = ?1",
        params![bundle_id],
    )
    .context("Failed to delete menu cache")?;

    info!(bundle_id = %bundle_id, "Menu cache entry deleted");
    Ok(())
}
/// Prune cache entries older than the specified age in seconds
pub fn prune_old_cache_entries(max_age_secs: u64) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = current_timestamp() - max_age_secs as i64;

    let count = conn
        .execute(
            "DELETE FROM menu_cache WHERE last_scanned < ?1",
            params![cutoff],
        )
        .context("Failed to prune old cache entries")?;

    if count > 0 {
        info!(count, max_age_secs, "Pruned old menu cache entries");
    }

    Ok(count)
}
/// Get the total number of cached menus
pub fn get_cache_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM menu_cache", [], |row| row.get(0))
        .context("Failed to count cache entries")?;

    Ok(count as usize)
}
