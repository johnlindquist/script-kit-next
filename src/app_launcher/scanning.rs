// ============================================================================
// Application Scanning
// ============================================================================

/// Scan for installed macOS applications
///
/// This function uses a two-phase loading strategy:
/// 1. First, instantly load from SQLite cache (if available) WITHOUT icon decoding
/// 2. Then, scan directories in background to find new/changed apps
///
/// # Returns
/// A reference to the cached vector of AppInfo structs.
///
/// # Performance
/// - First call: Returns SQLite-cached apps in <50ms (icons decoded in background)
/// - Subsequent calls: Returns immediately from in-memory cache
///
/// # Tracing
/// Uses spans to profile: db_lock, query, deserialization, icon_decode
pub fn scan_applications() -> Vec<AppInfo> {
    let _span = info_span!("scan_applications").entered();

    // Initialize the cache if needed
    let cache = APP_CACHE.get_or_init(|| {
        set_loading_state(AppLoadingState::LoadingFromCache);

        let start = Instant::now();

        // Load from SQLite cache with icons decoded synchronously
        let cached_apps = load_apps_from_db();
        let cache_load_ms = start.elapsed().as_millis();

        if !cached_apps.is_empty() {
            info!(
                app_count = cached_apps.len(),
                cache_load_ms, "Cache load complete (icons decoded synchronously)"
            );

            // Create Arc and spawn background thread to scan for new/changed apps
            let cache_arc = Arc::new(Mutex::new(cached_apps));
            let cache_for_thread = Arc::clone(&cache_arc);

            std::thread::spawn(move || {
                let _span = info_span!("background_app_scan").entered();
                set_loading_state(AppLoadingState::ScanningDirectories);

                let scan_start = Instant::now();
                let fresh_apps = scan_all_directories_with_db_update();
                let scan_duration = scan_start.elapsed().as_millis();
                let app_count = fresh_apps.len();

                // Update the in-memory cache (this Arc is shared with APP_CACHE)
                if let Ok(mut guard) = cache_for_thread.lock() {
                    *guard = fresh_apps;
                }

                let (db_count, db_size) = get_apps_db_stats();
                info!(
                    app_count,
                    duration_ms = scan_duration,
                    db_apps = db_count,
                    db_icon_size_kb = db_size / 1024,
                    "Background app scan complete"
                );

                set_loading_state(AppLoadingState::Ready);
            });

            // Return the same Arc that the background thread will update
            return cache_arc;
        }

        // No SQLite cache - do a full synchronous scan
        info!("No SQLite cache found, performing full scan");
        set_loading_state(AppLoadingState::ScanningDirectories);

        let apps = scan_all_directories_with_db_update();
        let duration_ms = start.elapsed().as_millis();

        let (db_count, db_size) = get_apps_db_stats();
        info!(
            app_count = apps.len(),
            duration_ms = duration_ms,
            db_apps = db_count,
            db_icon_size_kb = db_size / 1024,
            "Scanned applications (no cache)"
        );

        set_loading_state(AppLoadingState::Ready);

        Arc::new(Mutex::new(apps))
    });

    // Return a clone of the cached apps
    cache.lock().map(|g| g.clone()).unwrap_or_default()
}

/// Force a fresh scan of applications (bypasses cache)
///
/// This is useful if you need to detect newly installed applications.
/// Note: This does NOT update the static cache - it just returns fresh results.
#[allow(dead_code)]
pub fn scan_applications_fresh() -> Vec<AppInfo> {
    let start = Instant::now();
    let apps = scan_all_directories_with_db_update();
    let duration_ms = start.elapsed().as_millis();

    info!(
        app_count = apps.len(),
        duration_ms = duration_ms,
        "Fresh scan of applications"
    );

    apps
}

/// Reset icon extraction stats before a new scan
fn reset_icon_stats() {
    ICONS_EXTRACTED.store(0, Ordering::Relaxed);
    ICONS_FROM_CACHE.store(0, Ordering::Relaxed);
    EXTRACT_TIME_MS.store(0, Ordering::Relaxed);
}

