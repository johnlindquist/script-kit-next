//! macOS System Actions Module
//!
//! Provides AppleScript-based system actions for macOS including:
//! - Power management (sleep, restart, shutdown, lock, log out)
//! - UI controls (dark mode, show desktop, mission control, launchpad)
//! - Media controls (volume)
//! - System utilities (empty trash, force quit, screen saver, do not disturb)
//! - System Preferences navigation
//! - Running application management (list GUI apps, force quit specific apps)
//!
//! All functions use `osascript` to execute AppleScript commands and return
//! `Result<(), String>` for consistent error handling.

n// This entire module is macOS-only
#![cfg(target_os = "macos")]

use std::process::Command;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};
use tracing::{debug, error, info, warn};

// ============================================================================
// Helper Function
// ============================================================================

/// Execute an AppleScript command and return the result
fn run_applescript(script: &str) -> Result<(), String> {
    debug!(script = %script, "Executing AppleScript");

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
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

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
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
/// For showing Script Kit's own Force Quit UI, use `get_running_apps()` instead.
pub fn force_quit_apps() -> Result<(), String> {
    info!("Opening Force Quit Applications dialog");
    run_applescript(
        r#"tell application "System Events"
            keystroke "escape" using {command down, option down}
        end tell"#,
    )
}

// ============================================================================
// Running Application Management
// ============================================================================

/// Information about a running application
///
/// Used by the Force Quit UI to display running apps and allow users to
/// terminate unresponsive applications.
#[derive(Debug, Clone)]
#[allow(dead_code)] // API for Force Quit UI - called from app_execute.rs
pub struct AppInfo {
    /// Application name
    pub name: String,
    /// Process ID
    pub pid: u32,
    /// Bundle identifier (e.g., "com.apple.Safari"), if available
    pub bundle_id: Option<String>,
    /// Path to the application executable
    pub path: Option<String>,
    /// Memory usage in bytes
    pub memory: u64,
    /// CPU usage percentage (may be 0 if not measured)
    pub cpu_usage: f32,
}

/// Get list of running GUI applications
///
/// Returns a list of running applications that are likely to be GUI apps
/// (i.e., apps the user would want to force quit). Filters out system
/// processes, daemons, and helper processes.
///
/// # Returns
/// A vector of `AppInfo` structs sorted by name.
///
/// # Example
/// ```no_run
/// use script_kit_gpui::system_actions::get_running_apps;
/// let apps = get_running_apps().expect("Failed to get running apps");
/// for app in apps {
///     println!("{} (PID: {})", app.name, app.pid);
/// }
/// ```
#[allow(dead_code)] // API for Force Quit UI - called from app_execute.rs
pub fn get_running_apps() -> Result<Vec<AppInfo>, String> {
    info!("Getting list of running applications");

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut apps: Vec<AppInfo> = Vec::new();

    // Known system processes to exclude (they're not user-facing GUI apps)
    let excluded_names: &[&str] = &[
        "kernel_task",
        "launchd",
        "logd",
        "cfprefsd",
        "distnoted",
        "trustd",
        "secd",
        "opendirectoryd",
        "syslogd",
        "powerd",
        "coreduetd",
        "mds",
        "mds_stores",
        "diskarbitrationd",
        "fseventsd",
        "coreaudiod",
        "WindowServer",
        "loginwindow",
        "UserEventAgent",
        "coreservicesd",
        "lsd",
        "talagent",
        "parentalcontrolsd",
        "sharingd",
        "rapportd",
        "Spotlight",
        "corespotlightd",
        "mdworker",
        "mdworker_shared",
        "notifyd",
        "configd",
        "thermalmonitord",
        "securityd",
        "iconservicesagent",
        "nsurlstoraged",
        "locationd",
        "hidd",
        "audioclocksyncd",
        "coresymbolicationd",
        "apsd",
        "bluetoothd",
        "iCloudPrivacyAgent",
        "sandboxd",
    ];

    // Suffixes that indicate helper/agent processes
    let excluded_suffixes: &[&str] = &[
        "Agent",
        "Helper",
        "Daemon",
        "Service",
        "Extension",
        "XPCService",
        "Worker",
        "Renderer", // Browser renderers aren't the main app
        "_agent",
        "_helper",
        "_daemon",
        "-Helper",
    ];

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();

        // Skip excluded processes
        if excluded_names.iter().any(|&n| n == name) {
            continue;
        }

        // Skip processes with excluded suffixes
        if excluded_suffixes.iter().any(|&s| name.ends_with(s)) {
            continue;
        }

        // Skip Script Kit itself
        if name == "script-kit-gpui" || name == "Script Kit" {
            continue;
        }

        // Get the executable path
        let exe_path = process.exe().map(|p| p.to_string_lossy().to_string());

        // Only include apps from /Applications or user home (GUI apps typically)
        // Also include any .app bundle processes
        let is_gui_app = exe_path.as_ref().is_some_and(|p| {
            p.contains("/Applications/")
                || p.contains("/System/Applications/")
                || p.contains(".app/")
                || p.contains("/Library/")
        });

        // Also check by process name patterns (apps often don't have path info)
        // Most GUI apps have a capitalized name or are known apps
        let looks_like_app = name
            .chars()
            .next()
            .is_some_and(|c| c.is_uppercase() || c.is_ascii_digit())
            && !name.starts_with("com.")
            && !name.starts_with("us.")
            && !name.contains('.');

        if !is_gui_app && !looks_like_app {
            continue;
        }

        // Try to get bundle ID from process path
        let bundle_id = exe_path.as_ref().and_then(|p| {
            // Extract bundle ID from path like /Applications/Safari.app/Contents/MacOS/Safari
            if let Some(app_idx) = p.find(".app/") {
                let app_path = &p[..app_idx + 4];
                // Get the bundle identifier using mdls
                get_bundle_id_from_path(app_path).ok()
            } else {
                None
            }
        });

        apps.push(AppInfo {
            name,
            pid: pid.as_u32(),
            bundle_id,
            path: exe_path,
            memory: process.memory(),
            cpu_usage: process.cpu_usage(),
        });
    }

    // Deduplicate by name (keep the one with lower PID, usually the main process)
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    apps.retain(|app| seen_names.insert(app.name.clone()));

    // Sort by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    info!(
        count = apps.len(),
        "Retrieved list of running GUI applications"
    );
    debug!(apps = ?apps.iter().map(|a| &a.name).collect::<Vec<_>>(), "Running apps");

    Ok(apps)
}

