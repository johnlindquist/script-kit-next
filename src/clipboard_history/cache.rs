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
/// Wrapped in Arc to allow cheap clones when passing to UI.
static ENTRY_CACHE: OnceLock<Mutex<Arc<Vec<ClipboardEntryMeta>>>> = OnceLock::new();

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
pub fn get_entry_cache() -> &'static Mutex<Arc<Vec<ClipboardEntryMeta>>> {
    ENTRY_CACHE.get_or_init(|| Mutex::new(Arc::new(Vec::new())))
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
/// Falls back to SQLite if cache is empty.
/// Returns Arc to avoid cloning the entire cache - caller can clone individual entries if needed.
pub fn get_cached_entries(limit: usize) -> Vec<ClipboardEntryMeta> {
    if let Ok(cache) = get_entry_cache().lock() {
        if !cache.is_empty() {
            // Only clone the entries we need, not the entire cache
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
        *cache = Arc::new(Vec::new());
    }
}

/// Refresh the entry cache from database (metadata only, no content payload)
pub fn refresh_entry_cache() {
    // Use metadata-only query to avoid loading full content
    let entries = get_clipboard_history_meta(MAX_CACHED_ENTRIES, 0);
    if let Ok(mut cache) = get_entry_cache().lock() {
        // Replace the Arc with a new one containing the refreshed entries
        *cache = Arc::new(entries);
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

/// Incrementally upsert an entry in the cache.
///
/// This is much faster than refresh_entry_cache() because it:
/// - Doesn't query SQLite
/// - Only updates/inserts a single entry
/// - Re-sorts only when necessary
///
/// Use this after add_entry() instead of refresh_entry_cache().
pub fn upsert_entry_in_cache(entry: ClipboardEntryMeta) {
    if let Ok(mut cache_arc) = get_entry_cache().lock() {
        // Clone the Vec to modify it (Arc::make_mut would clone if refcount > 1)
        let mut cache = (**cache_arc).clone();

        // Remove existing entry with same ID (if any)
        cache.retain(|e| e.id != entry.id);

        // Insert at the correct position (pinned first, then by timestamp desc)
        // For efficiency, we insert at position 0 or after pinned entries
        // since new/touched entries have the latest timestamp
        let insert_pos = if entry.pinned {
            0 // Pinned entries go to front
        } else {
            // Find first non-pinned entry
            cache.iter().position(|e| !e.pinned).unwrap_or(0)
        };

        cache.insert(insert_pos, entry);

        // Truncate to max size
        cache.truncate(MAX_CACHED_ENTRIES);

        debug!(cache_size = cache.len(), "Incremental cache upsert");

        // Replace the Arc with the modified Vec
        *cache_arc = Arc::new(cache);
    }

    // Update timestamp
    update_cache_timestamp();
}

/// Remove an entry from the cache by ID.
///
/// Use this after remove_entry() instead of refresh_entry_cache().
pub fn remove_entry_from_cache(id: &str) {
    if let Ok(mut cache_arc) = get_entry_cache().lock() {
        let mut cache = (**cache_arc).clone();
        let before = cache.len();
        cache.retain(|e| e.id != id);
        if cache.len() < before {
            debug!(id = %id, "Removed entry from cache");
            // Only update Arc if we actually removed something
            *cache_arc = Arc::new(cache);
        }
    }
    update_cache_timestamp();
}

/// Update the pinned status of an entry in the cache.
///
/// Re-sorts the cache to maintain pinned-first ordering.
/// Use this after pin_entry/unpin_entry() instead of refresh_entry_cache().
pub fn update_pin_status_in_cache(id: &str, pinned: bool) {
    if let Ok(mut cache_arc) = get_entry_cache().lock() {
        let mut cache = (**cache_arc).clone();
        if let Some(entry) = cache.iter_mut().find(|e| e.id == id) {
            entry.pinned = pinned;
        }
        // Re-sort: pinned first, then by timestamp descending
        cache.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.timestamp.cmp(&a.timestamp))
        });
        debug!(id = %id, pinned = pinned, "Updated pin status in cache");
        *cache_arc = Arc::new(cache);
    }
    update_cache_timestamp();
}

