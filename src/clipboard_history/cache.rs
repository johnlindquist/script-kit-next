//! Clipboard history caching
//!
//! LRU caching for decoded images and entry metadata.

use gpui::RenderImage;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::debug;

use super::database::get_clipboard_history_meta;
use super::types::ClipboardEntryMeta;

/// Maximum number of decoded images to keep in memory (LRU eviction)
/// Each image can be 1-4MB, so 100 images = ~100-400MB max memory
pub const MAX_IMAGE_CACHE_ENTRIES: usize = 100;

/// Maximum entries to cache in memory for fast access
pub const MAX_CACHED_ENTRIES: usize = 500;

/// Global image cache for decoded RenderImages (thread-safe)
/// Key: entry ID, Value: decoded RenderImage
/// Uses LRU eviction to cap memory usage at ~100-400MB (100 images max)
static IMAGE_CACHE: OnceLock<Mutex<LruCache<String, Arc<RenderImage>>>> = OnceLock::new();

/// Cached clipboard entry metadata (NO content payload) for list views
/// Updated whenever a new entry is added. This is memory-efficient because
/// it doesn't include the full content field (which can be megabytes for images).
static ENTRY_CACHE: OnceLock<Mutex<Vec<ClipboardEntryMeta>>> = OnceLock::new();

/// Timestamp of last cache update
static CACHE_UPDATED: OnceLock<Mutex<i64>> = OnceLock::new();

/// Get the global image cache, initializing if needed
pub fn get_image_cache() -> &'static Mutex<LruCache<String, Arc<RenderImage>>> {
    IMAGE_CACHE.get_or_init(|| {
        let cap = NonZeroUsize::new(MAX_IMAGE_CACHE_ENTRIES).expect("cache size must be non-zero");
        Mutex::new(LruCache::new(cap))
    })
}

/// Get the global entry cache, initializing if needed
pub fn get_entry_cache() -> &'static Mutex<Vec<ClipboardEntryMeta>> {
    ENTRY_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Initialize the cache timestamp tracker
pub fn init_cache_timestamp() {
    let _ = CACHE_UPDATED.set(Mutex::new(0));
}

/// Get cached image by entry ID (updates LRU order)
pub fn get_cached_image(id: &str) -> Option<Arc<RenderImage>> {
    get_image_cache().lock().ok()?.get(id).cloned()
}

/// Cache a decoded image (with LRU eviction at MAX_IMAGE_CACHE_ENTRIES limit)
pub fn cache_image(id: &str, image: Arc<RenderImage>) {
    if let Ok(mut cache) = get_image_cache().lock() {
        // LruCache automatically evicts oldest entry when capacity is exceeded
        let evicted = cache.len() >= cache.cap().get();
        cache.put(id.to_string(), image);
        if evicted {
            debug!(
                id = %id,
                cache_size = cache.len(),
                max_size = MAX_IMAGE_CACHE_ENTRIES,
                "Cached decoded image (evicted oldest)"
            );
        } else {
            debug!(id = %id, cache_size = cache.len(), "Cached decoded image");
        }
    }
}

/// Get cached clipboard entry metadata (faster than querying SQLite)
/// Falls back to SQLite if cache is empty
pub fn get_cached_entries(limit: usize) -> Vec<ClipboardEntryMeta> {
    if let Ok(cache) = get_entry_cache().lock() {
        if !cache.is_empty() {
            let result: Vec<_> = cache.iter().take(limit).cloned().collect();
            debug!(
                count = result.len(),
                cached = true,
                "Retrieved clipboard entry metadata from cache"
            );
            return result;
        }
    }
    // Fall back to database (metadata-only query)
    get_clipboard_history_meta(limit, 0)
}

/// Invalidate the entry cache (called when entries change)
pub fn invalidate_entry_cache() {
    if let Ok(mut cache) = get_entry_cache().lock() {
        cache.clear();
    }
}

/// Refresh the entry cache from database (metadata only, no content payload)
pub fn refresh_entry_cache() {
    // Use metadata-only query to avoid loading full content
    let entries = get_clipboard_history_meta(MAX_CACHED_ENTRIES, 0);
    if let Ok(mut cache) = get_entry_cache().lock() {
        *cache = entries;
        debug!(count = cache.len(), "Refreshed entry metadata cache");
    }
    if let Some(updated) = CACHE_UPDATED.get() {
        if let Ok(mut ts) = updated.lock() {
            *ts = chrono::Utc::now().timestamp_millis();
        }
    }
}

/// Evict a single entry from the image cache
pub fn evict_image_cache(id: &str) {
    if let Some(cache) = IMAGE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.pop(id);
            debug!(id = %id, "Evicted image from cache");
        }
    }
}

/// Clear all caches (entry + image)
pub fn clear_all_caches() {
    invalidate_entry_cache();
    if let Some(cache) = IMAGE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.clear();
            debug!("Cleared image cache");
        }
    }
}
