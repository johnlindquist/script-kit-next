use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteboardSnapshot {
    #[cfg(target_os = "macos")]
    items: Vec<PasteboardItemSnapshot>,
    pub change_count: i64,
    /// Representations that could not be read and so won't be restored.
    pub skipped_representations: usize,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PasteboardItemSnapshot {
    representations: Vec<PasteboardRepresentation>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PasteboardRepresentation {
    type_name: String,
    data: Vec<u8>,
}

pub fn capture_general_pasteboard_snapshot() -> Result<PasteboardSnapshot> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
    #[cfg(target_os = "macos")]
    {
        PasteboardSnapshot::capture()
    }

    #[cfg(not(target_os = "macos"))]
    {
        bail!("focused-text Agent Chat pasteboard snapshots require macOS");
    }
}

pub fn restore_general_pasteboard_snapshot(snapshot: &PasteboardSnapshot) -> Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
    #[cfg(target_os = "macos")]
    {
        snapshot.restore()
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = snapshot;
        bail!("focused-text Agent Chat pasteboard snapshots require macOS");
    }
}

/// Paste `text` into the focused app through the shared promised-paste path
/// and put the user's full clipboard back once the app has read it.
pub fn paste_plain_text_preserving_clipboard(text: &str) -> Result<()> {
    use crate::platform::promised_paste::{paste_text_with_options, PasteOptions, RestoreOutcome};

    if text.is_empty() {
        return Ok(());
    }
    // Agent output exists nowhere else on screen: if the app ignores the
    // paste, leave it on the clipboard rather than losing it.
    let report = paste_text_with_options(
        text,
        None,
        PasteOptions {
            keep_text_if_unread: true,
        },
    )
    .context("Failed to paste focused-text Agent Chat output")?;
    if report.restore == RestoreOutcome::KeptPastedText {
        bail!("The Agent Chat output is on the clipboard because the focused app didn't take the paste");
    }
    if matches!(
        report.consumption,
        crate::platform::promised_paste::PasteConsumption::NotRequested { .. }
    ) {
        bail!("The focused app didn't take the Agent Chat output");
    }
    match report.restore {
        RestoreOutcome::Restored | RestoreOutcome::KeptPastedText => Ok(()),
        RestoreOutcome::SkippedExternalChange => {
            bail!(
                "Clipboard changed during focused-text Agent Chat paste fallback; skipped restore"
            )
        }
        RestoreOutcome::Failed(error) => bail!(
            "Failed to restore clipboard after focused-text Agent Chat paste fallback: {error}"
        ),
    }
}

/// Raycast-parity selection capture: simulate ⌘C into the active app, wait for
/// the pasteboard change count to move, read the copied text, then restore the
/// user's clipboard. This is the only capture channel that works in AX-opaque
/// editors (Google Docs renders text on a canvas and exposes no AXSelectedText).
/// Returns `Ok(None)` when no copy landed within the deadline (no selection, or
/// the app ignored ⌘C) — the pasteboard is untouched in that case. Unlike
/// `copy_all_plain_text_preserving_clipboard`, this never posts ⌘A, so the
/// user's selection survives.
pub fn copy_selection_plain_text_preserving_clipboard() -> Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        let snapshot = capture_general_pasteboard_snapshot()
            .context("Failed to snapshot clipboard before selection copy fallback")?;
        let baseline_change_count = general_pasteboard_change_count()
            .context("Failed to read clipboard change count before selection copy fallback")?;
        simulate_command_key(KEY_C)
            .context("Failed to simulate copy for selection copy fallback")?;
        let copied_change_count = match wait_for_pasteboard_change(
            baseline_change_count,
            std::time::Duration::from_millis(500),
        ) {
            Ok(count) => count,
            // No clipboard change means the app copied nothing (no selection).
            // The pasteboard is untouched, so there is nothing to restore.
            Err(_) => return Ok(None),
        };
        let text = read_plain_text_from_pasteboard().ok();

        let restore_result = match general_pasteboard_change_count() {
            Ok(current_change_count) if current_change_count == copied_change_count => {
                restore_general_pasteboard_snapshot(&snapshot)
            }
            Ok(_) => {
                bail!("Clipboard changed during selection copy fallback; skipped restore")
            }
            Err(e) => Err(e).context("Failed to read clipboard change count before restore"),
        };
        restore_result.context("Failed to restore clipboard after selection copy fallback")?;

        Ok(text.filter(|text| !text.trim().is_empty()))
    }

    #[cfg(not(target_os = "macos"))]
    {
        bail!("selection copy fallback requires macOS")
    }
}

