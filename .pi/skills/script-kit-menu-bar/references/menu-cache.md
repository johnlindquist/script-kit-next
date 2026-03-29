# Menu Cache Reference

Detailed documentation for `src/menu_cache.rs` - SQLite persistence for menu data.

## Table of Contents

1. [Database Schema](#database-schema)
2. [Global State](#global-state)
3. [Public API](#public-api)
4. [Cache Strategy](#cache-strategy)
5. [Thread Safety](#thread-safety)

## Database Schema

### Location

```
~/.scriptkit/db/menu-cache.sqlite
```

Created automatically by `init_menu_cache_db()`.

### Table: menu_cache

```sql
CREATE TABLE IF NOT EXISTS menu_cache (
    bundle_id TEXT PRIMARY KEY,     -- App identifier (e.g., "com.apple.Safari")
    menu_json TEXT NOT NULL,        -- JSON-serialized Vec<MenuBarItem>
    last_scanned INTEGER NOT NULL,  -- Unix timestamp (seconds)
    app_version TEXT                -- Optional version for cache invalidation
);

CREATE INDEX IF NOT EXISTS idx_menu_cache_last_scanned ON menu_cache(last_scanned);
```

### MenuBarItem (Serializable)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MenuBarItem {
    pub title: String,
    pub enabled: bool,
    pub shortcut: Option<String>,    // Display string, not KeyboardShortcut
    pub children: Vec<MenuBarItem>,
    pub menu_path: Vec<String>,      // Full path: ["File", "New Window"]
}
```

> **Note**: This `MenuBarItem` differs from `menu_bar.rs` version:
> - `shortcut` is `Option<String>` (display string) instead of `Option<KeyboardShortcut>`
> - `menu_path` instead of `ax_element_path`
> - Optimized for serialization and SDK transmission

## Global State

### Database Connection

```rust
static MENU_CACHE_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();
```

- `OnceLock`: Initialize once, read-only after
- `Arc<Mutex<Connection>>`: Thread-safe shared ownership
- Pattern matches `notes/storage.rs` for consistency

### Initialization Guard

```rust
pub fn init_menu_cache_db() -> Result<()> {
    // Fast path: already initialized
    if MENU_CACHE_DB.get().is_some() {
        return Ok(());
    }
    
    // Slow path: create DB
    // ...
    
    // Race-safe set
    let _ = MENU_CACHE_DB.get_or_init(|| Arc::new(Mutex::new(conn)));
    Ok(())
}
```

Safe to call multiple times - idempotent.

## Public API

### init_menu_cache_db

```rust
pub fn init_menu_cache_db() -> Result<()>
```

Initialize database. Call once at app startup.

**Creates**:
- Directory: `~/.scriptkit/db/`
- Database: `menu-cache.sqlite`
- Table: `menu_cache`
- Index: `idx_menu_cache_last_scanned`

### get_cached_menu

```rust
pub fn get_cached_menu(bundle_id: &str) -> Result<Option<Vec<MenuBarItem>>>
```

Retrieve cached menu for an app.

**Returns**:
- `Ok(Some(items))`: Cache hit
- `Ok(None)`: Cache miss (no entry)
- `Err(...)`: Database error

### set_cached_menu

```rust
pub fn set_cached_menu(
    bundle_id: &str,
    items: &[MenuBarItem],
    app_version: Option<&str>
) -> Result<()>
```

Insert or update cache entry (upsert).

**Parameters**:
- `bundle_id`: App identifier
- `items`: Menu items to cache
- `app_version`: Optional version string for smarter invalidation

**SQL**:
```sql
INSERT INTO menu_cache (bundle_id, menu_json, last_scanned, app_version)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(bundle_id) DO UPDATE SET
    menu_json = excluded.menu_json,
    last_scanned = excluded.last_scanned,
    app_version = excluded.app_version
```

### is_cache_valid

```rust
pub fn is_cache_valid(bundle_id: &str, max_age_secs: u64) -> Result<bool>
```

Check if cache entry exists and is not stale.

**Parameters**:
- `bundle_id`: App identifier
- `max_age_secs`: Maximum cache age in seconds

**Returns**:
- `true`: Entry exists and `(now - last_scanned) <= max_age_secs`
- `false`: No entry or expired

### delete_cached_menu

```rust
pub fn delete_cached_menu(bundle_id: &str) -> Result<()>
```

Remove cache entry. Useful when:
- App is uninstalled
- User requests fresh scan
- Cache corruption suspected

### prune_old_cache_entries

```rust
pub fn prune_old_cache_entries(max_age_secs: u64) -> Result<usize>
```

Delete all entries older than `max_age_secs`.

**Returns**: Number of entries deleted.

**Use case**: Periodic cleanup job to prevent database bloat.

### get_cache_count

```rust
pub fn get_cache_count() -> Result<usize>
```

Get total number of cached apps. Useful for diagnostics.

## Cache Strategy

### Recommended Flow

```rust
const CACHE_MAX_AGE: u64 = 3600; // 1 hour

pub fn get_menu_with_cache(bundle_id: &str) -> Result<Vec<MenuBarItem>> {
    // 1. Check cache validity
    if is_cache_valid(bundle_id, CACHE_MAX_AGE)? {
        if let Some(items) = get_cached_menu(bundle_id)? {
            return Ok(items);
        }
    }
    
    // 2. Scan fresh
    let items = scan_menu_bar_for_app(bundle_id)?;
    
    // 3. Update cache
    let app_version = get_app_version(bundle_id);
    set_cached_menu(bundle_id, &items, app_version.as_deref())?;
    
    Ok(items)
}
```

### Cache Invalidation Triggers

| Trigger | Action |
|---------|--------|
| Time-based | `is_cache_valid()` with max_age |
| Version change | Compare `app_version` field |
| User request | `delete_cached_menu()` |
| App uninstall | `delete_cached_menu()` |
| Periodic cleanup | `prune_old_cache_entries()` |

### Version-Based Invalidation

```rust
// When scanning, detect app version
let current_version = get_app_version(bundle_id);

// When using cache, compare versions
if cached_version != current_version {
    // App updated, invalidate cache
    delete_cached_menu(bundle_id)?;
}
```

## Thread Safety

### Locking Pattern

```rust
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    MENU_CACHE_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
}

pub fn some_operation() -> Result<()> {
    let db = get_db()?;
    let conn = db.lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;
    
    // Use conn...
    Ok(())
}
```

### Mutex Scope

Keep lock scope minimal:
```rust
// Good: Lock only for DB operation
{
    let conn = db.lock()?;
    conn.execute(...)?;
} // Lock released here
do_other_work();

// Bad: Hold lock during unrelated work
let conn = db.lock()?;
conn.execute(...)?;
do_other_work();  // Still holding lock!
```

### Concurrent Access

SQLite with WAL mode would allow concurrent reads, but current implementation uses default journal mode. For high-concurrency needs, consider:

1. Connection pooling (r2d2)
2. WAL mode: `PRAGMA journal_mode=WAL;`
3. Separate read/write connections

Current implementation is sufficient for Script Kit's usage pattern (single app, occasional concurrent access).
