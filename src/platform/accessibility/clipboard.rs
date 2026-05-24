use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteboardSnapshot {
    pub item_count: usize,
    pub change_count: Option<i64>,
}

pub fn capture_general_pasteboard_snapshot() -> Result<PasteboardSnapshot> {
    #[cfg(target_os = "macos")]
    {
        Ok(PasteboardSnapshot {
            item_count: 0,
            change_count: Some(general_pasteboard_change_count()?),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(PasteboardSnapshot {
            item_count: 0,
            change_count: None,
        })
    }
}

pub fn restore_general_pasteboard_snapshot(_snapshot: &PasteboardSnapshot) -> Result<()> {
    Ok(())
}

pub fn write_plain_text_to_pasteboard(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSPasteboard;
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSArray, NSString};
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let pasteboard: id = NSPasteboard::generalPasteboard(nil);
            if pasteboard == nil {
                bail!("NSPasteboard.generalPasteboard returned nil");
            }

            let _: i64 = msg_send![pasteboard, clearContents];
            let ns_text = NSString::alloc(nil).init_str(text);
            if ns_text == nil {
                bail!("Failed to create NSString for inline-agent output");
            }

            let objects: id = NSArray::arrayWithObjects(nil, &[ns_text]);
            if objects == nil {
                let _: () = msg_send![ns_text, release];
                bail!("Failed to create NSArray for inline-agent output");
            }

            let did_write: bool = msg_send![pasteboard, writeObjects: objects];
            let _: () = msg_send![ns_text, release];

            if !did_write {
                bail!("NSPasteboard.writeObjects returned false for inline-agent output");
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        bail!("inline-agent clipboard writes require macOS");
    }

    Ok(())
}

pub fn general_pasteboard_change_count() -> Result<i64> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSPasteboard;
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let pasteboard: id = NSPasteboard::generalPasteboard(nil);
            if pasteboard == nil {
                bail!("NSPasteboard.generalPasteboard returned nil");
            }

            return Ok(msg_send![pasteboard, changeCount]);
        }
    }

    #[cfg(not(target_os = "macos"))]
    bail!("inline-agent pasteboard change counts require macOS");
}
