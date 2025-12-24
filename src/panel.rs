// macOS Panel Configuration Module
// This module configures GPUI windows as macOS floating panels
// that appear above other applications

#[cfg(target_os = "macos")]
/// Configure the current key window as a floating panel window that appears above other apps.
///
/// This function:
/// - Sets the window level to NSFloatingWindowLevel (3) so it floats above normal windows
/// - Sets collection behavior to appear on all spaces/desktops
/// - Keeps the window visible when switching between applications
///
/// Should be called immediately after the window is created and visible.
pub fn configure_as_floating_panel() {
    // This will be called from main.rs where objc macros are available
    // The actual implementation is in main.rs to avoid macro issues in lib code
    crate::logging::log("PANEL", "Panel configuration (implemented in main.rs)");
}

#[cfg(not(target_os = "macos"))]
/// No-op on non-macOS platforms
pub fn configure_as_floating_panel() {}