/// Log a summary of icon extraction stats
fn log_icon_stats_summary() {
    let extracted = ICONS_EXTRACTED.load(Ordering::Relaxed);
    let from_cache = ICONS_FROM_CACHE.load(Ordering::Relaxed);
    let total_ms = EXTRACT_TIME_MS.load(Ordering::Relaxed);

    if extracted > 0 || from_cache > 0 {
        info!(
            icons_extracted = extracted,
            icons_from_cache = from_cache,
            total_extract_ms = total_ms,
            avg_extract_ms = if extracted > 0 {
                total_ms / extracted
            } else {
                0
            },
            "Icon extraction summary"
        );
    }
}

/// Scan all configured directories for applications and update SQLite
fn scan_all_directories_with_db_update() -> Vec<AppInfo> {
    let _span = info_span!("scan_all_directories_with_db_update").entered();
    let start = Instant::now();

    // Reset stats for this scan
    reset_icon_stats();

    let mut apps = Vec::new();
    let mut dirs_scanned = 0;

    for dir in APP_DIRECTORIES {
        let expanded = shellexpand::tilde(dir);
        let path = Path::new(expanded.as_ref());

        if path.exists() {
            let dir_start = Instant::now();
            match scan_directory_with_db_update(path) {
                Ok(found) => {
                    let count = found.len();
                    trace!(
                        directory = %path.display(),
                        count,
                        duration_ms = dir_start.elapsed().as_millis(),
                        "Scanned directory"
                    );
                    apps.extend(found);
                    dirs_scanned += 1;
                }
                Err(e) => {
                    warn!(
                        directory = %path.display(),
                        error = %e,
                        "Failed to scan directory"
                    );
                }
            }
        } else {
            trace!(directory = %path.display(), "Directory does not exist, skipping");
        }
    }

    // Sort by name for consistent ordering
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Remove duplicates (same name from different directories - prefer first)
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    // Log icon extraction summary (batched instead of per-app)
    log_icon_stats_summary();

    debug!(
        total_apps = apps.len(),
        dirs_scanned,
        total_duration_ms = start.elapsed().as_millis(),
        "Directory scan complete"
    );

    apps
}

/// Scan a single directory for .app bundles and update SQLite
///
/// Uses parallel iteration (rayon) for icon extraction which is the bottleneck.
fn scan_directory_with_db_update(dir: &Path) -> Result<Vec<AppInfo>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    // Collect app paths first (fast, just directory listing)
    let app_paths: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().map(|e| e == "app").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Process apps in parallel using rayon (icon extraction is the bottleneck)
    let apps: Vec<AppInfo> = app_paths
        .par_iter()
        .filter_map(|path| {
            if let Some((app_info, icon_bytes)) = parse_app_bundle_with_icon(path) {
                // Save to SQLite (thread-safe via mutex in get_apps_db)
                let mtime = get_mtime(path).unwrap_or(0);
                save_app_to_db(&app_info, icon_bytes.as_deref(), mtime);
                Some(app_info)
            } else {
                None
            }
        })
        .collect();

    Ok(apps)
}

/// Parse a .app bundle to extract application information and icon bytes
fn parse_app_bundle_with_icon(path: &Path) -> Option<(AppInfo, Option<Vec<u8>>)> {
    // Extract app name from bundle name (strip .app extension)
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;

    // Try to extract bundle identifier from Info.plist
    let bundle_id = extract_bundle_id(path);

    // Extract icon (macOS only)
    #[cfg(target_os = "macos")]
    let icon_bytes = get_or_extract_icon(path);
    #[cfg(not(target_os = "macos"))]
    let icon_bytes: Option<Vec<u8>> = None;

    // Pre-decode icon for rendering
    let icon = icon_bytes.as_ref().and_then(|bytes| {
        crate::list_item::decode_png_to_render_image_with_bgra_conversion(bytes).ok()
    });

    Some((
        AppInfo {
            name,
            path: path.to_path_buf(),
            bundle_id,
            icon,
        },
        icon_bytes,
    ))
}

/// Scan all configured directories for applications (legacy, no DB update)
#[allow(dead_code)]
fn scan_all_directories() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    for dir in APP_DIRECTORIES {
        let expanded = shellexpand::tilde(dir);
        let path = Path::new(expanded.as_ref());

        if path.exists() {
            match scan_directory(path) {
                Ok(found) => {
                    debug!(
                        directory = %path.display(),
                        count = found.len(),
                        "Scanned directory"
                    );
                    apps.extend(found);
                }
                Err(e) => {
                    warn!(
                        directory = %path.display(),
                        error = %e,
                        "Failed to scan directory"
                    );
                }
            }
        } else {
            debug!(directory = %path.display(), "Directory does not exist, skipping");
        }
    }

    // Sort by name for consistent ordering
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Remove duplicates (same name from different directories - prefer first)
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());

    apps
}

