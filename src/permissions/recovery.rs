//! Getting out of a grant that doesn't take.
//!
//! macOS keys a privacy grant to the app's identity. After Script Kit is
//! rebuilt, re-signed or moved, System Settings can show it switched on while
//! macOS still reports it as off, and Screen Recording applies only after a
//! relaunch. Two fixes cover both cases: restart Script Kit, or reset its entry
//! with `tccutil` (part of macOS) so the next request asks afresh.

use super::PermissionKind;
use anyhow::{bail, Context, Result};

/// The `tccutil` service name for `kind`.
pub fn tcc_service(kind: PermissionKind) -> &'static str {
    match kind {
        PermissionKind::Accessibility => "Accessibility",
        PermissionKind::ScreenRecording => "ScreenCapture",
        PermissionKind::Microphone => "Microphone",
        PermissionKind::InputMonitoring => "ListenEvent",
        PermissionKind::EventSynthesizing => "PostEvent",
    }
}

/// Script Kit's bundle identifier, when it runs from an app bundle.
pub fn host_bundle_identifier() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};
        use std::ffi::CStr;

        crate::platform::permiso::host_app::host_app_bundle_url().ok()?;
        // SAFETY: mainBundle is a process-wide singleton; bundleIdentifier
        // returns an autoreleased NSString or nil, read immediately.
        unsafe {
            let bundle: id = msg_send![class!(NSBundle), mainBundle];
            if bundle == nil {
                return None;
            }
            let identifier: id = msg_send![bundle, bundleIdentifier];
            if identifier == nil {
                return None;
            }
            let utf8: *const std::os::raw::c_char = msg_send![identifier, UTF8String];
            if utf8.is_null() {
                return None;
            }
            let value = CStr::from_ptr(utf8).to_string_lossy().into_owned();
            (!value.is_empty()).then_some(value)
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Whether [`relaunch_after_exit`] knows what to start again.
pub fn restart_supported() -> bool {
    cfg!(target_os = "macos") && relaunch_target().is_some()
}

/// What to start again: the app bundle, or the executable when Script Kit
/// runs outside a bundle.
fn relaunch_target() -> Option<(RelaunchKind, std::path::PathBuf)> {
    if let Ok(bundle) = crate::platform::permiso::host_app::host_app_bundle_url() {
        return Some((RelaunchKind::Bundle, bundle));
    }
    std::env::current_exe()
        .ok()
        .map(|exe| (RelaunchKind::Executable, exe))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelaunchKind {
    Bundle,
    Executable,
}

/// The shell script that waits for `pid` to exit (at most 30 s), then starts
/// the target. Arguments: `$0` = pid, `$1` = path.
fn relaunch_script(kind: RelaunchKind) -> &'static str {
    match kind {
        RelaunchKind::Bundle => {
            "i=0; while /bin/kill -0 \"$0\" 2>/dev/null && [ $i -lt 150 ]; do /bin/sleep 0.2; i=$((i+1)); done; exec /usr/bin/open \"$1\""
        }
        RelaunchKind::Executable => {
            "i=0; while /bin/kill -0 \"$0\" 2>/dev/null && [ $i -lt 150 ]; do /bin/sleep 0.2; i=$((i+1)); done; exec \"$1\""
        }
    }
}

/// Start a helper that reopens Script Kit once this process has exited. The
/// caller quits the app right after.
pub fn relaunch_after_exit() -> Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
    let Some((kind, target)) = relaunch_target() else {
        bail!("Couldn't find the Script Kit app to start again");
    };
    std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(relaunch_script(kind))
        .arg(std::process::id().to_string())
        .arg(&target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Couldn't start the relaunch helper")?;
    tracing::info!(
        category = "PERMISSIONS",
        event = "relaunch_scheduled",
        target = %target.display(),
        "Script Kit will reopen after it quits"
    );
    Ok(())
}

/// Remove Script Kit's entry for `kind` from the privacy database, so the
/// next request shows the macOS prompt again and records the current app.
pub fn reset_entry(kind: PermissionKind) -> Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
    let Some(bundle_id) = host_bundle_identifier() else {
        bail!("Script Kit isn't running from an app bundle, so its entry can't be reset");
    };
    let output = std::process::Command::new("/usr/bin/tccutil")
        .args(["reset", tcc_service(kind), &bundle_id])
        .output()
        .context("Couldn't run tccutil")?;
    if !output.status.success() {
        bail!(
            "tccutil couldn't reset {}: {}",
            kind.name(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    tracing::info!(
        category = "PERMISSIONS",
        event = "permission_entry_reset",
        permission = %kind,
        bundle_id = %bundle_id,
        "Reset Script Kit's privacy entry"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_tcc_service() {
        let services: Vec<_> = PermissionKind::all()
            .iter()
            .map(|&kind| tcc_service(kind))
            .collect();
        assert_eq!(
            services,
            vec![
                "Accessibility",
                "ScreenCapture",
                "PostEvent",
                "ListenEvent",
                "Microphone"
            ]
        );
    }

    #[test]
    fn relaunch_script_waits_for_exit_then_opens() {
        let bundle = relaunch_script(RelaunchKind::Bundle);
        assert!(bundle.contains("kill -0 \"$0\""));
        assert!(bundle.ends_with("exec /usr/bin/open \"$1\""));
        assert!(relaunch_script(RelaunchKind::Executable).ends_with("exec \"$1\""));
    }

    #[test]
    fn relaunch_script_gives_up_waiting_after_thirty_seconds() {
        // 150 checks, 0.2 s apart.
        assert!(relaunch_script(RelaunchKind::Bundle).contains("$i -lt 150"));
    }
}
