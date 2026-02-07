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