pub fn copy_all_plain_text_preserving_clipboard() -> Result<String> {
    let snapshot = capture_general_pasteboard_snapshot()
        .context("Failed to snapshot clipboard before focused-text copy fallback")?;

    #[cfg(target_os = "macos")]
    {
        select_all_text_for_focused_text_fallback()?;
        let baseline_change_count = general_pasteboard_change_count()
            .context("Failed to read clipboard change count before focused-text copy fallback")?;
        simulate_command_key(KEY_C).context("Failed to copy text for focused-text fallback")?;
        // Poll the change count instead of sleeping a fixed interval: fast apps
        // finish in a few ms, slow apps get the full deadline, and a copy that
        // never lands is reported as a failure instead of silently returning
        // the user's pre-existing clipboard contents as "captured text".
        let copied_change_count = wait_for_pasteboard_change(
            baseline_change_count,
            std::time::Duration::from_millis(600),
        )
        .context("Focused-text copy fallback did not produce a clipboard change")?;
        let text = read_plain_text_from_pasteboard()
            .context("Failed to read copied focused text from clipboard fallback")?;

        let restore_result = match general_pasteboard_change_count() {
            Ok(current_change_count) if current_change_count == copied_change_count => {
                restore_general_pasteboard_snapshot(&snapshot)
            }
            Ok(_) => {
                bail!("Clipboard changed during focused-text copy fallback; skipped restore")
            }
            Err(e) => Err(e).context("Failed to read clipboard change count before restore"),
        };
        restore_result.context("Failed to restore clipboard after focused-text copy fallback")?;

        Ok(text)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = snapshot;
        bail!("focused-text copy fallback requires macOS");
    }
}

