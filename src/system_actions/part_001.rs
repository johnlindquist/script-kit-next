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
