//! Local-only providers. Native work starts after microphone admission. At most
//! one slow AX/Vision worker can occupy resources, even across rapid cancellation.
use crate::dictation::screen_spelling::{terms, Evidence, Owner, Session};
use crate::platform::accessibility::ax::OwnedAxElement;
use crate::runtime_policy::{check, ExternalEffect};
use core_foundation::base::{CFRelease, CFRetain, CFTypeRef};
use std::{
    ffi::{c_char, CStr, CString},
    sync::atomic::{AtomicBool, Ordering},
};

extern "C" {
    fn sk_spelling_original(pid: i32, wid: u32, field: CFTypeRef) -> bool;
    fn sk_spelling_excerpt(
        pid: i32,
        wid: u32,
        field: CFTypeRef,
        out: *mut c_char,
        cap: usize,
    ) -> bool;
    fn sk_spelling_names(input: *const c_char, out: *mut c_char, cap: usize);
}

// An AXUIElement is a CF reference to a remote element. Retain it across the
// handoff; the worker only messages/compares it, never mutates or refocuses it.
struct Field(CFTypeRef);
unsafe impl Send for Field {}
impl Drop for Field {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) };
    }
}
impl Field {
    fn matches(&self, owner: Owner) -> bool {
        check(ExternalEffect::SystemDiscovery).is_ok()
            && unsafe { sk_spelling_original(owner.pid, owner.window, self.0) }
    }
    fn excerpt(&self, owner: Owner) -> Result<String, &'static str> {
        check(ExternalEffect::SystemDiscovery).map_err(|_| "unavailable")?;
        let mut buf = [0i8; 16385];
        if !unsafe {
            sk_spelling_excerpt(owner.pid, owner.window, self.0, buf.as_mut_ptr(), buf.len())
        } {
            return Err("unsafeOrChanged");
        }
        Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .into_owned())
    }
}

fn names(text: &str) -> Vec<String> {
    let Ok(c) = CString::new(text) else {
        return Vec::new();
    };
    let mut buf = [0i8; 4096];
    unsafe {
        sk_spelling_names(c.as_ptr(), buf.as_mut_ptr(), buf.len());
    }
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .lines()
        .take(32)
        .map(str::to_owned)
        .collect()
}

#[cfg(feature = "ocr")]
fn pixels(owner: Owner, field: &Field, live: impl Fn() -> bool) -> Result<String, &'static str> {
    if !live()
        || check(ExternalEffect::ScreenCapture).is_err()
        || super::screen_capture_access_preflight() != Some(true)
    {
        return Err("screenCaptureUnavailable");
    }
    let window = xcap::Window::all()
        .map_err(|_| "windowUnavailable")?
        .into_iter()
        .find(|w| w.id().ok() == Some(owner.window) && w.pid().ok() == Some(owner.pid as u32))
        .ok_or("windowUnavailable")?;
    if !matches!(window.width().ok().zip(window.height().ok()), Some((w,h)) if u64::from(w)*u64::from(h) <= 12_000_000)
    {
        return Err("windowTooLarge");
    }
    if !live() || !field.matches(owner) {
        return Err("unsafeOrChanged");
    }
    // xcap on macOS uses OptionIncludingWindow with this ID, never a display fallback.
    let img = window.capture_image().map_err(|_| "captureFailed")?;
    if !live() || !field.matches(owner) {
        return Err("unsafeOrChanged");
    }
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 32_000_000 {
        return Err("windowTooLarge");
    }
    let img = if w.max(h) > 1800 {
        let scale = 1800.0 / w.max(h) as f64;
        image::imageops::resize(
            &img,
            ((w as f64 * scale) as u32).max(1),
            ((h as f64 * scale) as u32).max(1),
            image::imageops::FilterType::Triangle,
        )
    } else {
        img
    };
    crate::ocr::extract_dictation_text(img.width(), img.height(), img.as_raw())
        .map_err(|_| "ocrUnavailable")
}
#[cfg(not(feature = "ocr"))]
fn pixels(_: Owner, _: &Field, _: impl Fn() -> bool) -> Result<String, &'static str> {
    Err("ocrUnavailable")
}

// Use OCR only when AX supplied no useful vocabulary. App-name evidence must
// not by itself suppress OCR. A failed security check is not an empty AX read.
fn collect(
    excerpt: Result<String, &'static str>,
    app: &str,
    personal: &[String],
    allow_ocr: bool,
    ocr: impl FnOnce() -> Result<String, &'static str>,
) -> Evidence {
    let text = match excerpt {
        Ok(text) => text,
        Err(reason) => return Evidence::Skipped(reason),
    };
    let extract = |s: &str| {
        let s = if s.len() > 4096 {
            let prefix = &s[..s.floor_char_boundary(4096)];
            &prefix[..prefix.rfind(|c: char| !c.is_alphanumeric()).unwrap_or(0)]
        } else {
            s
        };
        if crate::clipboard_history::matches_secret_content(s) {
            return Err("sensitiveText");
        }
        let found = names(s);
        Ok((terms(s, &found, personal), found))
    };
    let (list, mut found) = match extract(&text) {
        Ok(v) => v,
        Err(reason) => return Evidence::Skipped(reason),
    };
    let (source, text) = if list.is_empty() && allow_ocr {
        let text = match ocr() {
            Ok(text) => text,
            Err(reason) => return Evidence::Skipped(reason),
        };
        found = match extract(&text) {
            Ok((_, names)) => names,
            Err(reason) => return Evidence::Skipped(reason),
        };
        ("ocr", text)
    } else {
        ("accessibility", text)
    };
    found.insert(0, app.to_owned());
    let list = terms(&text, &found, personal);
    if list.is_empty() {
        Evidence::Skipped("noTerms")
    } else {
        Evidence::ready(source, &list)
    }
}

#[cfg(test)]
#[path = "dictation_spelling_tests.rs"]
mod tests;

static BUSY: AtomicBool = AtomicBool::new(false);
struct WorkerSlot;
impl Drop for WorkerSlot {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}

pub(crate) fn start(
    owner: Owner,
    field: &OwnedAxElement,
    bundle: &str,
    app: &str,
    mode: &str,
    personal: Vec<String>,
) -> Option<Session> {
    let allow_ocr = match mode {
        "accessibility" => false,
        "accessibilityAndOcr" => true,
        _ => return None,
    };
    let (s, p) = Session::new(owner);
    if std::env::var_os("SCRIPT_KIT_NONINTERACTIVE").is_some_and(|v| v != "0")
        || check(ExternalEffect::SystemDiscovery).is_err()
        || bundle.is_empty()
        || crate::clipboard_history::matches_blocked_source_app(bundle)
    {
        p.publish(Evidence::Skipped("excluded"));
        return Some(s);
    }
    if BUSY
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        p.publish(Evidence::Skipped("busy"));
        return Some(s);
    }
    let slot = WorkerSlot;
    let field = Field(unsafe { CFRetain(field.as_ptr()) });
    let app = app.to_owned();
    let spawn = std::thread::Builder::new()
        .name("dictation-screen-spelling".into())
        .spawn(move || {
            let _slot = slot;
            objc::rc::autoreleasepool(|| {
                if !p.live() {
                    return;
                }
                let result = collect(field.excerpt(owner), &app, &personal, allow_ocr, || {
                    pixels(owner, &field, || p.live())
                });
                if p.live() {
                    p.publish(if field.matches(owner) {
                        result
                    } else {
                        Evidence::Skipped("unsafeOrChanged")
                    });
                }
            });
        });
    if spawn.is_err() {
        s.unavailable("workerUnavailable");
    }
    Some(s)
}