/// Poll the general pasteboard until its change count moves off `baseline_change_count`,
/// returning the new count. Bails once `deadline` elapses with no change.
#[cfg(target_os = "macos")]
fn wait_for_pasteboard_change(
    baseline_change_count: i64,
    deadline: std::time::Duration,
) -> Result<i64> {
    let started = std::time::Instant::now();
    loop {
        let current = general_pasteboard_change_count()?;
        if current != baseline_change_count {
            return Ok(current);
        }
        if started.elapsed() >= deadline {
            bail!(
                "Pasteboard change count stayed at {baseline_change_count} for {}ms after simulated copy",
                deadline.as_millis()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

pub fn select_all_text_for_focused_text_fallback() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        simulate_command_key(KEY_A)
            .context("Failed to select all text for focused-text fallback")?;
        std::thread::sleep(std::time::Duration::from_millis(40));
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        bail!("focused-text select-all fallback requires macOS");
    }
}

#[cfg(target_os = "macos")]
impl PasteboardSnapshot {
    fn capture() -> Result<Self> {
        // SAFETY: generalPasteboard is a shared AppKit singleton.
        let pasteboard = unsafe { general_pasteboard_id()? };
        Self::capture_from(pasteboard)
    }

    /// Snapshot every item, type and data representation of `pasteboard`.
    /// Representations whose data can't be read (for example a file promise
    /// whose owner has quit) are skipped and counted rather than failing the
    /// whole snapshot.
    pub(crate) fn capture_from(pasteboard: cocoa::base::id) -> Result<Self> {
        // Snapshotting the pasteboard is racy: another process (a clipboard
        // manager, a browser, any app) can write the general pasteboard while
        // we enumerate its items, which makes AppKit's mutation handler raise
        // an Obj-C NSException mid-read. Left uncaught that aborts the whole
        // process (observed twice in the wild via EXC_BREAKPOINT in
        // `_updateTypeCacheIfNeeded`). We defend on two axes:
        //   1. Run the raw msg_send enumeration inside `objc_exception::try`
        //      so a concurrent-mutation NSException becomes a recoverable Err.
        //   2. Bracket the read with `changeCount` and retry a bounded number
        //      of times so we return a self-consistent snapshot rather than a
        //      torn one.
        const MAX_ATTEMPTS: usize = 4;
        let mut last_err: Option<anyhow::Error> = None;

        for _ in 0..MAX_ATTEMPTS {
            // SAFETY: `pasteboard` is a live NSPasteboard owned by the caller.
            let before = unsafe { change_count_of(pasteboard) };

            // SAFETY: every raw objc call runs inside `objc_exception::try`,
            // which converts the "collection mutated while enumerated"
            // NSException into `Err(exception_ptr)` instead of unwinding into
            // Rust (which would abort). We never touch the returned pointer.
            let caught =
                unsafe { objc_exception::r#try(|| Self::capture_items_unchecked(pasteboard)) };

            let items = match caught {
                Ok(Ok(items)) => items,
                Ok(Err(read_err)) => return Err(read_err),
                Err(_exception) => {
                    last_err = Some(anyhow::anyhow!(
                        "NSPasteboard was mutated during snapshot (Obj-C exception); retrying"
                    ));
                    std::thread::sleep(std::time::Duration::from_millis(8));
                    continue;
                }
            };

            // SAFETY: as above.
            let after = unsafe { change_count_of(pasteboard) };
            if after != before {
                last_err = Some(anyhow::anyhow!(
                    "clipboard changed during snapshot (change_count {before} -> {after}); retrying"
                ));
                std::thread::sleep(std::time::Duration::from_millis(8));
                continue;
            }

            let (items, skipped_representations) = items;
            return Ok(Self {
                items,
                change_count: after,
                skipped_representations,
            });
        }

        Err(last_err
            .unwrap_or_else(|| anyhow::anyhow!("failed to snapshot pasteboard"))
            .context("Gave up snapshotting a stable clipboard after retries"))
    }

    /// Raw pasteboard enumeration. MUST only be called inside
    /// `objc::exception::catch` (see `capture`): if the pasteboard is mutated
    /// concurrently, AppKit raises an Obj-C exception here that would otherwise
    /// abort the process.
    unsafe fn capture_items_unchecked(
        pasteboard: cocoa::base::id,
    ) -> Result<(Vec<PasteboardItemSnapshot>, usize)> {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};

        if pasteboard == nil {
            bail!("NSPasteboard was nil while snapshotting");
        }
        let items: id = msg_send![pasteboard, pasteboardItems];
        if items == nil {
            return Ok((Vec::new(), 0));
        }
        let mut skipped_representations = 0usize;

        let item_count: usize = msg_send![items, count];
        let mut snapshot_items = Vec::with_capacity(item_count);
        for item_index in 0..item_count {
            let item: id = msg_send![items, objectAtIndex: item_index];
            if item == nil {
                bail!("NSPasteboard returned nil item while snapshotting");
            }
            let types: id = msg_send![item, types];
            if types == nil {
                bail!("NSPasteboard item returned nil type list while snapshotting");
            }

            let type_count: usize = msg_send![types, count];
            let mut representations = Vec::with_capacity(type_count);
            for type_index in 0..type_count {
                let type_id: id = msg_send![types, objectAtIndex: type_index];
                if type_id == nil {
                    bail!("NSPasteboard item returned nil type while snapshotting");
                }
                let type_name = nsstring_to_string(type_id)
                    .context("Failed to read NSPasteboard type name while snapshotting")?;
                let data: id = msg_send![item, dataForType: type_id];
                if data == nil {
                    skipped_representations += 1;
                    continue;
                }

                let byte_len: usize = msg_send![data, length];
                let bytes_ptr: *const u8 = msg_send![data, bytes];
                let data = if byte_len == 0 {
                    Vec::new()
                } else {
                    if bytes_ptr.is_null() {
                        bail!("NSPasteboard item data pointer was nil while snapshotting");
                    }
                    std::slice::from_raw_parts(bytes_ptr, byte_len).to_vec()
                };
                representations.push(PasteboardRepresentation { type_name, data });
            }
            if !representations.is_empty() {
                snapshot_items.push(PasteboardItemSnapshot { representations });
            }
        }

        Ok((snapshot_items, skipped_representations))
    }

    fn restore(&self) -> Result<()> {
        // SAFETY: generalPasteboard is a shared AppKit singleton.
        let pasteboard = unsafe { general_pasteboard_id()? };
        self.restore_to(pasteboard)
    }

    /// Replace the contents of `pasteboard` with this snapshot.
    pub(crate) fn restore_to(&self, pasteboard: cocoa::base::id) -> Result<()> {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSArray, NSData, NSString};
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            if pasteboard == nil {
                bail!("NSPasteboard was nil while restoring clipboard");
            }

            let _: i64 = msg_send![pasteboard, clearContents];
            if self.items.is_empty() {
                return Ok(());
            }

            let mut objects: Vec<id> = Vec::with_capacity(self.items.len());
            for item in &self.items {
                let pasteboard_item: id = msg_send![class!(NSPasteboardItem), new];
                if pasteboard_item == nil {
                    release_objects(&objects);
                    bail!("Failed to create NSPasteboardItem while restoring clipboard");
                }

                for representation in &item.representations {
                    let ns_type = NSString::alloc(nil).init_str(&representation.type_name);
                    if ns_type == nil {
                        let _: () = msg_send![pasteboard_item, release];
                        release_objects(&objects);
                        bail!("Failed to create NSPasteboard type while restoring clipboard");
                    }

                    let data = NSData::dataWithBytes_length_(
                        nil,
                        representation.data.as_ptr() as *const std::ffi::c_void,
                        representation.data.len() as u64,
                    );
                    if data == nil {
                        let _: () = msg_send![ns_type, release];
                        let _: () = msg_send![pasteboard_item, release];
                        release_objects(&objects);
                        bail!("Failed to create NSData while restoring clipboard");
                    }

                    let did_set: bool = msg_send![pasteboard_item, setData: data forType: ns_type];
                    let _: () = msg_send![ns_type, release];
                    if !did_set {
                        let _: () = msg_send![pasteboard_item, release];
                        release_objects(&objects);
                        bail!("NSPasteboardItem.setData returned false while restoring clipboard");
                    }
                }
                objects.push(pasteboard_item);
            }

            let ns_objects: id = NSArray::arrayWithObjects(nil, objects.as_slice());
            if ns_objects == nil {
                release_objects(&objects);
                bail!("Failed to create NSArray while restoring clipboard");
            }

            let did_write: bool = msg_send![pasteboard, writeObjects: ns_objects];
            release_objects(&objects);
            if !did_write {
                bail!("NSPasteboard.writeObjects returned false while restoring clipboard");
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
impl PasteboardSnapshot {
    /// Content-free description for logs: counts and broad kinds only.
    pub(crate) fn summary(&self) -> PasteboardSnapshotSummary {
        let mut summary = PasteboardSnapshotSummary {
            item_count: self.items.len(),
            skipped_representations: self.skipped_representations,
            ..Default::default()
        };
        for item in &self.items {
            for representation in &item.representations {
                summary.type_count += 1;
                summary.total_bytes += representation.data.len();
                let name = representation.type_name.to_ascii_lowercase();
                if name.contains("image") || name.contains("png") || name.contains("tiff") {
                    summary.has_image = true;
                } else if name.contains("file-url") || name.contains("filename") {
                    summary.has_file_url = true;
                } else if name.contains("rtf") || name.contains("html") {
                    summary.has_rich_text = true;
                } else if name.contains("text") || name.contains("string") {
                    summary.has_text = true;
                } else {
                    summary.has_other = true;
                }
            }
        }
        summary
    }

    /// Type names of every item, in order (for tests and diagnostics).
    #[cfg(test)]
    pub(crate) fn type_names(&self) -> Vec<Vec<String>> {
        self.items
            .iter()
            .map(|item| {
                item.representations
                    .iter()
                    .map(|representation| representation.type_name.clone())
                    .collect()
            })
            .collect()
    }

    /// Data stored for `type_name` in item `index` (tests).
    #[cfg(test)]
    pub(crate) fn data_for(&self, index: usize, type_name: &str) -> Option<&[u8]> {
        self.items
            .get(index)?
            .representations
            .iter()
            .find_map(|representation| {
                (representation.type_name == type_name).then_some(representation.data.as_slice())
            })
    }
}

/// Counts and kinds of a pasteboard snapshot; never contains clipboard data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PasteboardSnapshotSummary {
    pub item_count: usize,
    pub type_count: usize,
    pub total_bytes: usize,
    pub skipped_representations: usize,
    pub has_text: bool,
    pub has_rich_text: bool,
    pub has_image: bool,
    pub has_file_url: bool,
    pub has_other: bool,
}

/// The shared general pasteboard (unretained singleton).
#[cfg(target_os = "macos")]
pub(crate) unsafe fn general_pasteboard_id() -> Result<cocoa::base::id> {
    use cocoa::appkit::NSPasteboard;
    use cocoa::base::nil;
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    if pasteboard == nil {
        bail!("NSPasteboard.generalPasteboard returned nil");
    }
    Ok(pasteboard)
}

#[cfg(target_os = "macos")]
pub(crate) unsafe fn change_count_of(pasteboard: cocoa::base::id) -> i64 {
    use objc::{msg_send, sel, sel_impl};
    msg_send![pasteboard, changeCount]
}

#[cfg(target_os = "macos")]
const KEY_A: core_graphics::event::CGKeyCode = 0;
#[cfg(target_os = "macos")]
const KEY_C: core_graphics::event::CGKeyCode = 8;

#[cfg(target_os = "macos")]
fn simulate_command_key(key: core_graphics::event::CGKeyCode) -> Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .ok()
        .context("Failed to create CGEventSource")?;
    let key_down = CGEvent::new_keyboard_event(source.clone(), key, true)
        .ok()
        .context("Failed to create command key down event")?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    let key_up = CGEvent::new_keyboard_event(source, key, false)
        .ok()
        .context("Failed to create command key up event")?;
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);

    key_down.post(CGEventTapLocation::HID);
    std::thread::sleep(std::time::Duration::from_millis(5));
    key_up.post(CGEventTapLocation::HID);
    Ok(())
}

#[cfg(target_os = "macos")]
fn read_plain_text_from_pasteboard() -> Result<String> {
    arboard::Clipboard::new()
        .context("Failed to open clipboard after focused-text copy fallback")?
        .get_text()
        .context("Clipboard did not contain plain text after focused-text copy fallback")
}

pub fn write_plain_text_to_pasteboard(text: &str) -> Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
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
                bail!("Failed to create NSString for focused-text Agent Chat output");
            }

            let objects: id = NSArray::arrayWithObjects(nil, &[ns_text]);
            if objects == nil {
                let _: () = msg_send![ns_text, release];
                bail!("Failed to create NSArray for focused-text Agent Chat output");
            }

            let did_write: bool = msg_send![pasteboard, writeObjects: objects];
            let _: () = msg_send![ns_text, release];

            if !did_write {
                bail!(
                    "NSPasteboard.writeObjects returned false for focused-text Agent Chat output"
                );
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        bail!("focused-text Agent Chat clipboard writes require macOS");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn nsstring_to_string(value: cocoa::base::id) -> Result<String> {
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;

    if value == cocoa::base::nil {
        bail!("NSString value was nil");
    }
    let utf8: *const std::os::raw::c_char = unsafe { msg_send![value, UTF8String] };
    if utf8.is_null() {
        bail!("NSString.UTF8String returned nil");
    }
    Ok(unsafe { CStr::from_ptr(utf8) }
        .to_string_lossy()
        .into_owned())
}

#[cfg(target_os = "macos")]
unsafe fn release_objects(objects: &[cocoa::base::id]) {
    use objc::{msg_send, sel, sel_impl};

    for object in objects {
        let _: () = msg_send![*object, release];
    }
}

pub fn general_pasteboard_change_count() -> Result<i64> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
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

            Ok(msg_send![pasteboard, changeCount])
        }
    }

    #[cfg(not(target_os = "macos"))]
    bail!("focused-text Agent Chat pasteboard change counts require macOS");
}