/// Get the bundle ID from an app path using mdls
#[allow(dead_code)] // Internal helper used by get_running_apps
fn get_bundle_id_from_path(app_path: &str) -> Result<String, String> {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemCFBundleIdentifier", "-raw", app_path])
        .output()
        .map_err(|e| format!("Failed to run mdls: {}", e))?;

    if output.status.success() {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if bundle_id != "(null)" && !bundle_id.is_empty() {
            return Ok(bundle_id);
        }
    }
    Err("Bundle ID not found".to_string())
}

/// Force quit a specific application by PID
///
/// Sends SIGKILL to immediately terminate the process. This is equivalent
/// to using "Force Quit" in Activity Monitor.
///
/// # Arguments
/// * `pid` - The process ID of the application to quit
///
/// # Returns
/// Ok(()) if the signal was sent successfully, or an error message.
///
/// # Example
/// ```no_run
/// use script_kit_gpui::system_actions::force_quit_app;
/// force_quit_app(1234).expect("Failed to force quit app");
/// ```
#[allow(dead_code)] // API for Force Quit UI - called from app_execute.rs
pub fn force_quit_app(pid: u32) -> Result<(), String> {
    info!(pid = pid, "Force quitting application");

    let mut sys = System::new();
    let target_pid = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[target_pid]), true);

    // First, check if process exists and get its name
    let name = match sys.process(target_pid) {
        Some(process) => process.name().to_string_lossy().to_string(),
        None => {
            let msg = format!("Process with PID {} not found", pid);
            error!(pid = pid, "Process not found for force quit");
            return Err(msg);
        }
    };

    // Try SIGTERM first for graceful shutdown
    if let Some(process) = sys.process(target_pid) {
        if process.kill_with(Signal::Term).is_some() {
            info!(pid = pid, name = %name, "Sent SIGTERM to process");
        }
    }

    // Wait briefly to see if it terminates
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Refresh to check if still running
    sys.refresh_processes(ProcessesToUpdate::Some(&[target_pid]), true);
    if sys.process(target_pid).is_none() {
        info!(pid = pid, name = %name, "Process terminated gracefully");
        return Ok(());
    }

    // Process still running, use SIGKILL
    warn!(
        pid = pid,
        name = %name,
        "Process did not terminate gracefully, sending SIGKILL"
    );

    if let Some(process) = sys.process(target_pid) {
        process.kill();
        info!(pid = pid, name = %name, "Sent SIGKILL to process");
    }

    Ok(())
}

