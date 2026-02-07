// ============================================================================
// Legacy filesystem cache (kept for backward compat during migration)
// ============================================================================

/// Get the icon cache directory path (~/.scriptkit/cache/app-icons/)
fn get_icon_cache_dir() -> Option<PathBuf> {
    let kit = PathBuf::from(shellexpand::tilde("~/.scriptkit").as_ref());
    Some(kit.join("cache").join("app-icons"))
}

/// Generate a unique cache key from an app path using a hash
fn hash_path(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get cached icon or extract fresh one if cache is stale/missing
///
/// Cache invalidation is based on the app bundle's modification time.
/// The cache file's mtime is set to match the app's mtime for easy comparison.
#[cfg(target_os = "macos")]
fn get_or_extract_icon(app_path: &Path) -> Option<Vec<u8>> {
    let start = Instant::now();
    let cache_dir = get_icon_cache_dir()?;
    let cache_key = hash_path(app_path);
    let cache_file = cache_dir.join(format!("{}.png", cache_key));

    // Get app's modification time
    let app_mtime = app_path.metadata().ok()?.modified().ok()?;

    // Check if cache file exists and is valid
    if cache_file.exists() {
        if let Ok(cache_meta) = cache_file.metadata() {
            if let Ok(cache_mtime) = cache_meta.modified() {
                // Cache is valid if its mtime matches or is newer than app mtime
                if cache_mtime >= app_mtime {
                    // Load from cache
                    if let Ok(png_bytes) = std::fs::read(&cache_file) {
                        ICONS_FROM_CACHE.fetch_add(1, Ordering::Relaxed);
                        trace!(
                            app = %app_path.display(),
                            duration_ms = start.elapsed().as_millis(),
                            source = "disk_cache",
                            "Loaded icon"
                        );
                        return Some(png_bytes);
                    }
                }
            }
        }
    }

    // Cache miss or stale - extract fresh icon
    // Note: Color channel swap (BGRA -> RGBA) is handled at decode time in
    // decode_png_to_render_image_with_rb_swap() for performance (no PNG re-encoding needed)
    let extract_start = Instant::now();
    let png_bytes = extract_app_icon(app_path)?;
    let extract_ms = extract_start.elapsed().as_millis();

    // Save to cache
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        warn!(
            error = %e,
            cache_dir = %cache_dir.display(),
            "Failed to create icon cache directory"
        );
    } else if let Err(e) = std::fs::write(&cache_file, &png_bytes) {
        warn!(
            error = %e,
            cache_file = %cache_file.display(),
            "Failed to write icon to cache"
        );
    } else {
        // Set cache file mtime to app mtime for future comparison
        let file_time = filetime::FileTime::from_system_time(app_mtime);
        if let Err(e) = filetime::set_file_mtime(&cache_file, file_time) {
            warn!(
                error = %e,
                cache_file = %cache_file.display(),
                "Failed to set cache file mtime"
            );
        } else {
            // Update stats for summary log (instead of per-app logging)
            ICONS_EXTRACTED.fetch_add(1, Ordering::Relaxed);
            EXTRACT_TIME_MS.fetch_add(extract_ms as usize, Ordering::Relaxed);
            trace!(
                app = %app_path.display(),
                extract_ms,
                total_ms = start.elapsed().as_millis(),
                source = "extracted",
                "Extracted and cached icon"
            );
        }
    }

    Some(png_bytes)
}

/// Get icon cache statistics
///
/// Returns (cache_file_count, total_size_bytes) for the icon cache directory.
/// Useful for logging and monitoring cache behavior.
#[allow(dead_code)]
pub fn get_icon_cache_stats() -> (usize, u64) {
    let cache_dir = match get_icon_cache_dir() {
        Some(dir) => dir,
        None => return (0, 0),
    };

    if !cache_dir.exists() {
        return (0, 0);
    }

    let mut count = 0;
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    count += 1;
                    total_size += metadata.len();
                }
            }
        }
    }

    (count, total_size)
}