#[cfg(all(test, target_os = "macos"))]
mod runtime_probe_tests {
    use super::*;

    /// Manual runtime proof of the Raycast-parity selection copy fallback.
    /// Requires AX permission and a frontmost app holding a text selection
    /// (TextEdit fixture; see the selection-fallback probe recipe). Posts a
    /// real ⌘C at the frontmost app, so it must never run in CI:
    /// `agent-cargo.sh test --lib copy_selection_fallback -- --ignored`
    #[test]
    #[ignore = "posts a real ⌘C at the frontmost app; run manually with a TextEdit selection fixture"]
    fn copy_selection_fallback_captures_selection_and_restores_clipboard() {
        eprintln!(
            "test process AX trusted: {}",
            super::super::permissions::has_accessibility_permission()
        );
        let clipboard_before = arboard::Clipboard::new()
            .expect("open clipboard")
            .get_text()
            .unwrap_or_default();
        let change_count_before =
            general_pasteboard_change_count().expect("read change count before");

        let captured = copy_selection_plain_text_preserving_clipboard()
            .expect("selection copy fallback should not error");

        let clipboard_after = arboard::Clipboard::new()
            .expect("open clipboard")
            .get_text()
            .unwrap_or_default();
        assert_eq!(
            clipboard_before, clipboard_after,
            "user clipboard must be restored byte-identically"
        );

        match captured {
            Some(text) => {
                assert!(
                    !text.trim().is_empty(),
                    "captured selection must be non-empty"
                );
                // A successful capture writes + restores: change count moves.
                let change_count_after =
                    general_pasteboard_change_count().expect("read change count after");
                assert!(
                    change_count_after > change_count_before,
                    "capture path should have bumped the change count during copy+restore"
                );
                eprintln!(
                    "captured selection ({} chars): {text}",
                    text.chars().count()
                );
            }
            None => {
                // No selection in the frontmost app: pasteboard untouched.
                let change_count_after =
                    general_pasteboard_change_count().expect("read change count after");
                assert_eq!(
                    change_count_before, change_count_after,
                    "no-selection path must leave the pasteboard completely untouched"
                );
                eprintln!("no selection captured; pasteboard untouched");
            }
        }
    }
}