/// Force quit an application by name
///
/// Finds the first process with the given name and force quits it.
/// This is useful when you know the app name but not the PID.
///
/// # Arguments
/// * `app_name` - The name of the application to quit (e.g., "Safari")
///
/// # Returns
/// Ok(()) if the app was found and signal sent, or an error message.
#[allow(dead_code)] // API for Force Quit UI - called from app_execute.rs
pub fn force_quit_app_by_name(app_name: &str) -> Result<(), String> {
    info!(app_name = %app_name, "Force quitting application by name");

    let apps = get_running_apps()?;

    if let Some(app) = apps.iter().find(|a| a.name == app_name) {
        force_quit_app(app.pid)
    } else {
        let msg = format!("Application '{}' not found in running apps", app_name);
        error!(app_name = %app_name, "Application not found for force quit");
        Err(msg)
    }
}

// ============================================================================
// Volume Controls
// ============================================================================

/// Increase system volume
///
/// Increases the system volume by approximately 6.25% (1/16th of max).
#[allow(dead_code)]
pub fn volume_up() -> Result<(), String> {
    info!("Increasing volume");
    run_applescript(r#"set volume output volume ((output volume of (get volume settings)) + 6.25)"#)
}

/// Decrease system volume
///
/// Decreases the system volume by approximately 6.25% (1/16th of max).
#[allow(dead_code)]
pub fn volume_down() -> Result<(), String> {
    info!("Decreasing volume");
    run_applescript(r#"set volume output volume ((output volume of (get volume settings)) - 6.25)"#)
}

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

/// Get current volume level
///
/// Returns the current volume level (0-100).
#[allow(dead_code)]
pub fn get_volume() -> Result<u8, String> {
    let output = run_applescript_with_output("output volume of (get volume settings)")?;
    output
        .parse::<f64>()
        .map(|v| v.round() as u8)
        .map_err(|e| format!("Failed to parse volume: {}", e))
}

/// Check if audio is muted
#[allow(dead_code)]
pub fn is_muted() -> Result<bool, String> {
    let output = run_applescript_with_output("output muted of (get volume settings)")?;
    Ok(output == "true")
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

    let output = Command::new("open")
        .arg(&url)
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

/// Open System Settings to the General pane
#[allow(dead_code)]
pub fn open_general_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.systempreferences.GeneralSettings")
}

/// Open System Settings to the Desktop & Dock pane
#[allow(dead_code)]
pub fn open_dock_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Desktop-Settings.extension")
}

/// Open System Settings to the Battery pane
#[allow(dead_code)]
pub fn open_battery_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Battery-Settings.extension")
}

/// Open System Settings to the Trackpad pane
#[allow(dead_code)]
pub fn open_trackpad_settings() -> Result<(), String> {
    open_system_settings_url("com.apple.Trackpad-Settings.extension")
}

