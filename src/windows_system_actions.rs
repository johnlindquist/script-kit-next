//! Windows system actions — equivalents to macOS `system_actions` module.
//!
//! Uses Win32 APIs and shell commands to implement system-level actions
//! (lock screen, sleep, restart, shutdown, volume control, settings, etc.)
//!
//! All functions return `Result<(), String>` matching the macOS API contract.

#![cfg(target_os = "windows")]

use std::process::Command;

// ── Win32 FFI declarations ─────────────────────────────────────────

#[allow(non_snake_case)]
mod ffi {
    // user32.dll
    extern "system" {
        pub fn LockWorkStation() -> i32;
        pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
    }

    // PowrProf.dll
    #[link(name = "PowrProf")]
    extern "system" {
        pub fn SetSuspendState(bHibernate: i32, bForce: i32, bWakeupEventsDisabled: i32) -> i32;
    }

    // shell32.dll
    extern "system" {
        pub fn SHEmptyRecycleBinW(
            hwnd: *mut std::ffi::c_void,
            pszRootPath: *const u16,
            dwFlags: u32,
        ) -> i32;
    }

    // Constants
    pub const KEYEVENTF_KEYUP: u32 = 0x0002;
    pub const VK_LWIN: u8 = 0x5B;
    pub const VK_D: u8 = 0x44;

    // SHEmptyRecycleBin flags
    pub const SHERB_NOCONFIRMATION: u32 = 0x0000_0001;
    pub const SHERB_NOPROGRESSUI: u32 = 0x0000_0002;
    pub const SHERB_NOSOUND: u32 = 0x0000_0004;
}

// ── Helpers ────────────────────────────────────────────────────────

fn cmd_err(action: &str, e: std::io::Error) -> String {
    format!("{}: {}", action, e)
}

fn cmd_fail(action: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{} failed: {}", action, stderr.trim())
}

// ── Power management ───────────────────────────────────────────────

