# Caching Strategies - Expert Bundle

## Overview

Script Kit uses multiple caching layers for performance: SQLite for persistent data, in-memory caches for hot data, and file system caching for scripts.

## SQLite Cache Layer

### Menu Cache (src/menu_cache.rs)

```rust
use rusqlite::{Connection, params, OptionalExtension};
use std::sync::{Arc, Mutex, OnceLock};

/// Global database connection
static MENU_CACHE_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Initialize the database (idempotent)
pub fn init_menu_cache_db() -> Result<()> {
    if MENU_CACHE_DB.get().is_some() {
        return Ok(()); // Already initialized
    }

    let conn = Connection::open(get_db_path())?;
    
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS menu_cache (
            bundle_id TEXT PRIMARY KEY,
            menu_json TEXT NOT NULL,
            last_scanned INTEGER NOT NULL,
            app_version TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_menu_cache_last_scanned 
            ON menu_cache(last_scanned);
    "#)?;

    let _ = MENU_CACHE_DB.get_or_init(|| Arc::new(Mutex::new(conn)));
    Ok(())
}
```

### Cache Operations

```rust
/// Get cached menu items
pub fn get_cached_menu(bundle_id: &str) -> Result<Option<Vec<MenuBarItem>>> {
    let db = get_db()?;
    let conn = db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

    let result: Option<String> = conn
        .query_row(
            "SELECT menu_json FROM menu_cache WHERE bundle_id = ?1",
            params![bundle_id],
            |row| row.get(0),
        )
        .optional()?;

    match result {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Set/update cached menu items
pub fn set_cached_menu(
    bundle_id: &str,
    items: &[MenuBarItem],
    app_version: Option<&str>,
) -> Result<()> {
    let db = get_db()?;
    let conn = db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

    let menu_json = serde_json::to_string(items)?;
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
    )?;

    Ok(())
}
```

### Cache Validity

```rust
/// Check if cache is still valid
pub fn is_cache_valid(bundle_id: &str, max_age_secs: u64) -> Result<bool> {
    let db = get_db()?;
    let conn = db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

    let result: Option<i64> = conn
        .query_row(
            "SELECT last_scanned FROM menu_cache WHERE bundle_id = ?1",
            params![bundle_id],
            |row| row.get(0),
        )
        .optional()?;

    match result {
        Some(last_scanned) => {
            let age = (current_timestamp() - last_scanned) as u64;
            Ok(age <= max_age_secs)
        }
        None => Ok(false),
    }
}

/// Prune old entries
pub fn prune_old_cache_entries(max_age_secs: u64) -> Result<usize> {
    let db = get_db()?;
    let conn = db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

    let cutoff = current_timestamp() - max_age_secs as i64;
    let count = conn.execute(
        "DELETE FROM menu_cache WHERE last_scanned < ?1",
        params![cutoff],
    )?;

    Ok(count)
}
```

## In-Memory Script Cache

### Scriptlet Cache (src/scriptlet_cache.rs)

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ScriptletCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

struct CacheEntry {
    scriptlet: Arc<Scriptlet>,
    cached_at: Instant,
}

impl ScriptletCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub fn get(&self, path: &str) -> Option<Arc<Scriptlet>> {
        let entries = self.entries.read().ok()?;
        
        if let Some(entry) = entries.get(path) {
            if entry.cached_at.elapsed() < self.ttl {
                return Some(Arc::clone(&entry.scriptlet));
            }
        }
        None
    }

    pub fn set(&self, path: &str, scriptlet: Scriptlet) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(path.to_string(), CacheEntry {
                scriptlet: Arc::new(scriptlet),
                cached_at: Instant::now(),
            });
        }
    }

    pub fn invalidate(&self, path: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(path);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Remove expired entries
    pub fn gc(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.retain(|_, entry| entry.cached_at.elapsed() < self.ttl);
        }
    }
}
```

### Global Cache Instance

```rust
static SCRIPTLET_CACHE: LazyLock<ScriptletCache> = 
    LazyLock::new(|| ScriptletCache::new(Duration::from_secs(60)));

pub fn get_cached_scriptlet(path: &str) -> Option<Arc<Scriptlet>> {
    SCRIPTLET_CACHE.get(path)
}

pub fn cache_scriptlet(path: &str, scriptlet: Scriptlet) {
    SCRIPTLET_CACHE.set(path, scriptlet);
}
```

## File-Based Caching

### Script Metadata Cache

```rust
use std::fs;
use std::path::Path;

const CACHE_DIR: &str = ".scriptkit/cache";

pub fn get_cached_script_metadata(script_path: &Path) -> Option<ScriptMetadata> {
    let cache_path = get_cache_path(script_path);
    
    // Check if cache exists and is newer than script
    if cache_path.exists() {
        let cache_mtime = fs::metadata(&cache_path).ok()?.modified().ok()?;
        let script_mtime = fs::metadata(script_path).ok()?.modified().ok()?;
        
        if cache_mtime >= script_mtime {
            let contents = fs::read_to_string(&cache_path).ok()?;
            return serde_json::from_str(&contents).ok();
        }
    }
    
    None
}

pub fn cache_script_metadata(script_path: &Path, metadata: &ScriptMetadata) -> Result<()> {
    let cache_path = get_cache_path(script_path);
    
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string(metadata)?;
    fs::write(&cache_path, json)?;
    
    Ok(())
}

