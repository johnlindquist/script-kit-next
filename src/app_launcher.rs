//! App Launcher Module
//!
//! Provides functionality to scan and launch macOS applications.
//!
//! ## Features
//! - Scans standard macOS application directories
//! - Caches apps and icons in SQLite for instant startup (~/.scriptkit/db/apps.sqlite)
//! - Extracts bundle identifiers from Info.plist when available
//! - Extracts app icons using NSWorkspace for display
//! - Launches applications via `open -a`
//! - Tracks loading state for UI feedback
//!
//! ## Loading States
//! - LoadingFromCache: Initial load from SQLite (instant)
//! - ScanningDirectories: Background directory scan in progress
//! - Ready: All apps loaded and up to date
//!

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tracing::{debug, error, info, warn};

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString as CocoaNSString;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

/// Pre-decoded icon image for efficient rendering
pub type DecodedIcon = Arc<gpui::RenderImage>;

/// Information about an installed application
#[derive(Clone)]
pub struct AppInfo {
    /// Display name of the application (e.g., "Safari")
    pub name: String,
    /// Full path to the .app bundle (e.g., "/Applications/Safari.app")
    pub path: PathBuf,
    /// Bundle identifier from Info.plist (e.g., "com.apple.Safari")
    pub bundle_id: Option<String>,
    /// Pre-decoded icon image (32x32), ready for rendering
    /// **IMPORTANT**: This is pre-decoded to avoid PNG decoding on every render frame
    pub icon: Option<DecodedIcon>,
}

impl std::fmt::Debug for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppInfo")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("bundle_id", &self.bundle_id)
            .field("icon", &self.icon.as_ref().map(|_| "<RenderImage>"))
            .finish()
    }
}

/// Loading state for the app cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLoadingState {
    /// Initial load from SQLite cache (instant, no disk scan)
    LoadingFromCache,
    /// Background directory scan in progress to find new/changed apps
    ScanningDirectories,
    /// All apps loaded and cache is up to date
    Ready,
}

impl AppLoadingState {
    /// Get a human-readable message for UI display
    #[allow(dead_code)]
    pub fn message(&self) -> &'static str {
        match self {
            AppLoadingState::LoadingFromCache => "Loading apps...",
            AppLoadingState::ScanningDirectories => "Scanning for new apps...",
            AppLoadingState::Ready => "Apps ready",
        }
    }
}

/// Cached list of applications (in-memory, populated from SQLite + directory scan)
static APP_CACHE: OnceLock<Arc<Mutex<Vec<AppInfo>>>> = OnceLock::new();

/// Current loading state (thread-safe, updated during scan)
static APP_LOADING_STATE: OnceLock<Mutex<AppLoadingState>> = OnceLock::new();

/// Database connection for apps cache
static APPS_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Directories to scan for .app bundles
const APP_DIRECTORIES: &[&str] = &[
    // Standard macOS app locations
    "/Applications",
    "/System/Applications",
    "/System/Applications/Utilities",
    "/Applications/Utilities",
    // User-specific apps
    "~/Applications",
    // Chrome installed web apps (PWAs)
    "~/Applications/Chrome Apps.localized",
    // Edge installed web apps (PWAs)
    "~/Applications/Edge Apps.localized",
    // Arc browser installed web apps
    "~/Applications/Arc Apps",
    // Setapp subscription apps (if installed)
    "/Applications/Setapp",
];

// ============================================================================
// SQLite Database Functions
// ============================================================================

/// Get the apps database path (~/.scriptkit/db/apps.sqlite)
fn get_apps_db_path() -> PathBuf {
    let kit = PathBuf::from(shellexpand::tilde("~/.scriptkit").as_ref());
    kit.join("db").join("apps.sqlite")
}

/// Initialize the apps database schema
fn init_apps_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            bundle_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            icon_blob BLOB,
            mtime INTEGER NOT NULL,
            last_seen INTEGER NOT NULL
        )",
        [],
    )
    .context("Failed to create apps table")?;

    // Index for path lookups (used during directory scan)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_apps_path ON apps(path)", [])
        .context("Failed to create path index")?;

    Ok(())
}

/// Get or initialize the apps database connection
fn get_apps_db() -> Result<Arc<Mutex<Connection>>> {
    if let Some(db) = APPS_DB.get() {
        return Ok(Arc::clone(db));
    }

    let db_path = get_apps_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create db directory")?;
    }

    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open apps database: {}", db_path.display()))?;

    init_apps_db(&conn)?;

    let db = Arc::new(Mutex::new(conn));

    // Try to store it, but another thread might beat us
    match APPS_DB.set(Arc::clone(&db)) {
        Ok(()) => Ok(db),
        Err(_) => {
            // Another thread initialized it first, use theirs
            Ok(Arc::clone(APPS_DB.get().unwrap()))
        }
    }
}