/// Scan a single directory for .app bundles (legacy, no DB update)
fn scan_directory(dir: &Path) -> Result<Vec<AppInfo>> {
    let mut apps = Vec::new();

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Check if it's a .app bundle
        if let Some(extension) = path.extension() {
            if extension == "app" {
                if let Some(app_info) = parse_app_bundle(&path) {
                    apps.push(app_info);
                }
            }
        }
    }

    Ok(apps)
}

/// Parse a .app bundle to extract application information (legacy)
fn parse_app_bundle(path: &Path) -> Option<AppInfo> {
    // Extract app name from bundle name (strip .app extension)
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;

    // Try to extract bundle identifier from Info.plist
    let bundle_id = extract_bundle_id(path);

    // Extract and pre-decode icon using disk cache (macOS only)
    // Uses get_or_extract_icon() which checks disk cache first, only extracts if stale/missing
    // Pre-decoding here is CRITICAL for performance - avoids PNG decode on every render
    // Uses decode_png_to_render_image_with_bgra_conversion for Metal compatibility
    #[cfg(target_os = "macos")]
    let icon = get_or_extract_icon(path).and_then(|png_bytes| {
        crate::list_item::decode_png_to_render_image_with_bgra_conversion(&png_bytes).ok()
    });
    #[cfg(not(target_os = "macos"))]
    let icon = None;

    Some(AppInfo {
        name,
        path: path.to_path_buf(),
        bundle_id,
        icon,
    })
}

/// Extract CFBundleIdentifier from Info.plist
///
/// Uses /usr/libexec/PlistBuddy for reliable plist parsing.
fn extract_bundle_id(app_path: &Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");

    if !plist_path.exists() {
        return None;
    }

    // Use PlistBuddy to extract CFBundleIdentifier (reliable and fast)
    let output = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleIdentifier", plist_path.to_str()?])
        .output()
        .ok()?;

    if output.status.success() {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !bundle_id.is_empty() {
            return Some(bundle_id);
        }
    }

    None
}

/// Extract application icon using NSWorkspace
///
/// Uses macOS Cocoa APIs to get the icon for an application bundle.
/// The icon is converted to PNG format at 32x32 pixels for list display.
/// Returns raw PNG bytes - caller should decode once and cache the RenderImage.
#[cfg(target_os = "macos")]
fn extract_app_icon(app_path: &Path) -> Option<Vec<u8>> {
    use std::slice;

    let path_str = app_path.to_str()?;

    unsafe {
        // Get NSWorkspace shared instance
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }

        // Create NSString for path
        let ns_path = CocoaNSString::alloc(nil).init_str(path_str);
        if ns_path == nil {
            return None;
        }

        // Get icon for file
        let icon: id = msg_send![workspace, iconForFile: ns_path];
        if icon == nil {
            return None;
        }

        // Set the icon size to 32x32 for list display
        let size = cocoa::foundation::NSSize::new(32.0, 32.0);
        let _: () = msg_send![icon, setSize: size];

        // Get TIFF representation
        let tiff_data: id = msg_send![icon, TIFFRepresentation];
        if tiff_data == nil {
            return None;
        }

        // Create bitmap image rep from TIFF data
        let bitmap_rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff_data];
        if bitmap_rep == nil {
            return None;
        }

        // Convert to PNG (NSPNGFileType = 4)
        let empty_dict: id = msg_send![class!(NSDictionary), dictionary];
        let png_data: id = msg_send![
            bitmap_rep,
            representationUsingType: 4u64  // NSPNGFileType
            properties: empty_dict
        ];
        if png_data == nil {
            return None;
        }

        // Get bytes from NSData
        let length: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];

        if bytes.is_null() || length == 0 {
            return None;
        }

        // Copy bytes to Vec<u8>
        let png_bytes = slice::from_raw_parts(bytes, length).to_vec();

        Some(png_bytes)
    }
}
