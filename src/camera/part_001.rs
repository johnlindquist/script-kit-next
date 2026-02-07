/// Start webcam capture. Returns a receiver that yields CVPixelBuffers and
/// a [`CaptureHandle`] that must be kept alive for the duration of capture.
/// Dropping the handle stops capture and releases all resources.
pub fn start_capture(
    width: u32,
) -> std::result::Result<(mpsc::Receiver<CVPixelBuffer>, CaptureHandle), WebcamStartError> {
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
            cleanup_start_capture_resources(
                session,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            return Err(WebcamStartError::NoDevice {
                attempted: "lookup default video device",
                failed: "camera device lookup",
                state: "session_created",
                details: "AVCaptureDevice::defaultDeviceWithMediaType returned null".to_string(),
            });
        }

        // Create input (convenience constructor — autoreleased, not ours to release)
        let mut error: *mut Object = std::ptr::null_mut();
        let input: *mut Object = msg_send![
            class!(AVCaptureDeviceInput),
            deviceInputWithDevice: device
            error: &mut error
        ];
        if input.is_null() {
            let summary = nserror_summary(error);
            cleanup_start_capture_resources(
                session,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            return Err(classify_input_init_error(summary));
        }

        let can_add_input: BOOL = msg_send![session, canAddInput: input];
        if can_add_input != YES {
            cleanup_start_capture_resources(
                session,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            return Err(WebcamStartError::CannotAddInput {
                attempted: "add camera input to session",
                failed: "session.canAddInput",
                state: "session_created_input_ready",
                details: "AVCaptureSession rejected AVCaptureDeviceInput".to_string(),
            });
        }
        let _: () = msg_send![session, addInput: input];

        // Create video data output (+1 retained)
        let output: *mut Object = msg_send![class!(AVCaptureVideoDataOutput), alloc];
        let output: *mut Object = msg_send![output, init];
        if output.is_null() {
            cleanup_start_capture_resources(
                session,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            return Err(WebcamStartError::OutputInitFailed {
                attempted: "create AVCaptureVideoDataOutput",
                failed: "video output init",
                state: "session_created_input_attached",
                details: "AVCaptureVideoDataOutput alloc/init returned null".to_string(),
            });
        }

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
        if queue.is_null() {
            cleanup_start_capture_resources(
                session,
                output,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            return Err(WebcamStartError::CallbackQueueUnavailable {
                attempted: "create dispatch callback queue",
                failed: "dispatch_queue_create",
                state: "session_created_input_output_ready",
                details: "dispatch_queue_create returned null".to_string(),
            });
        }

        // Create delegate (+1 retained), store sender
        let delegate_class = match Class::get("SKWebcamDelegate") {
            Some(cls) => cls,
            None => {
                cleanup_start_capture_resources(
                    session,
                    output,
                    std::ptr::null_mut(),
                    queue,
                    std::ptr::null_mut(),
                );
                return Err(WebcamStartError::DelegateClassUnavailable {
                    attempted: "lookup SKWebcamDelegate class",
                    failed: "delegate class registration",
                    state: "session_created_input_output_queue_ready",
                    details: "Class::get(\"SKWebcamDelegate\") returned None".to_string(),
                });
            }
        };
        let delegate: *mut Object = msg_send![delegate_class, alloc];
        let delegate: *mut Object = msg_send![delegate, init];
        if delegate.is_null() {
            cleanup_start_capture_resources(
                session,
                output,
                std::ptr::null_mut(),
                queue,
                std::ptr::null_mut(),
            );
            return Err(WebcamStartError::DelegateClassUnavailable {
                attempted: "create SKWebcamDelegate instance",
                failed: "delegate init",
                state: "session_created_input_output_queue_ready",
                details: "SKWebcamDelegate alloc/init returned null".to_string(),
            });
        }

        // Store the sender as a raw pointer in the delegate
        let tx_box = Box::new(tx);
        let sender_ptr = Box::into_raw(tx_box) as *mut c_void;
        (*delegate).set_ivar::<*mut c_void>("_sender", sender_ptr);

        let _: () =
            msg_send![output, setSampleBufferDelegate: delegate queue: queue as *mut Object];

        let can_add_output: BOOL = msg_send![session, canAddOutput: output];
        if can_add_output != YES {
            cleanup_start_capture_resources(session, output, delegate, queue, sender_ptr);
            return Err(WebcamStartError::CannotAddOutput {
                attempted: "add camera output to session",
                failed: "session.canAddOutput",
                state: "session_created_input_output_configured",
                details: "AVCaptureSession rejected AVCaptureVideoDataOutput".to_string(),
            });
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_input_error_returns_permission_denied_when_not_authorized() {
        let summary = NSErrorSummary {
            domain: Some("AVFoundationErrorDomain".to_string()),
            code: Some(-11852),
            description: Some("The operation couldn’t be completed. Not authorized".to_string()),
        };

        let error = classify_input_init_error(summary);
        assert!(matches!(error, WebcamStartError::PermissionDenied { .. }));
    }

    #[test]
    fn test_classify_input_error_returns_device_busy_when_in_use() {
        let summary = NSErrorSummary {
            domain: Some("AVFoundationErrorDomain".to_string()),
            code: Some(-11815),
            description: Some("The video device is in use by another client".to_string()),
        };

        let error = classify_input_init_error(summary);
        assert!(matches!(error, WebcamStartError::DeviceBusy { .. }));
    }

    #[test]
    fn test_classify_input_error_returns_generic_with_context_when_unknown() {
        let summary = NSErrorSummary {
            domain: Some("CustomCameraDomain".to_string()),
            code: Some(42),
            description: Some("Unknown failure from test camera stack".to_string()),
        };

        let error = classify_input_init_error(summary);
        match error {
            WebcamStartError::InputInitFailed {
                attempted,
                failed,
                state,
                details,
            } => {
                assert_eq!(attempted, "create camera input");
                assert_eq!(failed, "camera input initialization");
                assert_eq!(state, "session_created_device_present");
                assert!(details.contains("CustomCameraDomain"));
                assert!(details.contains("Unknown failure"));
            }
            other => panic!("Expected InputInitFailed, got {other:?}"),
        }
    }
}
