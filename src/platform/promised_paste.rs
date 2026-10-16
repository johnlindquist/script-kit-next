//! Paste text into the focused application and give the user's clipboard back.
//!
//! The clipboard paste has one hard question: when is it safe to put the
//! user's clipboard back? Restoring too early makes the target paste the old
//! clipboard; restoring too late leaves the snippet behind. Waiting a fixed
//! time guesses.
//!
//! Instead, the text goes on the pasteboard as a *promise*
//! (`NSPasteboardItem` with a data provider). The target app's paste is what
//! asks for the bytes, so the provider callback is an observed "the paste
//! happened" signal. After it fires (or a bounded timeout), the full original
//! pasteboard — every item and type — is written back, unless something else
//! changed the clipboard in the meantime.
//!
//! The item also carries `org.nspasteboard.TransientType`, which tells
//! clipboard managers (including Script Kit's own history) not to record it.
//!
//! Promise callbacks are delivered on the main thread's run loop, so a promise
//! is only used when the caller is on another thread. On the main thread, or
//! with `SCRIPT_KIT_PASTE_DELIVERY=eager`, the text is written up front and a
//! fixed settle time is used instead.

use super::accessibility::clipboard::PasteboardSnapshotSummary;
use super::synthetic_keys::KeyTarget;
use anyhow::Result;
use std::time::Duration;

/// Set to `eager` to always write text up front (no completion signal).
pub const PASTE_DELIVERY_ENV: &str = "SCRIPT_KIT_PASTE_DELIVERY";

/// Marker type defined at nspasteboard.org: "don't record this".
pub const TRANSIENT_PASTEBOARD_TYPE: &str = "org.nspasteboard.TransientType";
const PLAIN_TEXT_TYPE: &str = "public.utf8-plain-text";

/// Waits used by one paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteTiming {
    /// Longest wait for the target to ask for the promised text.
    pub request_timeout: Duration,
    /// After the target asked, wait this long before restoring, so an app
    /// that reads the pasteboard more than once still sees the text.
    pub settle_after_request: Duration,
    /// Eager delivery has no signal; wait this long before restoring.
    pub eager_settle: Duration,
}

impl Default for PasteTiming {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_millis(1200),
            settle_after_request: Duration::from_millis(150),
            eager_settle: Duration::from_millis(250),
        }
    }
}

/// How the text was put on the pasteboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    Promised,
    Eager,
}

impl Delivery {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Promised => "promised",
            Self::Eager => "eager",
        }
    }
}

/// Whether the target app was seen reading the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteConsumption {
    /// The target asked for the text this long after ⌘V was posted.
    Requested { after: Duration },
    /// Nothing asked within the timeout: the paste most likely did not land.
    NotRequested { waited: Duration },
    /// Eager delivery, or ⌘V could not be posted: nothing to observe.
    Unobserved,
}

impl PasteConsumption {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested { .. } => "requested",
            Self::NotRequested { .. } => "not_requested",
            Self::Unobserved => "unobserved",
        }
    }
}

/// What happened to the user's original clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreOutcome {
    Restored,
    /// Something else wrote the clipboard during the paste; it was kept.
    SkippedExternalChange,
    /// The target never read the text, so the text was left on the clipboard
    /// for the user to paste ([`PasteOptions::keep_text_if_unread`]).
    KeptPastedText,
    Failed(String),
}

impl RestoreOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Restored => "restored",
            Self::SkippedExternalChange => "skipped_external_change",
            Self::KeptPastedText => "kept_pasted_text",
            Self::Failed(_) => "failed",
        }
    }
}

/// Choices for one paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PasteOptions {
    /// When the target never reads the text, leave the text on the clipboard
    /// (so the user can press ⌘V) instead of restoring the old clipboard.
    /// Right for text that exists nowhere else, such as a transcript; the
    /// old clipboard stays in clipboard history.
    pub keep_text_if_unread: bool,
}

/// Result of one paste, safe to log (no clipboard or text content).
#[derive(Debug, Clone)]
pub struct PasteReport {
    pub delivery: Delivery,
    pub consumption: PasteConsumption,
    pub restore: RestoreOutcome,
    pub target: KeyTarget,
    pub original: PasteboardSnapshotSummary,
}

