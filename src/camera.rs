//! Zero-copy webcam capture via AVFoundation.
//!
//! Captures camera frames as CVPixelBuffer and sends them directly to
//! GPUI's `surface()` element — no CPU pixel conversion, no copies.
//!
//! The returned [`CaptureHandle`] owns all AVFoundation resources and
//! stops capture + releases everything on drop.

use anyhow::{anyhow, Result};
use core_foundation::base::TCFType;
use core_video::pixel_buffer::{CVPixelBuffer, CVPixelBufferRef};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::sync::mpsc;
use std::sync::Once;

// CoreVideo pixel format: 420YpCbCr8BiPlanarFullRange (NV12)
// GPUI's Metal renderer requires this specific format for surface()
const K_CV_PIXEL_FORMAT_TYPE_420_YP_CB_CR_8_BI_PLANAR_FULL_RANGE: u32 = 0x34323066; // '420f'

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

/// Start webcam capture. Returns a receiver that yields CVPixelBuffers and
/// a [`CaptureHandle`] that must be kept alive for the duration of capture.
/// Dropping the handle stops capture and releases all resources.
pub fn start_capture(width: u32) -> Result<(mpsc::Receiver<CVPixelBuffer>, CaptureHandle)> {
    let (tx, rx) = mpsc::sync_channel::<CVPixelBuffer>(1);

    // Register our delegate class (once)
    static REGISTER: Once = Once::new();
    REGISTER.call_once(register_delegate_class);

    unsafe {
        // Create capture session (+1 retained)
        let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
        let session: *mut Object = msg_send![session, init];

        // Set session preset
        let preset_cstr = if width <= 640 {
            c"AVCaptureSessionPreset640x480"
        } else {
            c"AVCaptureSessionPresetHigh"
        };
        let preset: *mut Object =
            msg_send![class!(NSString), stringWithUTF8String: preset_cstr.as_ptr()];
        let can_set: BOOL = msg_send![session, canSetSessionPreset: preset];
        if can_set == YES {
            let _: () = msg_send![session, setSessionPreset: preset];
        }

        // Get default video capture device
        let device: *mut Object = msg_send![
            class!(AVCaptureDevice),
            defaultDeviceWithMediaType: av_media_type_video()
        ];
        if device.is_null() {
            let _: () = msg_send![session, release];
            return Err(anyhow!("No camera found"));
        }

        // Create input (convenience constructor — autoreleased, not ours to release)
        let mut error: *mut Object = std::ptr::null_mut();
        let input: *mut Object = msg_send![
            class!(AVCaptureDeviceInput),
            deviceInputWithDevice: device
            error: &mut error
        ];
        if input.is_null() {
            let _: () = msg_send![session, release];
            return Err(anyhow!("Failed to create camera input"));
        }

        let can_add_input: BOOL = msg_send![session, canAddInput: input];
        if can_add_input != YES {
            let _: () = msg_send![session, release];
            return Err(anyhow!("Cannot add camera input to session"));
        }
        let _: () = msg_send![session, addInput: input];

        // Create video data output (+1 retained)
        let output: *mut Object = msg_send![class!(AVCaptureVideoDataOutput), alloc];
        let output: *mut Object = msg_send![output, init];

        // Set pixel format to NV12 (420f) — required by GPUI's Metal renderer
        let format_num: *mut Object = msg_send![
            class!(NSNumber),
            numberWithUnsignedInt: K_CV_PIXEL_FORMAT_TYPE_420_YP_CB_CR_8_BI_PLANAR_FULL_RANGE
        ];

        let pixel_format_key = kCVPixelBufferPixelFormatTypeKey();
        let video_settings: *mut Object = msg_send![
            class!(NSDictionary),
            dictionaryWithObject: format_num
            forKey: pixel_format_key
        ];
        let _: () = msg_send![output, setVideoSettings: video_settings];
        let _: () = msg_send![output, setAlwaysDiscardsLateVideoFrames: YES];

        // Create dispatch queue for callbacks (+1 retained)
        let queue_label = c"com.scriptkit.webcam".as_ptr();
        let queue: *mut c_void = dispatch_queue_create(queue_label, std::ptr::null_mut());

        // Create delegate (+1 retained), store sender
        let delegate_class = match Class::get("SKWebcamDelegate") {
            Some(cls) => cls,
            None => {
                let _: () = msg_send![output, release];
                let _: () = msg_send![session, release];
                dispatch_release(queue);
                return Err(anyhow!("Webcam delegate class not registered"));
            }
        };
        let delegate: *mut Object = msg_send![delegate_class, alloc];
        let delegate: *mut Object = msg_send![delegate, init];

        // Store the sender as a raw pointer in the delegate
        let tx_box = Box::new(tx);
        let sender_ptr = Box::into_raw(tx_box) as *mut c_void;
        (*delegate).set_ivar::<*mut c_void>("_sender", sender_ptr);

        let _: () =
            msg_send![output, setSampleBufferDelegate: delegate queue: queue as *mut Object];

        let can_add_output: BOOL = msg_send![session, canAddOutput: output];
        if can_add_output != YES {
            // Clean up on error
            (*delegate).set_ivar::<*mut c_void>("_sender", std::ptr::null_mut());
            let _ = Box::from_raw(sender_ptr as *mut mpsc::SyncSender<CVPixelBuffer>);
            let _: () = msg_send![delegate, release];
            let _: () = msg_send![output, release];
            let _: () = msg_send![session, release];
            dispatch_release(queue);
            return Err(anyhow!("Cannot add video output to session"));
        }
        let _: () = msg_send![session, addOutput: output];

        // Release output — session retains it, we don't need it after this
        let _: () = msg_send![output, release];

        // Start capturing
        let _: () = msg_send![session, startRunning];

        let handle = CaptureHandle {
            session,
            delegate,
            queue,
            sender_ptr,
        };

        Ok((rx, handle))
    }
}

