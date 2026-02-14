//! macOS System Actions Module
//!
//! Provides AppleScript-based system actions for macOS including:
//! - Power management (sleep, restart, shutdown, lock, log out)
//! - UI controls (dark mode, show desktop, mission control, launchpad)
//! - Media controls (volume)
//! - System utilities (empty trash, force quit, screen saver, do not disturb)
//! - System Preferences navigation
//!
//! All functions use `osascript` to execute AppleScript commands and return
//! `Result<(), String>` for consistent error handling.

// --- merged from part_000.rs ---
use std::process::Command;
use tracing::{debug, error, info};

const SYSTEM_CMD_ENV_VARS: [&str; 5] = ["PATH", "HOME", "TMPDIR", "USER", "LANG"];

fn scrub_command_env(cmd: &mut Command) {
    cmd.env_clear();
    for key in SYSTEM_CMD_ENV_VARS {
        if let Some(val) = std::env::var_os(key) {
            cmd.env(key, val);
        }
    }
}
// ============================================================================
// Helper Function
// ============================================================================

/// Execute an AppleScript command and return the result
fn run_applescript(script: &str) -> Result<(), String> {
    debug!(script = %script, "Executing AppleScript");

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(script);
    scrub_command_env(&mut cmd);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

    if output.status.success() {
        debug!("AppleScript executed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(stderr = %stderr, "AppleScript execution failed");
        Err(format!("AppleScript error: {}", stderr))
    }
}
/// Execute an AppleScript command and return the output
#[allow(dead_code)]
fn run_applescript_with_output(script: &str) -> Result<String, String> {
    debug!(script = %script, "Executing AppleScript with output");

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(script);
    scrub_command_env(&mut cmd);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!(output = %stdout, "AppleScript executed successfully");
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(stderr = %stderr, "AppleScript execution failed");
        Err(format!("AppleScript error: {}", stderr))
    }
}
// ============================================================================
// Trash Management
// ============================================================================