/// Set the current loading state
fn set_loading_state(state: AppLoadingState) {
    let mutex = APP_LOADING_STATE.get_or_init(|| Mutex::new(AppLoadingState::LoadingFromCache));
    if let Ok(mut guard) = mutex.lock() {
        *guard = state;
    }
}

/// Get the current loading state
#[allow(dead_code)]
pub fn get_app_loading_state() -> AppLoadingState {
    APP_LOADING_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|g| *g)
        .unwrap_or(AppLoadingState::Ready)
}

/// Get a human-readable message for the current loading state
#[allow(dead_code)]
pub fn get_app_loading_message() -> &'static str {
    get_app_loading_state().message()
}

/// Check if apps are still loading
#[allow(dead_code)]
pub fn is_apps_loading() -> bool {
    get_app_loading_state() != AppLoadingState::Ready
}

/// Get the in-memory app cache (may be empty if not yet loaded)
#[allow(dead_code)]
pub fn get_cached_apps() -> Vec<AppInfo> {
    APP_CACHE
        .get()
        .and_then(|arc| arc.lock().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Get modification time for a path as Unix timestamp
fn get_mtime(path: &Path) -> Option<i64> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

// ============================================================================
// SQLite Cache Operations
// ============================================================================

/// Load all apps from the SQLite cache
///
/// Returns apps with their icons already decoded as RenderImages.
/// This is the fast path for startup - no filesystem scanning needed.
fn load_apps_from_db() -> Vec<AppInfo> {
    let db = match get_apps_db() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, "Failed to get apps database");
            return Vec::new();
        }
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to lock apps database");
            return Vec::new();
        }
    };

    let mut stmt = match conn
        .prepare("SELECT bundle_id, name, path, icon_blob FROM apps ORDER BY name COLLATE NOCASE")
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "Failed to prepare apps query");
            return Vec::new();
        }
    };

    let apps_iter = stmt.query_map([], |row| {
        let bundle_id: Option<String> = row.get(0)?;
        let name: String = row.get(1)?;
        let path_str: String = row.get(2)?;
        let icon_blob: Option<Vec<u8>> = row.get(3)?;

        Ok((bundle_id, name, path_str, icon_blob))
    });

    let mut apps = Vec::new();

    if let Ok(iter) = apps_iter {
        for (bundle_id, name, path_str, icon_blob) in iter.flatten() {
            let path = PathBuf::from(&path_str);

            // Skip apps that no longer exist
            if !path.exists() {
                continue;
            }

            // Decode icon if present
            let icon = icon_blob.and_then(|bytes| {
                crate::list_item::decode_png_to_render_image_with_bgra_conversion(&bytes).ok()
            });

            apps.push(AppInfo {
                name,
                path,
                bundle_id,
                icon,
            });
        }
    }

    apps
}

/// Save or update an app in the SQLite cache
fn save_app_to_db(app: &AppInfo, icon_bytes: Option<&[u8]>, mtime: i64) {
    let db = match get_apps_db() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, app = %app.name, "Failed to get apps database for save");
            return;
        }
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, app = %app.name, "Failed to lock apps database for save");
            return;
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let path_str = app.path.to_string_lossy().to_string();
    let bundle_id = app.bundle_id.as_deref().unwrap_or(&path_str);

    let result = conn.execute(
        "INSERT INTO apps (bundle_id, name, path, icon_blob, mtime, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(bundle_id) DO UPDATE SET
             name = excluded.name,
             path = excluded.path,
             icon_blob = COALESCE(excluded.icon_blob, apps.icon_blob),
             mtime = excluded.mtime,
             last_seen = excluded.last_seen",
        params![bundle_id, app.name, path_str, icon_bytes, mtime, now],
    );

    if let Err(e) = result {
        warn!(error = %e, app = %app.name, "Failed to save app to database");
    }
}

/// Check if an app needs to be updated in the cache
///
/// Returns true if the app's mtime is newer than what's cached.
#[allow(dead_code)]
fn app_needs_update(path: &Path, current_mtime: i64) -> bool {
    let db = match get_apps_db() {
        Ok(db) => db,
        Err(_) => return true, // If we can't check, assume it needs update
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => return true,
    };

    let path_str = path.to_string_lossy().to_string();

    let cached_mtime: Result<i64, _> = conn.query_row(
        "SELECT mtime FROM apps WHERE path = ?1",
        params![path_str],
        |row| row.get(0),
    );

    match cached_mtime {
        Ok(mtime) => current_mtime > mtime,
        Err(_) => true, // Not in cache, needs update
    }
}