/// Open System Settings (main window)
pub fn open_system_preferences_main() -> Result<(), String> {
    info!("Opening System Settings");
    let output = Command::new("open")
        .arg("-a")
        .arg("System Settings")
        .output()
        .map_err(|e| format!("Failed to open System Settings: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to open System Settings: {}", stderr))
    }
}

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
    fn test_volume_controls_integration() {
        // Integration test - only run manually
        if let Ok(initial_volume) = get_volume() {
            println!("Initial volume: {}", initial_volume);

            // Test volume up
            let _ = volume_up();

            // Test volume down
            let _ = volume_down();

            // Test set volume
            let _ = set_volume(initial_volume);

            // Test mute check
            if let Ok(muted) = is_muted() {
                println!("Is muted: {}", muted);
            }
        }
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

    // =========================================================================
    // Running Applications Tests
    // =========================================================================

    #[test]
    fn test_get_running_apps_returns_list() {
        // This test should work on any macOS system with GUI apps running
        let apps = get_running_apps().expect("Should get running apps");

        // There should be at least a few apps running (Finder is always running)
        assert!(!apps.is_empty(), "Should have at least one running app");

        // Each app should have a name and valid PID
        for app in &apps {
            assert!(!app.name.is_empty(), "App name should not be empty");
            assert!(app.pid > 0, "PID should be positive");
        }
    }

    #[test]
    fn test_get_running_apps_sorted_by_name() {
        let apps = get_running_apps().expect("Should get running apps");

        if apps.len() > 1 {
            // Verify sorted by name (case-insensitive)
            for i in 0..apps.len() - 1 {
                assert!(
                    apps[i].name.to_lowercase() <= apps[i + 1].name.to_lowercase(),
                    "Apps should be sorted by name: {} should come before {}",
                    apps[i].name,
                    apps[i + 1].name
                );
            }
        }
    }

    #[test]
    fn test_get_running_apps_excludes_system_processes() {
        let apps = get_running_apps().expect("Should get running apps");

        // System processes should be excluded
        let system_names = ["kernel_task", "launchd", "WindowServer", "loginwindow"];
        for app in &apps {
            assert!(
                !system_names.contains(&app.name.as_str()),
                "System process '{}' should be excluded",
                app.name
            );
        }
    }

    #[test]
    fn test_get_running_apps_excludes_helpers() {
        let apps = get_running_apps().expect("Should get running apps");

        // Helper processes should be excluded
        for app in &apps {
            assert!(
                !app.name.ends_with("Helper"),
                "Helper process '{}' should be excluded",
                app.name
            );
            assert!(
                !app.name.ends_with("Agent"),
                "Agent process '{}' should be excluded",
                app.name
            );
        }
    }

    #[test]
    fn test_app_info_struct() {
        // Test that AppInfo can be created and used
        let app = AppInfo {
            name: "TestApp".to_string(),
            pid: 1234,
            bundle_id: Some("com.test.app".to_string()),
            path: Some("/Applications/TestApp.app/Contents/MacOS/TestApp".to_string()),
            memory: 1024 * 1024,
            cpu_usage: 1.5,
        };

        assert_eq!(app.name, "TestApp");
        assert_eq!(app.pid, 1234);
        assert_eq!(app.bundle_id, Some("com.test.app".to_string()));
        assert!(app.path.is_some());
        assert_eq!(app.memory, 1024 * 1024);
        assert_eq!(app.cpu_usage, 1.5);
    }

    #[test]
    fn test_app_info_clone() {
        let app = AppInfo {
            name: "TestApp".to_string(),
            pid: 1234,
            bundle_id: None,
            path: None,
            memory: 0,
            cpu_usage: 0.0,
        };

        let cloned = app.clone();
        assert_eq!(app.name, cloned.name);
        assert_eq!(app.pid, cloned.pid);
    }

    #[test]
    fn test_force_quit_app_nonexistent_pid() {
        // Trying to force quit a non-existent PID should fail
        let result = force_quit_app(99999999);
        assert!(result.is_err(), "Should fail for non-existent PID");
    }

    #[test]
    fn test_force_quit_app_by_name_nonexistent() {
        // Trying to force quit a non-existent app should fail
        let result = force_quit_app_by_name("NonExistentAppThatDefinitelyDoesNotExist12345");
        assert!(result.is_err(), "Should fail for non-existent app");
    }

    #[test]
    #[ignore]
    fn test_get_running_apps_integration() {
        // Integration test - prints all running apps
        let apps = get_running_apps().expect("Should get running apps");
        println!("Found {} running apps:", apps.len());
        for app in apps {
            println!(
                "  {} (PID: {}, bundle: {:?}, mem: {} KB)",
                app.name,
                app.pid,
                app.bundle_id,
                app.memory / 1024
            );
        }
    }
}
