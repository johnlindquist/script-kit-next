//! Asset path resolution for Script Kit GPUI

/// Get the path to a bundled asset that works both in development and in release builds.
///
/// In development (cargo run), assets are at `CARGO_MANIFEST_DIR/assets/`.
/// In release builds (.app bundle), assets are at `APP_BUNDLE/Contents/Resources/assets/`.
///
/// # Arguments
/// * `relative_path` - Path relative to the assets directory (e.g., "logo.svg" or "icons/check.svg")
///
/// # Returns
/// The full path to the asset as a String, suitable for use with GPUI's `svg().external_path()`.
pub fn get_asset_path(relative_path: &str) -> String {
    // First, try to find the asset in the app bundle (for release builds)
    #[cfg(target_os = "macos")]
    {
        if let Some(bundle_path) = get_macos_bundle_resources_path() {
            let asset_path = format!("{}/assets/{}", bundle_path, relative_path);
            if std::path::Path::new(&asset_path).exists() {
                return asset_path;
            }
        }
    }

    // Fall back to CARGO_MANIFEST_DIR for development builds
    // This is set at compile time, so it works when running via `cargo run`
    let dev_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/");
    format!("{}{}", dev_path, relative_path)
}

/// Get the macOS app bundle's Resources directory path.
/// Returns None if not running from an app bundle.
#[cfg(target_os = "macos")]
fn get_macos_bundle_resources_path() -> Option<String> {
    // Get the path to the current executable
    let exe_path = std::env::current_exe().ok()?;

    // Check if we're in an app bundle structure:
    // /path/to/App.app/Contents/MacOS/executable
    let exe_dir = exe_path.parent()?; // Contents/MacOS
    let contents_dir = exe_dir.parent()?; // Contents

    // Verify this looks like a bundle
    if contents_dir.file_name()?.to_str()? != "Contents" {
        return None;
    }

    let resources_dir = contents_dir.join("Resources");
    if resources_dir.exists() {
        return resources_dir.to_str().map(|s| s.to_string());
    }

    None
}

/// Convenience function to get the logo.svg path
pub fn get_logo_path() -> String {
    get_asset_path("logo.svg")
}
