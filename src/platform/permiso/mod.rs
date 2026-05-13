pub mod panel;

pub use panel::PermisoPanel;

pub struct PermisoAssistant;

#[cfg(target_os = "macos")]
pub fn present_settings_url(panel: PermisoPanel) -> anyhow::Result<()> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CString;

    let url = CString::new(panel.settings_url())?;

    unsafe {
        let ns_string: id = msg_send![class!(NSString), stringWithUTF8String: url.as_ptr()];
        if ns_string == nil {
            anyhow::bail!("failed to create settings URL string");
        }

        let ns_url: id = msg_send![class!(NSURL), URLWithString: ns_string];
        if ns_url == nil {
            anyhow::bail!("failed to create settings URL");
        }

        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            anyhow::bail!("failed to access NSWorkspace");
        }

        let opened: bool = msg_send![workspace, openURL: ns_url];
        if !opened {
            anyhow::bail!("failed to open {} settings", panel.display_name());
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn present_settings_url(_panel: PermisoPanel) -> anyhow::Result<()> {
    anyhow::bail!("Permission assistant is only available on macOS");
}
