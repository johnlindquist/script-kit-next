use anyhow::{Context, Result};
use rayon::prelude::*;
use rusqlite::{params, Connection};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::time::Instant;
use tracing::{debug, error, info, info_span, trace, warn};

/// Stats for icon extraction during a scan (thread-safe)
static ICONS_EXTRACTED: AtomicUsize = AtomicUsize::new(0);
static ICONS_FROM_CACHE: AtomicUsize = AtomicUsize::new(0);
static EXTRACT_TIME_MS: AtomicUsize = AtomicUsize::new(0);

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
static APP_CACHE: LazyLock<Arc<Mutex<Vec<AppInfo>>>> = LazyLock::new(|| {
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

/// Current loading state (thread-safe, updated during scan)
static APP_LOADING_STATE: LazyLock<Mutex<AppLoadingState>> =
    LazyLock::new(|| Mutex::new(AppLoadingState::LoadingFromCache));

/// Database connection for apps cache
static APPS_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Directories to scan for .app bundles
const APP_DIRECTORIES: &[&str] = &[
    // Standard macOS app locations
    "/Applications",
    "/System/Applications",
    "/System/Applications/Utilities",
    "/Applications/Utilities",
    // System utilities (Keychain Access, Screen Sharing, etc.)
    "/System/Library/CoreServices/Applications",
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