fn choose_delivery(on_main_thread: bool, env_value: Option<&str>) -> Delivery {
    let forced_eager = env_value.is_some_and(|value| value.trim().eq_ignore_ascii_case("eager"));
    if on_main_thread || forced_eager {
        Delivery::Eager
    } else {
        Delivery::Promised
    }
}

#[cfg(target_os = "macos")]
pub use macos::{copy_text_to_clipboard, paste_text_with_options};

/// Paste `text` into `target` (default: the focused app) and restore the
/// user's full clipboard afterwards.
pub fn paste_text_preserving_clipboard(
    text: &str,
    target: Option<KeyTarget>,
) -> Result<PasteReport> {
    paste_text_with_options(text, target, PasteOptions::default())
}

#[cfg(not(target_os = "macos"))]
pub fn paste_text_with_options(
    _text: &str,
    _target: Option<KeyTarget>,
    _options: PasteOptions,
) -> Result<PasteReport> {
    anyhow::bail!("pasting into other applications requires macOS")
}

#[cfg(not(target_os = "macos"))]
pub fn copy_text_to_clipboard(_text: &str) -> Result<()> {
    anyhow::bail!("the system clipboard requires macOS")
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::platform::accessibility::clipboard::{
        change_count_of, general_pasteboard_id, PasteboardSnapshot,
    };
    use crate::platform::synthetic_keys;
    use anyhow::{bail, Context};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSArray, NSString};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Protocol, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::c_void;
    use std::sync::{Arc, Condvar, Mutex, OnceLock};
    use std::time::Instant;
    use tracing::{info, warn};

    /// One paste at a time per process, so a second paste always snapshots
    /// the user's real clipboard rather than the first paste's text.
    static PASTE_LOCK: Mutex<()> = Mutex::new(());

    /// Put `text` on the clipboard as ordinary copied text (recorded by
    /// clipboard managers), for when pasting isn't possible.
    pub fn copy_text_to_clipboard(text: &str) -> Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
        objc::rc::autoreleasepool(|| {
            // SAFETY: generalPasteboard is a shared AppKit singleton.
            let pasteboard = unsafe { general_pasteboard_id()? };
            // SAFETY: `pasteboard` is a live NSPasteboard.
            unsafe { write_paste_item(pasteboard, text, Delivery::Eager, false) }.map(|_| ())
        })
    }

    /// Paste `text` into `target` (default: the focused app), then restore
    /// the user's full clipboard (or keep the text, see [`PasteOptions`]).
    pub fn paste_text_with_options(
        text: &str,
        target: Option<KeyTarget>,
        options: PasteOptions,
    ) -> Result<PasteReport> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemClipboard)?;
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;

        let on_main_thread = is_main_thread();
        // A background paste may be waiting for the main thread to answer its
        // promise; blocking the main thread on the lock would stall both.
        let _serial = if on_main_thread {
            match PASTE_LOCK.try_lock() {
                Ok(guard) => guard,
                Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => {
                    bail!("Another paste into the focused app is still in progress")
                }
            }
        } else {
            PASTE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
        };
        let target = target.unwrap_or_else(synthetic_keys::focused_key_target);
        let delivery = choose_delivery(
            on_main_thread,
            std::env::var(PASTE_DELIVERY_ENV).ok().as_deref(),
        );
        let timing = PasteTiming::default();
        let started = Instant::now();

        let result = objc::rc::autoreleasepool(|| {
            // SAFETY: generalPasteboard is a shared AppKit singleton.
            let pasteboard = unsafe { general_pasteboard_id()? };
            run_paste_with(pasteboard, text, delivery, timing, options, || {
                synthetic_keys::post_paste_shortcut(target)
            })
        });

        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                warn!(
                    category = "PASTE",
                    event = "synthetic_paste",
                    delivery = delivery.as_str(),
                    route = target.route_name(),
                    text_len = text.len(),
                    total_ms = started.elapsed().as_millis() as u64,
                    success = false,
                    error = %error,
                    "Paste into focused app failed"
                );
                return Err(error);
            }
        };

        let report = PasteReport {
            delivery,
            consumption: outcome.consumption,
            restore: outcome.restore,
            target,
            original: outcome.original,
        };
        log_report(&report, text.len(), started.elapsed());
        Ok(report)
    }

    fn log_report(report: &PasteReport, text_len: usize, total: Duration) {
        let requested_after_ms = match report.consumption {
            PasteConsumption::Requested { after } => after.as_millis() as u64,
            _ => 0,
        };
        let restore_error = match &report.restore {
            RestoreOutcome::Failed(error) => error.as_str(),
            _ => "",
        };
        let consumed = matches!(
            report.consumption,
            PasteConsumption::Requested { .. } | PasteConsumption::Unobserved
        );
        let fields_ok = consumed && matches!(report.restore, RestoreOutcome::Restored);
        if fields_ok {
            info!(
                category = "PASTE",
                event = "synthetic_paste",
                delivery = report.delivery.as_str(),
                consumption = report.consumption.as_str(),
                requested_after_ms,
                restore = report.restore.as_str(),
                route = report.target.route_name(),
                text_len,
                original_items = report.original.item_count,
                original_types = report.original.type_count,
                original_bytes = report.original.total_bytes,
                original_unreadable_types = report.original.skipped_representations,
                total_ms = total.as_millis() as u64,
                success = true,
                "Pasted into focused app"
            );
        } else {
            warn!(
                category = "PASTE",
                event = "synthetic_paste",
                delivery = report.delivery.as_str(),
                consumption = report.consumption.as_str(),
                requested_after_ms,
                restore = report.restore.as_str(),
                restore_error,
                route = report.target.route_name(),
                text_len,
                original_items = report.original.item_count,
                original_types = report.original.type_count,
                original_bytes = report.original.total_bytes,
                original_unreadable_types = report.original.skipped_representations,
                total_ms = total.as_millis() as u64,
                success = consumed,
                "Paste finished with a warning: the target did not read the text or the clipboard was not restored"
            );
        }
    }

    pub(crate) struct PasteOutcome {
        pub consumption: PasteConsumption,
        pub restore: RestoreOutcome,
        pub original: PasteboardSnapshotSummary,
    }

    /// [`run_paste_with`] using the default options.
    #[cfg(test)]
    pub(crate) fn run_paste(
        pasteboard: id,
        text: &str,
        delivery: Delivery,
        timing: PasteTiming,
        post_shortcut: impl FnOnce() -> Result<()>,
    ) -> Result<PasteOutcome> {
        run_paste_with(
            pasteboard,
            text,
            delivery,
            timing,
            PasteOptions::default(),
            post_shortcut,
        )
    }

    /// Snapshot → write → post shortcut → wait → restore, on any pasteboard.
    /// Returns the shortcut error (after the restore attempt) if posting failed.
    pub(crate) fn run_paste_with(
        pasteboard: id,
        text: &str,
        delivery: Delivery,
        timing: PasteTiming,
        options: PasteOptions,
        post_shortcut: impl FnOnce() -> Result<()>,
    ) -> Result<PasteOutcome> {
        let snapshot = PasteboardSnapshot::capture_from(pasteboard)
            .context("Failed to snapshot clipboard before paste")?;
        let original = snapshot.summary();

        // SAFETY: `pasteboard` is a live NSPasteboard.
        let (written_change_count, promise) =
            match unsafe { write_paste_item(pasteboard, text, delivery, true) } {
                Ok(written) => written,
                Err(error) => {
                    // The write may have cleared the clipboard before failing.
                    // SAFETY: `pasteboard` is a live NSPasteboard.
                    if unsafe { change_count_of(pasteboard) } != snapshot.change_count {
                        if let Err(restore_error) = snapshot.restore_to(pasteboard) {
                            warn!(
                                category = "PASTE",
                                error = %restore_error,
                                "Failed to restore clipboard after a failed paste write"
                            );
                        }
                    }
                    return Err(error);
                }
            };

        let posted_at = Instant::now();
        let post_result = post_shortcut();

        let consumption = if post_result.is_err() {
            PasteConsumption::Unobserved
        } else {
            match &promise {
                Some(promise) => match promise.wait_for_request(timing.request_timeout) {
                    Some(requested_at) => {
                        std::thread::sleep(timing.settle_after_request);
                        PasteConsumption::Requested {
                            after: requested_at.saturating_duration_since(posted_at),
                        }
                    }
                    None => PasteConsumption::NotRequested {
                        waited: posted_at.elapsed(),
                    },
                },
                None => {
                    std::thread::sleep(timing.eager_settle);
                    PasteConsumption::Unobserved
                }
            }
        };

        // SAFETY: `pasteboard` is a live NSPasteboard.
        let current_change_count = unsafe { change_count_of(pasteboard) };
        let unread = matches!(consumption, PasteConsumption::NotRequested { .. });
        let restore = if current_change_count != written_change_count {
            RestoreOutcome::SkippedExternalChange
        } else if unread && options.keep_text_if_unread && keep_text(pasteboard, text) {
            RestoreOutcome::KeptPastedText
        } else {
            match snapshot.restore_to(pasteboard) {
                Ok(()) => RestoreOutcome::Restored,
                Err(error) => RestoreOutcome::Failed(format!("{error:#}")),
            }
        };
        drop(promise);

        post_result.context("Failed to post the paste shortcut")?;
        Ok(PasteOutcome {
            consumption,
            restore,
            original,
        })
    }

    /// Leave `text` itself on the clipboard: the promise ends with this paste.
    fn keep_text(pasteboard: id, text: &str) -> bool {
        // SAFETY: `pasteboard` is a live NSPasteboard.
        unsafe { write_paste_item(pasteboard, text, Delivery::Eager, false) }.is_ok()
    }

    /// Clear `pasteboard` and write one item holding `text` (promised or
    /// eager), plus the transient marker when `transient`. Returns the
    /// resulting change count.
    unsafe fn write_paste_item(
        pasteboard: id,
        text: &str,
        delivery: Delivery,
        transient: bool,
    ) -> Result<(i64, Option<PromisedText>)> {
        let item: id = msg_send![class!(NSPasteboardItem), new];
        if item == nil {
            bail!("Failed to create NSPasteboardItem for paste");
        }
        let text_type = NSString::alloc(nil).init_str(PLAIN_TEXT_TYPE);
        let transient_type = NSString::alloc(nil).init_str(TRANSIENT_PASTEBOARD_TYPE);
        let release_all = |objects: &[id]| {
            for object in objects {
                let _: () = msg_send![*object, release];
            }
        };

        let promise = match delivery {
            Delivery::Promised => {
                let promise = PromisedText::new(text)?;
                let types = NSArray::arrayWithObjects(nil, &[text_type]);
                let ok: bool = msg_send![item, setDataProvider: promise.provider forTypes: types];
                if !ok {
                    release_all(&[item, text_type, transient_type]);
                    bail!("NSPasteboardItem.setDataProvider returned false");
                }
                Some(promise)
            }
            Delivery::Eager => {
                let ns_text = NSString::alloc(nil).init_str(text);
                let ok: bool = msg_send![item, setString: ns_text forType: text_type];
                let _: () = msg_send![ns_text, release];
                if !ok {
                    release_all(&[item, text_type, transient_type]);
                    bail!("NSPasteboardItem.setString returned false");
                }
                None
            }
        };

        if transient {
            let empty: id = msg_send![class!(NSData), data];
            let _: bool = msg_send![item, setData: empty forType: transient_type];
        }

        let _: i64 = msg_send![pasteboard, clearContents];
        let objects = NSArray::arrayWithObjects(nil, &[item]);
        let written: bool = msg_send![pasteboard, writeObjects: objects];
        release_all(&[item, text_type, transient_type]);
        if !written {
            bail!("NSPasteboard.writeObjects returned false for paste");
        }
        Ok((change_count_of(pasteboard), promise))
    }

    // ------------------------------------------------------------------
    // Promised text provider
    // ------------------------------------------------------------------

    struct PromiseState {
        text: String,
        requested_at: Mutex<Option<Instant>>,
        requested: Condvar,
    }

    impl PromiseState {
        fn mark_requested(&self) {
            let mut requested_at = self.requested_at.lock().unwrap_or_else(|e| e.into_inner());
            requested_at.get_or_insert_with(Instant::now);
            self.requested.notify_all();
        }
    }

    /// Our +1 reference to the provider object. The pasteboard item retains
    /// the provider too, so releasing ours never leaves a dangling provider.
    struct PromisedText {
        provider: id,
        state: Arc<PromiseState>,
    }

    impl PromisedText {
        fn new(text: &str) -> Result<Self> {
            let class = provider_class().context("Promised-text provider class is unavailable")?;
            let state = Arc::new(PromiseState {
                text: text.to_string(),
                requested_at: Mutex::new(None),
                requested: Condvar::new(),
            });
            // SAFETY: alloc/init a registered NSObject subclass; the ivar
            // takes one strong count of `state`, given back in `dealloc`.
            unsafe {
                let provider: id = msg_send![class, alloc];
                let provider: id = msg_send![provider, init];
                if provider == nil {
                    bail!("Failed to create promised-text provider");
                }
                let raw = Arc::into_raw(Arc::clone(&state)) as *mut c_void;
                (*provider).set_ivar::<*mut c_void>(STATE_IVAR, raw);
                Ok(Self { provider, state })
            }
        }

        fn wait_for_request(&self, timeout: Duration) -> Option<Instant> {
            let guard = self
                .state
                .requested_at
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let (guard, _) = self
                .state
                .requested
                .wait_timeout_while(guard, timeout, |requested_at| requested_at.is_none())
                .unwrap_or_else(|e| e.into_inner());
            *guard
        }
    }

    impl Drop for PromisedText {
        fn drop(&mut self) {
            // SAFETY: balances the alloc/init in `new`.
            unsafe {
                let _: () = msg_send![self.provider, release];
            }
        }
    }

    const STATE_IVAR: &str = "_scriptKitPromiseState";

    fn provider_class() -> Option<&'static Class> {
        static CLASS: OnceLock<Option<usize>> = OnceLock::new();
        let address = CLASS.get_or_init(|| {
            const NAME: &str = "ScriptKitPromisedTextProvider";
            if let Some(existing) = Class::get(NAME) {
                return Some(existing as *const Class as usize);
            }
            let mut decl = ClassDecl::new(NAME, class!(NSObject))?;
            decl.add_ivar::<*mut c_void>(STATE_IVAR);
            // SAFETY: method signatures match the Objective-C selectors.
            unsafe {
                decl.add_method(
                    sel!(pasteboard:item:provideDataForType:),
                    provide_data as extern "C" fn(&Object, Sel, id, id, id),
                );
                decl.add_method(
                    sel!(pasteboardFinishedWithDataProvider:),
                    finished_with_provider as extern "C" fn(&Object, Sel, id),
                );
                decl.add_method(
                    sel!(dealloc),
                    dealloc_provider as extern "C" fn(&mut Object, Sel),
                );
            }
            if let Some(protocol) = Protocol::get("NSPasteboardItemDataProvider") {
                decl.add_protocol(protocol);
            }
            Some(decl.register() as *const Class as usize)
        });
        // SAFETY: registered classes live for the life of the process.
        address.map(|address| unsafe { &*(address as *const Class) })
    }

    extern "C" fn provide_data(this: &Object, _sel: Sel, _pasteboard: id, item: id, data_type: id) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // SAFETY: the ivar holds an Arc<PromiseState> pointer set in
            // `PromisedText::new` and released only in `dealloc`.
            unsafe {
                let raw: *mut c_void = *this.get_ivar(STATE_IVAR);
                if raw.is_null() {
                    return;
                }
                let state = &*(raw as *const PromiseState);
                let ns_text = NSString::alloc(nil).init_str(&state.text);
                let _: bool = msg_send![item, setString: ns_text forType: data_type];
                let _: () = msg_send![ns_text, release];
                state.mark_requested();
            }
        }));
    }

    extern "C" fn finished_with_provider(_this: &Object, _sel: Sel, _pasteboard: id) {}

    extern "C" fn dealloc_provider(this: &mut Object, _sel: Sel) {
        // SAFETY: reclaims the Arc stored in `PromisedText::new`, then calls
        // NSObject's dealloc.
        unsafe {
            let raw: *mut c_void = *this.get_ivar(STATE_IVAR);
            if !raw.is_null() {
                drop(Arc::from_raw(raw as *const PromiseState));
                this.set_ivar::<*mut c_void>(STATE_IVAR, std::ptr::null_mut());
            }
            let _: () = msg_send![super(this, class!(NSObject)), dealloc];
        }
    }

    fn is_main_thread() -> bool {
        // SAFETY: NSThread.isMainThread only reads thread identity.
        unsafe { msg_send![class!(NSThread), isMainThread] }
    }

    // ------------------------------------------------------------------
    // Tests: a private, uniquely named pasteboard stands in for the general
    // clipboard, and a reader thread stands in for the target app.
    // ------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::mpsc;

        struct PrivatePasteboard(id);

        // SAFETY: NSPasteboard is thread-safe for these calls; tests hand
        // the pointer to one reader thread at a time.
        unsafe impl Send for PrivatePasteboard {}

        impl PrivatePasteboard {
            fn new() -> Self {
                // SAFETY: retained here, released (globally) in Drop.
                unsafe {
                    let pasteboard: id = msg_send![class!(NSPasteboard), pasteboardWithUniqueName];
                    let _: id = msg_send![pasteboard, retain];
                    Self(pasteboard)
                }
            }

            fn write(&self, items: &[&[(&str, &[u8])]]) {
                // SAFETY: plain AppKit writes on a private pasteboard.
                unsafe {
                    let _: i64 = msg_send![self.0, clearContents];
                    let mut objects = Vec::new();
                    for representations in items {
                        let item: id = msg_send![class!(NSPasteboardItem), new];
                        for (type_name, bytes) in *representations {
                            let ns_type = NSString::alloc(nil).init_str(type_name);
                            let data: id = msg_send![class!(NSData), dataWithBytes: bytes.as_ptr() as *const c_void length: bytes.len()];
                            let _: bool = msg_send![item, setData: data forType: ns_type];
                            let _: () = msg_send![ns_type, release];
                        }
                        objects.push(item);
                    }
                    let array = NSArray::arrayWithObjects(nil, &objects);
                    let _: bool = msg_send![self.0, writeObjects: array];
                    for object in objects {
                        let _: () = msg_send![object, release];
                    }
                }
            }

            fn read_string(&self) -> Option<String> {
                // SAFETY: AppKit read; result copied into Rust.
                unsafe {
                    let ns_type = NSString::alloc(nil).init_str(PLAIN_TEXT_TYPE);
                    let value: id = msg_send![self.0, stringForType: ns_type];
                    let _: () = msg_send![ns_type, release];
                    if value == nil {
                        return None;
                    }
                    let utf8: *const std::os::raw::c_char = msg_send![value, UTF8String];
                    Some(
                        std::ffi::CStr::from_ptr(utf8)
                            .to_string_lossy()
                            .into_owned(),
                    )
                }
            }

            fn has_type(&self, type_name: &str) -> bool {
                // SAFETY: AppKit read.
                unsafe {
                    let types: id = msg_send![self.0, types];
                    if types == nil {
                        return false;
                    }
                    let count: usize = msg_send![types, count];
                    (0..count).any(|index| {
                        let value: id = msg_send![types, objectAtIndex: index];
                        let utf8: *const std::os::raw::c_char = msg_send![value, UTF8String];
                        std::ffi::CStr::from_ptr(utf8).to_string_lossy() == type_name
                    })
                }
            }

            fn snapshot(&self) -> PasteboardSnapshot {
                PasteboardSnapshot::capture_from(self.0).expect("snapshot private pasteboard")
            }
        }

        impl Drop for PrivatePasteboard {
            fn drop(&mut self) {
                // SAFETY: balances the retain in `new`.
                unsafe {
                    let _: () = msg_send![self.0, releaseGlobally];
                    let _: () = msg_send![self.0, release];
                }
            }
        }

        const RTF: &[u8] = b"{\\rtf1 original}";
        const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 1, 2, 3];

        fn write_original(pasteboard: &PrivatePasteboard) {
            pasteboard.write(&[
                &[(PLAIN_TEXT_TYPE, b"original text"), ("public.rtf", RTF)],
                &[("public.png", PNG)],
            ]);
        }

        /// AppKit may report extra synthesized types (for example
        /// `public.utf16-external-plain-text`) next to the ones written, so
        /// compare against a snapshot taken before the paste, plus the
        /// payloads written by `write_original`.
        fn assert_original_restored(pasteboard: &PrivatePasteboard, before: &PasteboardSnapshot) {
            let after = pasteboard.snapshot();
            assert_eq!(after.type_names(), before.type_names());
            for (index, types) in before.type_names().iter().enumerate() {
                for type_name in types {
                    assert_eq!(
                        after.data_for(index, type_name),
                        before.data_for(index, type_name),
                        "item {index} type {type_name}"
                    );
                }
            }
            assert_eq!(
                after.data_for(0, PLAIN_TEXT_TYPE),
                Some(&b"original text"[..])
            );
            assert_eq!(after.data_for(0, "public.rtf"), Some(RTF));
            assert_eq!(after.data_for(1, "public.png"), Some(PNG));
        }

        fn quick_timing() -> PasteTiming {
            PasteTiming {
                request_timeout: Duration::from_millis(1000),
                settle_after_request: Duration::from_millis(20),
                eager_settle: Duration::from_millis(20),
            }
        }

        /// Simulates the target app: after `delay`, read the text and report
        /// it together with whether the transient marker was present.
        fn spawn_target_reader(
            pasteboard: &PrivatePasteboard,
            delay: Duration,
        ) -> mpsc::Receiver<(Option<String>, bool)> {
            let (sender, receiver) = mpsc::channel();
            let shared = PrivatePasteboard(pasteboard.0);
            std::thread::spawn(move || {
                objc::rc::autoreleasepool(|| {
                    std::thread::sleep(delay);
                    let marked = shared.has_type(TRANSIENT_PASTEBOARD_TYPE);
                    let _ = sender.send((shared.read_string(), marked));
                });
                std::mem::forget(shared); // owned by the test, not the reader
            });
            receiver
        }

        #[test]
        fn promised_paste_is_read_by_target_then_full_clipboard_is_restored() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let before = pasteboard.snapshot();
                let mut reader = None;
                let outcome = run_paste(
                    pasteboard.0,
                    "expanded snippet",
                    Delivery::Promised,
                    quick_timing(),
                    || {
                        reader = Some(spawn_target_reader(&pasteboard, Duration::from_millis(40)));
                        Ok(())
                    },
                )
                .expect("paste");

                let (read, marked) = reader.unwrap().recv().expect("reader result");
                assert_eq!(read.as_deref(), Some("expanded snippet"));
                assert!(marked, "paste item must carry the transient marker");
                assert!(
                    matches!(outcome.consumption, PasteConsumption::Requested { after } if after >= Duration::from_millis(30)),
                    "restore must wait for the target's request, got {:?}",
                    outcome.consumption
                );
                assert_eq!(outcome.restore, RestoreOutcome::Restored);
                assert_eq!(outcome.original.item_count, 2);
                assert!(outcome.original.type_count >= 3);
                assert_original_restored(&pasteboard, &before);
            });
        }

        #[test]
        fn target_that_never_reads_is_reported_and_clipboard_still_restored() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let before = pasteboard.snapshot();
                let timing = PasteTiming {
                    request_timeout: Duration::from_millis(80),
                    ..quick_timing()
                };
                let outcome =
                    run_paste(pasteboard.0, "ignored", Delivery::Promised, timing, || {
                        Ok(())
                    })
                    .expect("paste");
                assert!(matches!(
                    outcome.consumption,
                    PasteConsumption::NotRequested { waited } if waited >= Duration::from_millis(80)
                ));
                assert_eq!(outcome.restore, RestoreOutcome::Restored);
                assert_original_restored(&pasteboard, &before);
            });
        }

        #[test]
        fn unread_text_is_left_on_the_clipboard_when_asked() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let timing = PasteTiming {
                    request_timeout: Duration::from_millis(60),
                    ..quick_timing()
                };
                let outcome = run_paste_with(
                    pasteboard.0,
                    "dictated words",
                    Delivery::Promised,
                    timing,
                    PasteOptions {
                        keep_text_if_unread: true,
                    },
                    || Ok(()),
                )
                .expect("paste");
                assert!(matches!(
                    outcome.consumption,
                    PasteConsumption::NotRequested { .. }
                ));
                assert_eq!(outcome.restore, RestoreOutcome::KeptPastedText);
                assert_eq!(pasteboard.read_string().as_deref(), Some("dictated words"));
                assert!(
                    !pasteboard.has_type(TRANSIENT_PASTEBOARD_TYPE),
                    "kept text is ordinary copied text, recorded by clipboard history"
                );
            });
        }

        #[test]
        fn text_that_was_read_restores_the_clipboard_even_when_keeping_is_asked() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let before = pasteboard.snapshot();
                let mut reader = None;
                let outcome = run_paste_with(
                    pasteboard.0,
                    "read me",
                    Delivery::Promised,
                    quick_timing(),
                    PasteOptions {
                        keep_text_if_unread: true,
                    },
                    || {
                        reader = Some(spawn_target_reader(&pasteboard, Duration::from_millis(20)));
                        Ok(())
                    },
                )
                .expect("paste");
                let (read, _) = reader.unwrap().recv().expect("reader result");
                assert_eq!(read.as_deref(), Some("read me"));
                assert_eq!(outcome.restore, RestoreOutcome::Restored);
                assert_original_restored(&pasteboard, &before);
            });
        }

        #[test]
        fn clipboard_written_by_someone_else_during_paste_is_kept() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let _before = pasteboard.snapshot();
                let outcome = run_paste(
                    pasteboard.0,
                    "snippet",
                    Delivery::Eager,
                    quick_timing(),
                    || {
                        pasteboard.write(&[&[(PLAIN_TEXT_TYPE, b"user copied this")]]);
                        Ok(())
                    },
                )
                .expect("paste");
                assert_eq!(outcome.restore, RestoreOutcome::SkippedExternalChange);
                assert_eq!(
                    pasteboard.read_string().as_deref(),
                    Some("user copied this")
                );
            });
        }

        #[test]
        fn eager_delivery_has_text_ready_before_the_shortcut() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let before = pasteboard.snapshot();
                let mut seen = None;
                let outcome = run_paste(
                    pasteboard.0,
                    "eager text",
                    Delivery::Eager,
                    quick_timing(),
                    || {
                        seen = pasteboard.read_string();
                        Ok(())
                    },
                )
                .expect("paste");
                assert_eq!(seen.as_deref(), Some("eager text"));
                assert_eq!(outcome.consumption, PasteConsumption::Unobserved);
                assert_eq!(outcome.restore, RestoreOutcome::Restored);
                assert_original_restored(&pasteboard, &before);
            });
        }

        #[test]
        fn shortcut_failure_restores_clipboard_then_returns_the_error() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                write_original(&pasteboard);
                let before = pasteboard.snapshot();
                let error = run_paste(
                    pasteboard.0,
                    "x",
                    Delivery::Promised,
                    quick_timing(),
                    || Err(anyhow::anyhow!("no target")),
                )
                .err()
                .expect("shortcut failure must be returned");
                assert!(format!("{error:#}").contains("no target"));
                assert_original_restored(&pasteboard, &before);
            });
        }

        #[test]
        fn empty_clipboard_is_restored_as_empty() {
            objc::rc::autoreleasepool(|| {
                let pasteboard = PrivatePasteboard::new();
                // SAFETY: clear the private pasteboard.
                unsafe {
                    let _: i64 = msg_send![pasteboard.0, clearContents];
                }
                let outcome = run_paste(
                    pasteboard.0,
                    "text",
                    Delivery::Eager,
                    quick_timing(),
                    || Ok(()),
                )
                .expect("paste");
                assert_eq!(outcome.restore, RestoreOutcome::Restored);
                assert_eq!(pasteboard.read_string(), None);
                assert!(!pasteboard.has_type(TRANSIENT_PASTEBOARD_TYPE));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise_is_used_off_the_main_thread_unless_forced_eager() {
        assert_eq!(choose_delivery(false, None), Delivery::Promised);
        assert_eq!(choose_delivery(false, Some("promised")), Delivery::Promised);
        assert_eq!(choose_delivery(false, Some(" EAGER ")), Delivery::Eager);
        assert_eq!(choose_delivery(true, None), Delivery::Eager);
    }
}