fn get_cache_path(script_path: &Path) -> PathBuf {
    let hash = blake3::hash(script_path.to_string_lossy().as_bytes());
    let hex = hex::encode(&hash.as_bytes()[..8]); // First 8 bytes
    
    dirs::home_dir()
        .unwrap_or_default()
        .join(CACHE_DIR)
        .join(format!("{}.json", hex))
}
```

## LRU Cache Pattern

```rust
use std::collections::HashMap;
use std::sync::Mutex;

pub struct LruCache<K, V> {
    inner: Mutex<LruCacheInner<K, V>>,
}

struct LruCacheInner<K, V> {
    map: HashMap<K, V>,
    order: Vec<K>,
    capacity: usize,
}

impl<K: Clone + Eq + std::hash::Hash, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(LruCacheInner {
                map: HashMap::new(),
                order: Vec::with_capacity(capacity),
                capacity,
            }),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> 
    where V: Clone {
        let mut inner = self.inner.lock().ok()?;
        
        if let Some(value) = inner.map.get(key) {
            // Move to front (most recently used)
            inner.order.retain(|k| k != key);
            inner.order.push(key.clone());
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut inner) = self.inner.lock() {
            // Remove existing if present
            inner.order.retain(|k| k != &key);
            
            // Evict if at capacity
            while inner.order.len() >= inner.capacity {
                if let Some(old_key) = inner.order.first().cloned() {
                    inner.order.remove(0);
                    inner.map.remove(&old_key);
                }
            }
            
            inner.map.insert(key.clone(), value);
            inner.order.push(key);
        }
    }
}
```

## Cache Invalidation Strategies

### File Watcher Integration

```rust
pub fn setup_cache_invalidation(watcher: &mut FileWatcher) {
    // Invalidate script cache when scripts change
    watcher.watch_dir("~/.scriptkit/scripts", |event| {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in &event.paths {
                    SCRIPTLET_CACHE.invalidate(&path.to_string_lossy());
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    SCRIPTLET_CACHE.invalidate(&path.to_string_lossy());
                }
            }
            _ => {}
        }
    });
}
```

### Time-Based Invalidation

```rust
impl App {
    fn periodic_cache_gc(&self, cx: &mut Context<Self>) {
        cx.spawn(|_this, _cx| async move {
            loop {
                Timer::after(Duration::from_secs(300)).await; // Every 5 minutes
                
                // GC in-memory caches
                SCRIPTLET_CACHE.gc();
                
                // Prune SQLite cache
                let _ = prune_old_cache_entries(86400 * 7); // 7 days
            }
        }).detach();
    }
}
```

### Event-Based Invalidation

```rust
pub enum CacheInvalidationEvent {
    ScriptModified(PathBuf),
    ConfigChanged,
    ThemeChanged,
    AllScripts,
}

static INVALIDATION_CHANNEL: OnceLock<(Sender<CacheInvalidationEvent>, Receiver<CacheInvalidationEvent>)> = OnceLock::new();

pub fn invalidate_cache(event: CacheInvalidationEvent) {
    if let Some((sender, _)) = INVALIDATION_CHANNEL.get() {
        let _ = sender.try_send(event);
    }
}

fn process_invalidation(event: CacheInvalidationEvent) {
    match event {
        CacheInvalidationEvent::ScriptModified(path) => {
            SCRIPTLET_CACHE.invalidate(&path.to_string_lossy());
        }
        CacheInvalidationEvent::ConfigChanged => {
            // Config affects script behavior, clear all
            SCRIPTLET_CACHE.clear();
        }
        CacheInvalidationEvent::ThemeChanged => {
            // Theme doesn't affect scripts, no action
        }
        CacheInvalidationEvent::AllScripts => {
            SCRIPTLET_CACHE.clear();
        }
    }
}
```

## Cache Warming

```rust
pub async fn warm_script_cache() {
    let scripts_dir = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/scripts");
    
    if let Ok(entries) = fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ts" || e == "js") {
                // Parse and cache in background
                tokio::spawn(async move {
                    if let Ok(scriptlet) = parse_scriptlet(&path) {
                        cache_scriptlet(&path.to_string_lossy(), scriptlet);
                    }
                });
            }
        }
    }
}
```

## Cache Statistics

```rust
#[derive(Default)]
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub entries: AtomicU64,
}

impl CacheStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        
        if total > 0.0 { hits / total } else { 0.0 }
    }
}

static CACHE_STATS: LazyLock<CacheStats> = LazyLock::new(CacheStats::default);
```

## Best Practices

1. **Use SQLite for persistent cache** - survives restarts
2. **Use in-memory for hot data** - RwLock + HashMap
3. **Set appropriate TTLs** - balance freshness vs performance
4. **Implement cache warming** - pre-populate on startup
5. **Use file modification times** - cheap validity check
6. **Invalidate on file changes** - integrate with watchers
7. **Periodic GC** - prevent unbounded growth
8. **Monitor hit rates** - optimize based on data

## Summary

| Cache Type | Storage | TTL | Use Case |
|-----------|---------|-----|----------|
| Menu Cache | SQLite | 7 days | App menu hierarchies |
| Scriptlet Cache | Memory | 60s | Parsed script data |
| Metadata Cache | Files | mtime | Script metadata |
| LRU Cache | Memory | Size-based | Hot path data |
