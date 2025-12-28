//! App Launcher Module
//!
//! Provides functionality to scan and launch macOS applications.
//!
//! ## Features
//! - Scans standard macOS application directories
//! - Caches results for performance (apps don't change often)
//! - Extracts bundle identifiers from Info.plist when available
//! - Extracts app icons using NSWorkspace for display
//! - Launches applications via `open -a`
//!
//! ## Usage
//! ```ignore
//! use crate::app_launcher::{scan_applications, launch_application, AppInfo};
//!
//! // Get all installed applications (cached after first call)
//! let apps = scan_applications();
//!
//! // Launch an application
//! if let Some(app) = apps.iter().find(|a| a.name == "Finder") {
//!     launch_application(app)?;
//! }
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{debug, info, warn};

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

/// Cached list of applications (scanned once, reused)
static APP_CACHE: OnceLock<Vec<AppInfo>> = OnceLock::new();

/// Directories to scan for .app bundles
const APP_DIRECTORIES: &[&str] = &[
    "/Applications",
    "/System/Applications",
    "~/Applications",
    "/Applications/Utilities",
];

/// Scan for installed macOS applications
///
/// This function scans standard macOS application directories and returns
/// a list of all found .app bundles. Results are cached after the first call
/// for performance (applications don't change frequently).
///
/// # Returns
/// A reference to the cached vector of AppInfo structs.
///
/// # Performance
/// Initial scan may take ~100ms depending on the number of installed apps.
/// Subsequent calls return immediately from cache.
pub fn scan_applications() -> &'static Vec<AppInfo> {
    APP_CACHE.get_or_init(|| {
        let start = Instant::now();
        let apps = scan_all_directories();
        let duration_ms = start.elapsed().as_millis();

        info!(
            app_count = apps.len(),
            duration_ms = duration_ms,
            "Scanned applications"
        );

        apps
    })
}

/// Force a fresh scan of applications (bypasses cache)
///
/// This is useful if you need to detect newly installed applications.
/// Note: This does NOT update the static cache - it just returns fresh results.
#[allow(dead_code)]
pub fn scan_applications_fresh() -> Vec<AppInfo> {
    let start = Instant::now();
    let apps = scan_all_directories();
    let duration_ms = start.elapsed().as_millis();

    info!(
        app_count = apps.len(),
        duration_ms = duration_ms,
        "Fresh scan of applications"
    );

    apps
}

/// Scan all configured directories for applications
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

/// Scan a single directory for .app bundles
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

/// Parse a .app bundle to extract application information
fn parse_app_bundle(path: &Path) -> Option<AppInfo> {
    // Extract app name from bundle name (strip .app extension)
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;

    // Try to extract bundle identifier from Info.plist
    let bundle_id = extract_bundle_id(path);

    // Extract and pre-decode icon using NSWorkspace (macOS only)
    // Pre-decoding here is CRITICAL for performance - avoids PNG decode on every render
    #[cfg(target_os = "macos")]
    let icon = extract_app_icon(path).and_then(|png_bytes| {
        crate::list_item::decode_png_to_render_image(&png_bytes).ok()
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
/// # Example
/// ```ignore
/// let apps = scan_applications();
/// if let Some(finder) = apps.iter().find(|a| a.name == "Finder") {
///     launch_application(finder)?;
/// }
/// ```
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
            assert!(calculator.bundle_id.is_some(), "Calculator should have a bundle ID");
            assert_eq!(
                calculator.bundle_id.as_deref(),
                Some("com.apple.calculator"),
                "Calculator bundle ID should be com.apple.calculator"
            );
        }
    }

    #[test]
    fn test_scan_applications_cached() {
        // First call populates cache
        let apps1 = scan_applications();

        // Second call should return same reference (cached)
        let apps2 = scan_applications();

        // Both should point to the same data
        assert_eq!(apps1.len(), apps2.len());
        assert!(std::ptr::eq(apps1, apps2), "Should return cached reference");
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
        let mut names: Vec<_> = apps.iter().map(|a| a.name.to_lowercase()).collect();
        let original_len = names.len();
        names.dedup();

        assert_eq!(
            original_len,
            names.len(),
            "Should not have duplicate app names"
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
}