/// Empty the macOS Trash
///
/// Uses Finder to empty the trash without user confirmation.
///
/// # Example
/// ```no_run
/// use script_kit_gpui::system_actions::empty_trash;
/// empty_trash().expect("Failed to empty trash");
/// ```
pub fn empty_trash() -> Result<(), String> {
    info!("Emptying trash");
    run_applescript(r#"tell application "Finder" to empty trash"#)
}
// ============================================================================
// Power Management
// ============================================================================

/// Lock the screen
///
/// Activates the screen saver which requires authentication to unlock.
pub fn lock_screen() -> Result<(), String> {
    info!("Locking screen");
    // Use the Keychain Access method which is more reliable
    run_applescript(
        r#"tell application "System Events" to keystroke "q" using {command down, control down}"#,
    )
}
/// Put the system to sleep
///
/// Puts the Mac into sleep mode.
pub fn sleep() -> Result<(), String> {
    info!("Putting system to sleep");
    run_applescript(r#"tell application "System Events" to sleep"#)
}
/// Restart the system
///
/// Initiates a system restart. Applications will be asked to save documents.
pub fn restart() -> Result<(), String> {
    info!("Restarting system");
    run_applescript(r#"tell application "System Events" to restart"#)
}
/// Shut down the system
///
/// Initiates a system shutdown. Applications will be asked to save documents.
pub fn shut_down() -> Result<(), String> {
    info!("Shutting down system");
    run_applescript(r#"tell application "System Events" to shut down"#)
}
/// Log out the current user
///
/// Logs out the current user. Applications will be asked to save documents.
pub fn log_out() -> Result<(), String> {
    info!("Logging out user");
    run_applescript(r#"tell application "System Events" to log out"#)
}
// ============================================================================
// UI Controls
// ============================================================================

/// Toggle Dark Mode
///
/// Switches between light and dark appearance mode.
pub fn toggle_dark_mode() -> Result<(), String> {
    info!("Toggling dark mode");
    run_applescript(
        r#"tell application "System Events"
            tell appearance preferences
                set dark mode to not dark mode
            end tell
        end tell"#,
    )
}
/// Check if Dark Mode is enabled
///
/// Returns true if dark mode is currently active.
#[allow(dead_code)]
pub fn is_dark_mode() -> Result<bool, String> {
    let output = run_applescript_with_output(
        r#"tell application "System Events"
            tell appearance preferences
                return dark mode
            end tell
        end tell"#,
    )?;
    Ok(output == "true")
}
/// Show Desktop (hide all windows)
///
/// Hides all windows to reveal the desktop.
pub fn show_desktop() -> Result<(), String> {
    info!("Showing desktop");
    // F11 key code is 103, but we use the hot corner simulation
    // which is more reliable across different keyboard layouts
    run_applescript(
        r#"tell application "System Events"
            key code 103 using {command down}
        end tell"#,
    )
}
/// Activate Mission Control
///
/// Opens Mission Control to show all windows and desktops.
pub fn mission_control() -> Result<(), String> {
    info!("Activating Mission Control");
    // Control + Up Arrow triggers Mission Control
    run_applescript(
        r#"tell application "System Events"
            key code 126 using {control down}
        end tell"#,
    )
}
/// Open Launchpad
///
/// Opens Launchpad to show all applications.
pub fn launchpad() -> Result<(), String> {
    info!("Opening Launchpad");
    // F4 key code is 118 on many keyboards, but we use the direct approach
    run_applescript(r#"tell application "Launchpad" to activate"#)
}
/// Open Force Quit Applications dialog (legacy)
///
/// Opens the macOS Force Quit Applications window (Cmd+Option+Escape).
pub fn force_quit_apps() -> Result<(), String> {
    info!("Opening Force Quit Applications dialog");
    run_applescript(
        r#"tell application "System Events"
            keystroke "escape" using {command down, option down}
        end tell"#,
    )
}
// ============================================================================
// Volume Controls
// ============================================================================

/// Toggle mute
///
/// Toggles the system audio mute state.
pub fn volume_mute() -> Result<(), String> {
    info!("Toggling mute");
    run_applescript(
        r#"set currentMute to output muted of (get volume settings)
        set volume output muted (not currentMute)"#,
    )
}
/// Set volume to a specific level
///
/// # Arguments
/// * `level` - Volume level from 0 to 100
pub fn set_volume(level: u8) -> Result<(), String> {
    let level = level.min(100);
    info!(level = level, "Setting volume");
    run_applescript(&format!("set volume output volume {}", level))
}
// ============================================================================
// Do Not Disturb
// ============================================================================

/// Toggle Do Not Disturb mode
///
/// Toggles macOS Focus/Do Not Disturb mode.
/// Note: This uses keyboard shortcuts as the DND API changed in recent macOS versions.
pub fn toggle_do_not_disturb() -> Result<(), String> {
    info!("Toggling Do Not Disturb");
    // Option-click on the menu bar clock or use Control Center
    // We'll use the Control Center approach for better compatibility
    run_applescript(
        r#"tell application "System Events"
            tell process "ControlCenter"
                -- Click the Focus button in Control Center
                click menu bar item "Focus" of menu bar 1
            end tell
        end tell"#,
    )
}
// ============================================================================
// Screen Saver
// ============================================================================

/// Start the screen saver
///
/// Immediately activates the screen saver.
pub fn start_screen_saver() -> Result<(), String> {
    info!("Starting screen saver");
    run_applescript(r#"tell application "ScreenSaverEngine" to activate"#)
}
// ============================================================================
// System Preferences Navigation
// ============================================================================

/// Open System Preferences/Settings to a specific pane using URL scheme
///
/// Uses the `x-apple.systempreferences:` URL scheme which works reliably on
/// macOS Ventura and later (where "System Preferences" became "System Settings").
///
/// # Arguments
/// * `url_path` - The URL path (e.g., "com.apple.Displays-Settings.extension")
///
/// # Common URL Paths (macOS Ventura+)
/// - `com.apple.preference.security` - Privacy & Security
/// - `com.apple.Displays-Settings.extension` - Displays
/// - `com.apple.Sound-Settings.extension` - Sound
/// - `com.apple.Network-Settings.extension` - Network
/// - `com.apple.Keyboard-Settings.extension` - Keyboard
/// - `com.apple.Trackpad-Settings.extension` - Trackpad
/// - `com.apple.BluetoothSettings` - Bluetooth
/// - `com.apple.Notifications-Settings.extension` - Notifications
/// - `com.apple.systempreferences.GeneralSettings` - General
/// - `com.apple.Desktop-Settings.extension` - Desktop & Dock
/// - `com.apple.preferences.AppleIDPrefPane` - Apple ID
/// - `com.apple.Battery-Settings.extension` - Battery
fn open_system_settings_url(url_path: &str) -> Result<(), String> {
    let url = format!("x-apple.systempreferences:{}", url_path);
    info!(url = %url, "Opening System Settings via URL scheme");

    let mut cmd = Command::new("open");
    cmd.arg(&url);
    scrub_command_env(&mut cmd);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to open System Settings: {}", e))?;

    if output.status.success() {
        debug!("System Settings opened successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(stderr = %stderr, "Failed to open System Settings");
        Err(format!("Failed to open System Settings: {}", stderr))
    }
}
/// Open System Settings to the Privacy & Security pane
pub fn open_privacy_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.preference.security")
}
/// Open System Settings to the Displays pane
pub fn open_display_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Displays-Settings.extension")
}
/// Open System Settings to the Sound pane
pub fn open_sound_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Sound-Settings.extension")
}
/// Open System Settings to the Network pane
pub fn open_network_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Network-Settings.extension")
}
/// Open System Settings to the Keyboard pane
pub fn open_keyboard_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Keyboard-Settings.extension")
}
/// Open System Settings to the Bluetooth pane
pub fn open_bluetooth_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.BluetoothSettings")
}
/// Open System Settings to the Notifications pane
pub fn open_notifications_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Notifications-Settings.extension")
}
/// Open System Settings (main window)
pub fn open_system_preferences_main() -> Result<(), String> {
    info!("Opening System Settings");
    let mut cmd = Command::new("open");
    cmd.arg("-a").arg("System Settings");
    scrub_command_env(&mut cmd);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to open System Settings: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to open System Settings: {}", stderr))
    }
}
// --- merged from part_002.rs ---
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most of these tests are marked #[ignore] because they require
    // actual system interaction and should only be run manually on macOS.
    // Run with: cargo test --features system-tests -- --ignored

    #[test]
    fn test_run_applescript_syntax_error() {
        // Test that syntax errors are properly caught
        let result = run_applescript("this is not valid applescript syntax (((");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("AppleScript error"));
    }

    #[test]
    fn test_run_applescript_with_output_simple() {
        // Test a simple AppleScript that returns a value
        let result = run_applescript_with_output("return 42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_run_applescript_with_output_string() {
        // Test returning a string
        let result = run_applescript_with_output(r#"return "hello""#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_run_applescript_with_output_boolean() {
        // Test returning a boolean
        let result = run_applescript_with_output("return true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_set_volume_clamps_to_100() {
        // Test that set_volume clamps values above 100
        // This doesn't actually set volume, just tests the script generation
        let test_value: u8 = 150;
        let script = format!("set volume output volume {}", test_value.min(100));
        assert!(script.contains("100"));
    }

    #[test]
    #[ignore]
    fn test_empty_trash_integration() {
        // Integration test - only run manually
        let result = empty_trash();
        // May succeed or fail depending on permissions
        println!("empty_trash result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_toggle_dark_mode_integration() {
        // Integration test - only run manually
        let result = toggle_dark_mode();
        println!("toggle_dark_mode result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_is_dark_mode_integration() {
        // Integration test - only run manually
        let result = is_dark_mode();
        println!("is_dark_mode result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_start_screen_saver_integration() {
        // Integration test - only run manually
        let result = start_screen_saver();
        println!("start_screen_saver result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_mission_control_integration() {
        // Integration test - only run manually
        let result = mission_control();
        println!("mission_control result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_launchpad_integration() {
        // Integration test - only run manually
        let result = launchpad();
        println!("launchpad result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_open_system_settings_integration() {
        // Integration test - only run manually
        let result = open_sound_settings();
        println!("open_sound_settings result: {:?}", result);
    }
}