/// Update OCR text for a single cached entry.
pub(crate) fn update_ocr_text_in_cache(id: &str, text: String) {
    if let Ok(mut cache_arc) = get_entry_cache().lock() {
        let mut cache = (**cache_arc).clone();
        let mut updated = false;

        if let Some(entry) = cache.iter_mut().find(|e| e.id == id) {
            entry.ocr_text = Some(text);
            updated = true;
        }

        if updated {
            debug!(id = %id, "Updated OCR text in cache");
            *cache_arc = Arc::new(cache);
            drop(cache_arc);
            update_cache_timestamp();
        }
    }
}

/// Update the cache timestamp (internal helper)
fn update_cache_timestamp() {
    if let Some(updated) = CACHE_UPDATED.get() {
        if let Ok(mut ts) = updated.lock() {
            *ts = chrono::Utc::now().timestamp_millis();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard_history::types::ContentType;

    fn make_meta(id: &str, timestamp: i64, pinned: bool) -> ClipboardEntryMeta {
        ClipboardEntryMeta {
            id: id.to_string(),
            content_type: ContentType::Text,
            timestamp,
            pinned,
            text_preview: format!("preview-{}", id),
            image_width: None,
            image_height: None,
            byte_size: 10,
            ocr_text: None,
        }
    }

    #[test]
    fn test_upsert_new_entry() {
        // Clear cache first
        invalidate_entry_cache();

        let entry = make_meta("test1", 1000, false);
        upsert_entry_in_cache(entry.clone());

        let cached = get_cached_entries(10);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].id, "test1");
    }

    #[test]
    fn test_upsert_updates_existing() {
        invalidate_entry_cache();

        // Add initial entry
        upsert_entry_in_cache(make_meta("test2", 1000, false));

        // Update with new timestamp
        upsert_entry_in_cache(make_meta("test2", 2000, false));

        let cached = get_cached_entries(10);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].timestamp, 2000);
    }

    #[test]
    fn test_upsert_maintains_pinned_order() {
        invalidate_entry_cache();

        // Add unpinned entry first
        upsert_entry_in_cache(make_meta("unpinned", 2000, false));
        // Add pinned entry
        upsert_entry_in_cache(make_meta("pinned", 1000, true));

        let cached = get_cached_entries(10);
        assert_eq!(cached.len(), 2);
        // Pinned should be first despite lower timestamp
        assert_eq!(cached[0].id, "pinned");
        assert_eq!(cached[1].id, "unpinned");
    }

    #[test]
    fn test_remove_entry_from_cache() {
        invalidate_entry_cache();

        upsert_entry_in_cache(make_meta("to_remove", 1000, false));
        upsert_entry_in_cache(make_meta("to_keep", 2000, false));

        remove_entry_from_cache("to_remove");

        let cached = get_cached_entries(10);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].id, "to_keep");
    }

    #[test]
    fn test_update_pin_status() {
        invalidate_entry_cache();

        upsert_entry_in_cache(make_meta("entry1", 2000, false));
        upsert_entry_in_cache(make_meta("entry2", 1000, false));

        // Pin entry2 (lower timestamp)
        update_pin_status_in_cache("entry2", true);

        let cached = get_cached_entries(10);
        // entry2 should now be first (pinned)
        assert_eq!(cached[0].id, "entry2");
        assert!(cached[0].pinned);
    }

    #[test]
    fn test_update_ocr_text_in_cache_sets_text_when_entry_exists() {
        init_cache_timestamp();
        invalidate_entry_cache();

        upsert_entry_in_cache(make_meta("ocr-target", 3000, false));
        update_ocr_text_in_cache("ocr-target", "recognized text".to_string());

        let cached = get_cached_entries(10);
        assert_eq!(cached[0].ocr_text.as_deref(), Some("recognized text"));
    }
}
