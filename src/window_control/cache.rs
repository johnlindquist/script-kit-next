use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::cf::*;
use super::ffi::*;

/// Global window cache using OnceLock (std alternative to lazy_static)
pub(super) static WINDOW_CACHE: OnceLock<Mutex<HashMap<u32, usize>>> = OnceLock::new();

/// An owned cached window reference retained while in use.
pub(super) struct OwnedCachedWindowRef {
    window_ref: AXUIElementRef,
}

impl OwnedCachedWindowRef {
    pub(super) fn as_ptr(&self) -> AXUIElementRef {
        self.window_ref
    }
}

impl Drop for OwnedCachedWindowRef {
    fn drop(&mut self) {
        cf_release(self.window_ref as CFTypeRef);
    }
}

/// Get or initialize the window cache
pub(super) fn get_cache() -> &'static Mutex<HashMap<u32, usize>> {
    WINDOW_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn cache_window(id: u32, window_ref: AXUIElementRef) {
    if let Ok(mut cache) = get_cache().lock() {
        if let Some(previous) = cache.insert(id, window_ref as usize) {
            // The cache owns retained window references. Replacing an entry must
            // release the previous retained pointer to avoid leaks.
            cf_release(previous as CFTypeRef);
        }
    }
}

pub(super) fn get_cached_window(id: u32) -> Option<OwnedCachedWindowRef> {
    let cache = get_cache().lock().ok()?;
    let ptr = *cache.get(&id)?;
    let retained = cf_retain(ptr as CFTypeRef) as AXUIElementRef;
    if retained.is_null() {
        None
    } else {
        Some(OwnedCachedWindowRef {
            window_ref: retained,
        })
    }
}

pub(super) fn clear_window_cache() {
    if let Ok(mut cache) = get_cache().lock() {
        // Release all retained window refs before clearing
        for &window_ptr in cache.values() {
            cf_release(window_ptr as CFTypeRef);
        }
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    fn cf_get_retain_count(cf: CFTypeRef) -> isize {
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFGetRetainCount(cf: CFTypeRef) -> isize;
        }

        unsafe { CFGetRetainCount(cf) }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_window_cache_releases_previous_pointer_on_overwrite() {
        clear_window_cache();

        let window_id = 0xCAFE_1000;
        let first_window =
            try_create_cf_string("window-cache-overwrite-first").expect("valid CFString literal");
        let second_window =
            try_create_cf_string("window-cache-overwrite-second").expect("valid CFString literal");

        cache_window(window_id, cf_retain(first_window) as AXUIElementRef);
        let first_after_insert = cf_get_retain_count(first_window);

        cache_window(window_id, cf_retain(second_window) as AXUIElementRef);
        let first_after_overwrite = cf_get_retain_count(first_window);

        assert_eq!(
            first_after_overwrite + 1,
            first_after_insert,
            "cache overwrite should release old retained window pointer"
        );

        clear_window_cache();
        cf_release(first_window);
        cf_release(second_window);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_window_cache_get_returns_owned_reference_and_releases_on_drop() {
        clear_window_cache();

        let window_id = 0xCAFE_2000;
        let window =
            try_create_cf_string("window-cache-owned-get").expect("valid CFString literal");
        cache_window(window_id, cf_retain(window) as AXUIElementRef);

        let before_get = cf_get_retain_count(window);
        let owned = get_cached_window(window_id).expect("window should exist in cache");
        assert_eq!(owned.as_ptr(), window as AXUIElementRef);

        let during_get = cf_get_retain_count(window);
        assert_eq!(
            during_get,
            before_get + 1,
            "get_cached_window should retain before returning"
        );

        drop(owned);
        let after_drop = cf_get_retain_count(window);
        assert_eq!(
            after_drop, before_get,
            "dropping owned cached window should release retained reference"
        );

        clear_window_cache();
        cf_release(window);
    }
}
