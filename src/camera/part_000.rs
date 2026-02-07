use core_foundation::base::TCFType;
use core_video::pixel_buffer::{CVPixelBuffer, CVPixelBufferRef};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::{c_void, CStr};
use std::sync::{mpsc, Once};
use thiserror::Error;
// CoreVideo pixel format: 420YpCbCr8BiPlanarFullRange (NV12)
// GPUI's Metal renderer requires this specific format for surface()
const K_CV_PIXEL_FORMAT_TYPE_420_YP_CB_CR_8_BI_PLANAR_FULL_RANGE: u32 = 0x34323066; // '420f'

/// Structured webcam startup errors so callers can map failures to specific UI guidance.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WebcamStartError {
    #[error(
        "Camera permission denied (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    PermissionDenied {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Camera device is busy (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    DeviceBusy {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "No camera device available (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    NoDevice {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Failed to initialize camera input (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    InputInitFailed {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Failed to attach camera input (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    CannotAddInput {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Failed to initialize video output (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    OutputInitFailed {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Failed to create webcam callback queue (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    CallbackQueueUnavailable {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Webcam delegate unavailable (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    DelegateClassUnavailable {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
    #[error(
        "Failed to attach video output (attempted={attempted}, failed={failed}, state={state}, details={details})"
    )]
    CannotAddOutput {
        attempted: &'static str,
        failed: &'static str,
        state: &'static str,
        details: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct NSErrorSummary {
    domain: Option<String>,
    code: Option<i64>,
    description: Option<String>,
}
fn classify_input_init_error(summary: NSErrorSummary) -> WebcamStartError {
    let details = format_nserror_details(&summary);
    let description = summary
        .description
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    let domain = summary.domain.as_deref().unwrap_or("");
    let code = summary.code.unwrap_or_default();

    let is_permission_denied = (domain == "AVFoundationErrorDomain" && code == -11852)
        || description.contains("not authorized")
        || description.contains("permission")
        || description.contains("access denied");
    if is_permission_denied {
        return WebcamStartError::PermissionDenied {
            attempted: "create camera input",
            failed: "camera permission check",
            state: "session_created_device_present",
            details,
        };
    }

    let is_device_busy = (domain == "AVFoundationErrorDomain" && code == -11815)
        || description.contains("in use")
        || description.contains("busy")
        || description.contains("another client");
    if is_device_busy {
        return WebcamStartError::DeviceBusy {
            attempted: "create camera input",
            failed: "camera device lock",
            state: "session_created_device_present",
            details,
        };
    }

    let is_no_device = description.contains("no video device") || description.contains("no camera");
    if is_no_device {
        return WebcamStartError::NoDevice {
            attempted: "create camera input",
            failed: "camera device availability",
            state: "session_created_device_present",
            details,
        };
    }

    WebcamStartError::InputInitFailed {
        attempted: "create camera input",
        failed: "camera input initialization",
        state: "session_created_device_present",
        details,
    }
}
fn format_nserror_details(summary: &NSErrorSummary) -> String {
    let domain = summary.domain.as_deref().unwrap_or("unknown-domain");
    let code = summary
        .code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown-code".to_string());
    let description = summary
        .description
        .as_deref()
        .unwrap_or("unknown-description");
    format!("domain={domain}, code={code}, description={description}")
}
unsafe fn nserror_summary(error: *mut Object) -> NSErrorSummary {
    if error.is_null() {
        return NSErrorSummary {
            domain: None,
            code: None,
            description: None,
        };
    }

    let domain_obj: *mut Object = msg_send![error, domain];
    let code: i64 = msg_send![error, code];
    let description_obj: *mut Object = msg_send![error, localizedDescription];

    NSErrorSummary {
        domain: nsstring_to_string(domain_obj),
        code: Some(code),
        description: nsstring_to_string(description_obj),
    }
}
unsafe fn nsstring_to_string(value: *mut Object) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let utf8: *const i8 = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;
    }

    Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
}
/// Handle to a running AVCaptureSession.
///
/// Stops the capture session, drains the dispatch queue to ensure no
/// callbacks are in-flight, then releases all AVFoundation objects and
/// reclaims the boxed sender on drop.
pub struct CaptureHandle {
    session: *mut Object,
    delegate: *mut Object,
    queue: *mut c_void,
    sender_ptr: *mut c_void,
}
// Safety: AVCaptureSession and dispatch queues are thread-safe.
// Raw pointers are only accessed in Drop after synchronous session stop
// and dispatch queue drain.
unsafe impl Send for CaptureHandle {}
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        unsafe {
            // 1. Stop the capture session (synchronous — blocks until fully stopped,
            //    no new callbacks will be dispatched after this returns)
            let _: () = msg_send![self.session, stopRunning];

            // 2. Drain the dispatch queue — ensures any already-dispatched callback
            //    has finished executing before we free the sender
            extern "C" fn noop(_ctx: *mut c_void) {}
            dispatch_sync_f(self.queue, std::ptr::null_mut(), noop);

            // 3. Null out the sender ivar (belt-and-suspenders safety)
            (*self.delegate).set_ivar::<*mut c_void>("_sender", std::ptr::null_mut());

            // 4. Reclaim the boxed sender
            if !self.sender_ptr.is_null() {
                let _ = Box::from_raw(self.sender_ptr as *mut mpsc::SyncSender<CVPixelBuffer>);
            }

            // 5. Release ObjC objects we own (+1 from alloc/init)
            let _: () = msg_send![self.delegate, release];
            let _: () = msg_send![self.session, release];

            // 6. Release the dispatch queue (+1 from dispatch_queue_create)
            dispatch_release(self.queue);
        }
    }
}
unsafe fn cleanup_start_capture_resources(
    session: *mut Object,
    output: *mut Object,
    delegate: *mut Object,
    queue: *mut c_void,
    sender_ptr: *mut c_void,
) {
    if !sender_ptr.is_null() {
        let _ = Box::from_raw(sender_ptr as *mut mpsc::SyncSender<CVPixelBuffer>);
    }

    if !delegate.is_null() {
        let _: () = msg_send![delegate, release];
    }

    if !output.is_null() {
        let _: () = msg_send![output, release];
    }

    if !session.is_null() {
        let _: () = msg_send![session, release];
    }

    if !queue.is_null() {
        dispatch_release(queue);
    }
}