/// Register our Objective-C delegate class for receiving camera frames.
/// If registration fails (e.g. class name collision), start_capture will
/// return an error when it tries to look up the class.
fn register_delegate_class() {
    let superclass = class!(NSObject);
    let Some(mut decl) = ClassDecl::new("SKWebcamDelegate", superclass) else {
        return;
    };

    decl.add_ivar::<*mut c_void>("_sender");

    unsafe {
        decl.add_method(
            sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
            capture_callback
                as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
        );
    }

    decl.register();
}

/// AVCaptureVideoDataOutputSampleBufferDelegate callback.
/// Extracts CVPixelBuffer from the sample buffer and sends it through the channel.
extern "C" fn capture_callback(
    this: &mut Object,
    _sel: Sel,
    _output: *mut Object,
    sample_buffer: *mut Object,
    _connection: *mut Object,
) {
    unsafe {
        let pixel_buffer_ref: CVPixelBufferRef = CMSampleBufferGetImageBuffer(sample_buffer as _);
        if pixel_buffer_ref.is_null() {
            return;
        }

        // Retain the CVPixelBuffer (wrap_under_get_rule calls CFRetain)
        let pixel_buffer = CVPixelBuffer::wrap_under_get_rule(pixel_buffer_ref);

        // Send through channel — try_send is intentional: if the channel is full
        // (consumer hasn't drained the previous frame yet), we drop this frame.
        // This is the desired behavior for real-time video: always show the latest
        // frame rather than queuing up stale ones.
        let sender_ptr = *this.get_ivar::<*mut c_void>("_sender");
        if !sender_ptr.is_null() {
            let sender = &*(sender_ptr as *const mpsc::SyncSender<CVPixelBuffer>);
            let _ = sender.try_send(pixel_buffer);
        }
    }
}

// External declarations
extern "C" {
    fn CMSampleBufferGetImageBuffer(sbuf: *mut c_void) -> CVPixelBufferRef;
    fn dispatch_queue_create(label: *const i8, attr: *mut c_void) -> *mut c_void;
    fn dispatch_release(object: *mut c_void);
    fn dispatch_sync_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

/// Get AVMediaTypeVideo string
fn av_media_type_video() -> *mut Object {
    unsafe {
        let s: *mut Object = msg_send![class!(NSString), stringWithUTF8String: c"vide".as_ptr()];
        s
    }
}

/// Get kCVPixelBufferPixelFormatTypeKey
#[allow(non_snake_case)]
fn kCVPixelBufferPixelFormatTypeKey() -> *mut Object {
    extern "C" {
        static kCVPixelBufferPixelFormatTypeKey: *mut Object;
    }
    unsafe { kCVPixelBufferPixelFormatTypeKey }
}