/// Lock the workstation (Win+L equivalent).
pub fn lock_screen() -> Result<(), String> {
    let result = unsafe { ffi::LockWorkStation() };
    if result == 0 {
        return Err(format!(
            "LockWorkStation failed (error {})",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

/// Put the system to sleep.
pub fn sleep() -> Result<(), String> {
    // SetSuspendState(hibernate=false, force=false, wakeup_events=false)
    let result = unsafe { ffi::SetSuspendState(0, 0, 0) };
    if result == 0 {
        return Err(format!(
            "SetSuspendState failed (error {})",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

/// Restart the system.
pub fn restart() -> Result<(), String> {
    Command::new("shutdown")
        .args(["/r", "/t", "0"])
        .spawn()
        .map_err(|e| cmd_err("shutdown /r", e))?;
    Ok(())
}

/// Shut down the system.
pub fn shut_down() -> Result<(), String> {
    Command::new("shutdown")
        .args(["/s", "/t", "0"])
        .spawn()
        .map_err(|e| cmd_err("shutdown /s", e))?;
    Ok(())
}

/// Log out the current user.
pub fn log_out() -> Result<(), String> {
    Command::new("shutdown")
        .args(["/l"])
        .spawn()
        .map_err(|e| cmd_err("shutdown /l", e))?;
    Ok(())
}

// ── Trash / Recycle Bin ────────────────────────────────────────────

/// Empty the Recycle Bin.
pub fn empty_recycle_bin() -> Result<(), String> {
    let flags = ffi::SHERB_NOCONFIRMATION | ffi::SHERB_NOPROGRESSUI | ffi::SHERB_NOSOUND;
    let hr = unsafe { ffi::SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), flags) };
    // S_OK = 0, S_FALSE = 1 (already empty) — both are fine
    if hr < 0 {
        return Err(format!(
            "SHEmptyRecycleBinW failed (HRESULT 0x{:08X})",
            hr as u32
        ));
    }
    Ok(())
}

// ── UI controls ────────────────────────────────────────────────────

/// Toggle Windows dark mode by flipping the registry key.
pub fn toggle_dark_mode() -> Result<(), String> {
    // Read current value
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .map_err(|e| cmd_err("query dark mode registry", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The output contains "0x0" (dark) or "0x1" (light)
    let current_is_light = stdout.contains("0x1");
    let new_value = if current_is_light { "0" } else { "1" };

    // Set both app and system theme
    for value_name in &["AppsUseLightTheme", "SystemUsesLightTheme"] {
        let out = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                value_name,
                "/t",
                "REG_DWORD",
                "/d",
                new_value,
                "/f",
            ])
            .output()
            .map_err(|e| cmd_err(&format!("set {} registry value", value_name), e))?;

        if !out.status.success() {
            return Err(cmd_fail(&format!("set {}", value_name), &out));
        }
    }

    Ok(())
}

/// Show desktop (Win+D keyboard shortcut).
pub fn show_desktop() -> Result<(), String> {
    unsafe {
        // Press Win+D
        ffi::keybd_event(ffi::VK_LWIN, 0, 0, 0);
        ffi::keybd_event(ffi::VK_D, 0, 0, 0);
        ffi::keybd_event(ffi::VK_D, 0, ffi::KEYEVENTF_KEYUP, 0);
        ffi::keybd_event(ffi::VK_LWIN, 0, ffi::KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

// ── Volume controls ────────────────────────────────────────────────

/// Set system volume to a percentage (0-100).
///
/// Uses `waveOutSetVolume` via a small PowerShell script.
pub fn set_volume(percent: u32) -> Result<(), String> {
    let percent = percent.min(100);
    let fraction = percent as f64 / 100.0;
    let ps_script = format!(
        "Add-Type -TypeDefinition @'\n\
         using System;\n\
         using System.Runtime.InteropServices;\n\
         public class Audio {{\n\
             [DllImport(\"winmm.dll\")]\n\
             public static extern int waveOutSetVolume(IntPtr hwo, uint dwVolume);\n\
         }}\n\
         '@\n\
         $vol = [math]::Round({fraction} * 65535)\n\
         $combined = ($vol -bor ($vol -shl 16))\n\
         [Audio]::waveOutSetVolume([IntPtr]::Zero, $combined)",
        fraction = fraction,
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
        .map_err(|e| cmd_err("set volume via PowerShell", e))?;

    if !output.status.success() {
        return Err(cmd_fail("set volume", &output));
    }
    Ok(())
}

/// Toggle mute via the virtual mute key.
pub fn toggle_mute() -> Result<(), String> {
    let ps_script = concat!(
        "Add-Type -TypeDefinition @'\n",
        "using System;\n",
        "using System.Runtime.InteropServices;\n",
        "public class VolumeControl {\n",
        "    [DllImport(\"user32.dll\")]\n",
        "    public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);\n",
        "    public const byte VK_VOLUME_MUTE = 0xAD;\n",
        "    public const uint KEYEVENTF_KEYUP = 0x0002;\n",
        "}\n",
        "'@\n",
        "[VolumeControl]::keybd_event([VolumeControl]::VK_VOLUME_MUTE, 0, 0, [UIntPtr]::Zero)\n",
        "[VolumeControl]::keybd_event([VolumeControl]::VK_VOLUME_MUTE, 0, [VolumeControl]::KEYEVENTF_KEYUP, [UIntPtr]::Zero)",
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .output()
        .map_err(|e| cmd_err("toggle mute via PowerShell", e))?;

    if !output.status.success() {
        return Err(cmd_fail("toggle mute", &output));
    }
    Ok(())
}

// ── Settings ───────────────────────────────────────────────────────

/// Open a Windows Settings page via `ms-settings:` URI.
pub fn open_settings(uri: &str) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", uri])
        .spawn()
        .map_err(|e| cmd_err(&format!("open settings: {}", uri), e))?;
    Ok(())
}