/// Get database statistics for logging
pub fn get_apps_db_stats() -> (usize, u64) {
    let db = match get_apps_db() {
        Ok(db) => db,
        Err(_) => return (0, 0),
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM apps", [], |row| row.get(0))
        .unwrap_or(0);

    let total_icon_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(LENGTH(icon_blob)), 0) FROM apps WHERE icon_blob IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    (count, total_icon_size as u64)
}

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
                        debug!(
                            app = %app_path.display(),
                            cache_file = %cache_file.display(),
                            "Loaded icon from cache"
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
    let png_bytes = extract_app_icon(app_path)?;

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
            debug!(
                app = %app_path.display(),
                cache_file = %cache_file.display(),
                "Saved icon to cache"
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

// ============================================================================
// Application Scanning
// ============================================================================

/// Scan for installed macOS applications
///
/// This function uses a two-phase loading strategy:
/// 1. First, instantly load from SQLite cache (if available)
/// 2. Then, scan directories in background to find new/changed apps
///
/// # Returns
/// A reference to the cached vector of AppInfo structs.
///
/// # Performance
/// - First call: Returns SQLite-cached apps instantly, then background scans
/// - Subsequent calls: Returns immediately from in-memory cache
pub fn scan_applications() -> Vec<AppInfo> {
    // Initialize the cache if needed
    let cache = APP_CACHE.get_or_init(|| {
        set_loading_state(AppLoadingState::LoadingFromCache);

        let start = Instant::now();

        // Phase 1: Load from SQLite (instant)
        let cached_apps = load_apps_from_db();
        let db_duration = start.elapsed().as_millis();

        if !cached_apps.is_empty() {
            info!(
                app_count = cached_apps.len(),
                duration_ms = db_duration,
                "Loaded apps from SQLite cache"
            );

            // Start background scan for updates
            // IMPORTANT: Create ONE Arc and return it. The background thread gets a clone
            // of the same Arc so its updates are visible to callers.
            let cache_arc = Arc::new(Mutex::new(cached_apps));
            let cache_for_thread = Arc::clone(&cache_arc);

            std::thread::spawn(move || {
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

/// Scan all configured directories for applications and update SQLite
fn scan_all_directories_with_db_update() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    for dir in APP_DIRECTORIES {
        let expanded = shellexpand::tilde(dir);
        let path = Path::new(expanded.as_ref());

        if path.exists() {
            match scan_directory_with_db_update(path) {
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

/// Scan a single directory for .app bundles and update SQLite
fn scan_directory_with_db_update(dir: &Path) -> Result<Vec<AppInfo>> {
    let mut apps = Vec::new();

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Check if it's a .app bundle
        if let Some(extension) = path.extension() {
            if extension == "app" {
                if let Some((app_info, icon_bytes)) = parse_app_bundle_with_icon(&path) {
                    // Save to SQLite
                    let mtime = get_mtime(&path).unwrap_or(0);
                    save_app_to_db(&app_info, icon_bytes.as_deref(), mtime);

                    apps.push(app_info);
                }
            }
        }
    }

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

/// Launch an application
///
/// Uses macOS `open -a` command to launch the application.
///
/// # Arguments
/// * `app` - The application to launch
///
/// # Returns
/// Ok(()) if the application was launched successfully, Err otherwise.
///
pub fn launch_application(app: &AppInfo) -> Result<()> {
    info!(
        app_name = %app.name,
        app_path = %app.path.display(),
        "Launching application"
    );

    Command::new("open")
        .arg("-a")
        .arg(&app.path)
        .spawn()
        .with_context(|| format!("Failed to launch application: {}", app.name))?;

    Ok(())
}

/// Launch an application by name
///
/// Convenience function that looks up an application by name and launches it.
///
/// # Arguments
/// * `name` - The name of the application (case-insensitive)
///
/// # Returns
/// Ok(()) if the application was found and launched, Err otherwise.
#[allow(dead_code)]
pub fn launch_application_by_name(name: &str) -> Result<()> {
    let apps = scan_applications();
    let name_lower = name.to_lowercase();

    let app = apps
        .iter()
        .find(|a| a.name.to_lowercase() == name_lower)
        .with_context(|| format!("Application not found: {}", name))?;

    launch_application(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_applications_returns_apps() {
        let apps = scan_applications();

        // Should find at least some apps on any macOS system
        assert!(
            !apps.is_empty(),
            "Should find at least some applications on macOS"
        );

        // Check that Calculator exists (it's always present in /System/Applications on macOS)
        let calculator = apps.iter().find(|a| a.name == "Calculator");
        assert!(calculator.is_some(), "Calculator.app should be found");

        if let Some(calculator) = calculator {
            assert!(
                calculator.path.exists(),
                "Calculator path should exist: {:?}",
                calculator.path
            );
            assert!(
                calculator.bundle_id.is_some(),
                "Calculator should have a bundle ID"
            );
            assert_eq!(
                calculator.bundle_id.as_deref(),
                Some("com.apple.calculator"),
                "Calculator bundle ID should be com.apple.calculator"
            );
        }
    }

    #[test]
    fn test_app_info_has_required_fields() {
        let apps = scan_applications();

        for app in apps.iter().take(10) {
            // Name should not be empty
            assert!(!app.name.is_empty(), "App name should not be empty");

            // Path should end with .app
            assert!(
                app.path.extension().map(|e| e == "app").unwrap_or(false),
                "App path should end with .app: {:?}",
                app.path
            );

            // Path should exist
            assert!(app.path.exists(), "App path should exist: {:?}", app.path);
        }
    }

    #[test]
    fn test_apps_sorted_alphabetically() {
        let apps = scan_applications();

        // Verify apps are sorted by lowercase name
        for window in apps.windows(2) {
            let a = &window[0];
            let b = &window[1];
            assert!(
                a.name.to_lowercase() <= b.name.to_lowercase(),
                "Apps should be sorted: {} should come before {}",
                a.name,
                b.name
            );
        }
    }

    #[test]
    fn test_extract_bundle_id_finder() {
        let finder_path = Path::new("/System/Applications/Finder.app");
        if finder_path.exists() {
            let bundle_id = extract_bundle_id(finder_path);
            assert_eq!(
                bundle_id,
                Some("com.apple.finder".to_string()),
                "Should extract Finder bundle ID"
            );
        }
    }

    #[test]
    fn test_extract_bundle_id_nonexistent() {
        let fake_path = Path::new("/nonexistent/Fake.app");
        let bundle_id = extract_bundle_id(fake_path);
        assert!(
            bundle_id.is_none(),
            "Should return None for nonexistent app"
        );
    }

    #[test]
    fn test_parse_app_bundle() {
        let finder_path = Path::new("/System/Applications/Finder.app");
        if finder_path.exists() {
            let app_info = parse_app_bundle(finder_path);
            assert!(app_info.is_some(), "Should parse Finder.app");

            let app = app_info.unwrap();
            assert_eq!(app.name, "Finder");
            assert_eq!(app.path, finder_path);
            assert!(app.bundle_id.is_some());
        }
    }

    #[test]
    fn test_no_duplicate_apps() {
        let apps = scan_applications();
        // Use a set to check for true duplicates
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for app in apps.iter() {
            let lower_name = app.name.to_lowercase();
            if !seen.insert(lower_name.clone()) {
                duplicates.push(app.name.clone());
            }
        }

        // Allow a small number of duplicates (some systems have app variants)
        // e.g., same app name in different locations
        assert!(
            duplicates.len() <= 5,
            "Too many duplicate app names ({}): {:?}",
            duplicates.len(),
            duplicates
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_extract_app_icon() {
        // Test icon extraction for Calculator (always present on macOS)
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if calculator_path.exists() {
            let icon = extract_app_icon(calculator_path);
            assert!(icon.is_some(), "Should extract Calculator icon");

            if let Some(icon_data) = icon {
                // PNG magic bytes: 0x89 0x50 0x4E 0x47
                assert!(icon_data.len() > 8, "Icon data should have content");
                assert_eq!(
                    &icon_data[0..4],
                    &[0x89, 0x50, 0x4E, 0x47],
                    "Icon should be valid PNG"
                );
            }
        }
    }

    #[test]
    fn test_app_has_icon() {
        let apps = scan_applications();

        // Check that at least some apps have icons (most should)
        let apps_with_icons = apps.iter().filter(|a| a.icon.is_some()).count();

        // Most apps should have icons - at least 50%
        assert!(
            apps_with_icons > apps.len() / 2,
            "At least half of apps should have icons, got {}/{}",
            apps_with_icons,
            apps.len()
        );
    }

    // Note: launch_application is not tested automatically to avoid
    // actually launching apps during test runs. It can be tested manually.

    #[test]
    fn test_get_icon_cache_dir() {
        let cache_dir = get_icon_cache_dir();
        assert!(cache_dir.is_some(), "Should return a cache directory path");

        let dir = cache_dir.unwrap();
        assert!(
            dir.ends_with("cache/app-icons"),
            "Cache dir should end with cache/app-icons: {:?}",
            dir
        );
        assert!(
            dir.to_string_lossy().contains(".scriptkit"),
            "Cache dir should be under .scriptkit: {:?}",
            dir
        );
    }

    #[test]
    fn test_hash_path() {
        let path1 = Path::new("/Applications/Safari.app");
        let path2 = Path::new("/Applications/Safari.app");
        let path3 = Path::new("/Applications/Finder.app");

        // Same path should produce same hash
        assert_eq!(
            hash_path(path1),
            hash_path(path2),
            "Same path should produce same hash"
        );

        // Different paths should produce different hashes
        assert_ne!(
            hash_path(path1),
            hash_path(path3),
            "Different paths should produce different hashes"
        );

        // Hash should be 16 hex characters
        let hash = hash_path(path1);
        assert_eq!(hash.len(), 16, "Hash should be 16 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex characters: {}",
            hash
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_or_extract_icon_caches() {
        // Test that get_or_extract_icon properly caches icons
        let calculator_path = Path::new("/System/Applications/Calculator.app");
        if !calculator_path.exists() {
            return;
        }

        // First call - may or may not hit cache
        let icon1 = get_or_extract_icon(calculator_path);
        assert!(icon1.is_some(), "Should extract Calculator icon");

        // Second call should hit cache
        let icon2 = get_or_extract_icon(calculator_path);
        assert!(icon2.is_some(), "Should load Calculator icon from cache");

        // Both should have the same content
        let bytes1 = icon1.unwrap();
        let bytes2 = icon2.unwrap();
        assert_eq!(bytes1, bytes2, "Cached icon should match extracted icon");

        // Verify cache file exists
        let cache_dir = get_icon_cache_dir().unwrap();
        let cache_key = hash_path(calculator_path);
        let cache_file = cache_dir.join(format!("{}.png", cache_key));
        assert!(
            cache_file.exists(),
            "Cache file should exist: {:?}",
            cache_file
        );
    }

    #[test]
    fn test_decode_with_rb_swap() {
        use image::ImageEncoder;

        // Create a simple 2x2 PNG with known colors
        // Pixel at (0,0) = Red (255, 0, 0, 255)
        // Pixel at (1,0) = Blue (0, 0, 255, 255)
        // Pixel at (0,1) = Green (0, 255, 0, 255)
        // Pixel at (1,1) = White (255, 255, 255, 255)
        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255])); // Red
        img.put_pixel(1, 0, image::Rgba([0, 0, 255, 255])); // Blue
        img.put_pixel(0, 1, image::Rgba([0, 255, 0, 255])); // Green
        img.put_pixel(1, 1, image::Rgba([255, 255, 255, 255])); // White

        // Encode to PNG
        let mut original_png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut original_png);
        encoder
            .write_image(&img, 2, 2, image::ExtendedColorType::Rgba8)
            .expect("Failed to encode PNG");

        // Use the decode function with BGRA conversion
        let render_image =
            crate::list_item::decode_png_to_render_image_with_bgra_conversion(&original_png)
                .expect("Should decode with BGRA conversion");

        // Verify we got a RenderImage (we can't easily inspect pixels in RenderImage,
        // but we can verify it was created successfully)
        assert!(
            std::sync::Arc::strong_count(&render_image) >= 1,
            "Should create valid RenderImage"
        );
    }

    #[test]
    fn test_get_icon_cache_stats() {
        let (count, size) = get_icon_cache_stats();
        // We can't make strong assertions about exact counts since
        // other tests may have populated the cache, but we can check types
        assert!(
            count == 0 || size > 0,
            "If there are cached files, size should be non-zero"
        );
    }

    #[test]
    fn test_get_apps_db_path() {
        let db_path = get_apps_db_path();
        assert!(
            db_path.ends_with("db/apps.sqlite"),
            "DB path should end with db/apps.sqlite: {:?}",
            db_path
        );
        assert!(
            db_path.to_string_lossy().contains(".scriptkit"),
            "DB path should be under .scriptkit: {:?}",
            db_path
        );
    }

    #[test]
    fn test_loading_state() {
        // Initial state should be Ready (default)
        let state = get_app_loading_state();
        // Note: state may vary if other tests are running

        // Test message generation
        assert!(!state.message().is_empty(), "Should have a message");
    }

    #[test]
    fn test_get_apps_db_stats() {
        let (count, size) = get_apps_db_stats();
        // Stats should be valid - size is usize so always >= 0
        // Just verify the function returns without error
        let _ = (count, size);
    }
}
