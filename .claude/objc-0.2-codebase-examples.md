# Objective-C 0.2: Real Examples from Script Kit GPUI

This document shows actual patterns from the codebase. Copy-paste these patterns.

---

## Example 1: Complete Ownership Lifecycle (Camera Module)

**File**: `src/camera/mod.rs`

This is the **gold standard** for objc 0.2 memory management in Script Kit.

### Creating and Owning Objects

```rust
pub fn start_capture(width: u32) -> std::result::Result<
    (mpsc::Receiver<CVPixelBuffer>, CaptureHandle),
    WebcamStartError,
> {
    let (tx, rx) = mpsc::sync_channel::<CVPixelBuffer>(1);

    unsafe {
        // Pattern: alloc + init = we own this (+1 retain)
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

        // Check for success
        let can_set: BOOL = msg_send![session, canSetSessionPreset: preset];
        if can_set == YES {
            let _: () = msg_send![session, setSessionPreset: preset];
        }

        // ... more setup ...

        // Pattern: convenience constructor = autoreleased, don't release
        let device: *mut Object = msg_send![
            class!(AVCaptureDevice),
            defaultDeviceWithMediaType: av_media_type_video()
        ];
        if device.is_null() {
            cleanup_start_capture_resources(session, ...);
            return Err(WebcamStartError::NoDevice { ... });
        }

        // Pattern: autoreleased convenience method, error output parameter
        let mut error: *mut Object = std::ptr::null_mut();
        let input: *mut Object = msg_send![
            class!(AVCaptureDeviceInput),
            deviceInputWithDevice: device
            error: &mut error
        ];
        if input.is_null() {
            let summary = nserror_summary(error);
            cleanup_start_capture_resources(session, ...);
            return Err(classify_input_init_error(summary));
        }

        // ... more setup ...

        // Pattern: Create owned object
        let output: *mut Object = msg_send![class!(AVCaptureVideoDataOutput), alloc];
        let output: *mut Object = msg_send![output, init];
        if output.is_null() {
            cleanup_start_capture_resources(session, ...);
            return Err(WebcamStartError::OutputInitFailed { ... });
        }

        // ... configure output ...

        // Pattern: Create dispatch queue (owns +1)
        let queue_label = c"com.scriptkit.webcam".as_ptr();
        let queue: *mut c_void = dispatch_queue_create(queue_label, std::ptr::null_mut());
        if queue.is_null() {
            cleanup_start_capture_resources(session, output, ...);
            return Err(WebcamStartError::CallbackQueueUnavailable { ... });
        }

        // Pattern: Create delegate (owns +1)
        let delegate_class = match Class::get("SKWebcamDelegate") {
            Some(cls) => cls,
            None => {
                cleanup_start_capture_resources(session, output, ...);
                return Err(WebcamStartError::DelegateClassUnavailable { ... });
            }
        };
        let delegate: *mut Object = msg_send![delegate_class, alloc];
        let delegate: *mut Object = msg_send![delegate, init];
        if delegate.is_null() {
            cleanup_start_capture_resources(session, output, ...);
            return Err(WebcamStartError::DelegateClassUnavailable { ... });
        }

        // Pattern: Store raw pointers in struct
        let tx_box = Box::new(tx);
        let sender_ptr = Box::into_raw(tx_box) as *mut c_void;
        (*delegate).set_ivar::<*mut c_void>("_sender", sender_ptr);

        // ... more setup ...

        // Pattern: Add output to session (session retains it)
        let _: () = msg_send![output, setSampleBufferDelegate: delegate queue: queue as *mut Object];
        let _: () = msg_send![session, addOutput: output];

        // Pattern: Release our reference because session now owns it
        let _: () = msg_send![output, release];

        // Pattern: Start the session
        let _: () = msg_send![session, startRunning];

        // Pattern: Return handle that owns the resources
        let handle = CaptureHandle {
            session,
            delegate,
            queue,
            sender_ptr,
        };

        Ok((rx, handle))
    }
}
```

### Cleanup Helper Function

```rust
unsafe fn cleanup_start_capture_resources(
    session: *mut Object,
    output: *mut Object,
    delegate: *mut Object,
    queue: *mut c_void,
    sender_ptr: *mut c_void,
) {
    // Pattern: Reclaim boxed sender
    if !sender_ptr.is_null() {
        let _ = Box::from_raw(sender_ptr as *mut mpsc::SyncSender<CVPixelBuffer>);
    }

    // Pattern: Release objects we owned
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
```

### Drop Implementation (Safe Cleanup)

```rust
pub struct CaptureHandle {
    session: *mut Object,
    delegate: *mut Object,
    queue: *mut c_void,
    sender_ptr: *mut c_void,
}

unsafe impl Send for CaptureHandle {}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        unsafe {
            // Pattern 1: Stop the capture session (synchronous)
            let _: () = msg_send![self.session, stopRunning];

            // Pattern 2: Drain the dispatch queue synchronously
            extern "C" fn noop(_ctx: *mut c_void) {}
            dispatch_sync_f(self.queue, std::ptr::null_mut(), noop);

            // Pattern 3: Null out the sender ivar for safety
            (*self.delegate).set_ivar::<*mut c_void>("_sender", std::ptr::null_mut());

            // Pattern 4: Reclaim the boxed sender
            if !self.sender_ptr.is_null() {
                let _ = Box::from_raw(self.sender_ptr as *mut mpsc::SyncSender<CVPixelBuffer>);
            }

            // Pattern 5: Release ObjC objects we own
            let _: () = msg_send![self.delegate, release];
            let _: () = msg_send![self.session, release];

            // Pattern 6: Release the dispatch queue
            dispatch_release(self.queue);
        }
    }
}
```

### Delegate Class Registration

```rust
fn register_delegate_class() {
    let superclass = class!(NSObject);
    let Some(mut decl) = ClassDecl::new("SKWebcamDelegate", superclass) else {
        return;
    };

    // Pattern: Add instance variable for storing data
    decl.add_ivar::<*mut c_void>("_sender");

    // Pattern: Add callback method
    unsafe {
        decl.add_method(
            sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
            capture_callback
                as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
        );
    }

    decl.register();
}

// Pattern: Callback signature with Sel parameter
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

        let pixel_buffer = CVPixelBuffer::wrap_under_get_rule(pixel_buffer_ref);

        // Pattern: Retrieve stored data from instance variable
        let sender_ptr = *this.get_ivar::<*mut c_void>("_sender");
        if !sender_ptr.is_null() {
            let sender = &*(sender_ptr as *const mpsc::SyncSender<CVPixelBuffer>);
            let _ = sender.try_send(pixel_buffer);
        }
    }
}
```

### NSString Extraction Helper

```rust
unsafe fn nsstring_to_string(value: *mut Object) -> Option<String> {
    if value.is_null() {
        return None;
    }

    // Pattern: Get UTF8 C string
    let utf8: *const i8 = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;
    }

    // Pattern: Convert to Rust String
    Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
}

// Usage in error extraction
unsafe fn nserror_summary(error: *mut Object) -> NSErrorSummary {
    if error.is_null() {
        return NSErrorSummary {
            domain: None,
            code: None,
            description: None,
        };
    }

    // Pattern: Extract fields via msg_send!
    let domain_obj: *mut Object = msg_send![error, domain];
    let code: i64 = msg_send![error, code];
    let description_obj: *mut Object = msg_send![error, localizedDescription];

    NSErrorSummary {
        domain: nsstring_to_string(domain_obj),
        code: Some(code),
        description: nsstring_to_string(description_obj),
    }
}
```

---

## Example 2: Main Thread Safety (Window Management)

**File**: `src/platform/app_window_management.rs`

### Thread Check Guard

```rust
fn is_main_thread() -> bool {
    // Pattern: Check main thread
    unsafe {
        let is_main: bool = msg_send![class!(NSThread), isMainThread];
        is_main
    }
}

fn require_main_thread(fn_name: &str) -> bool {
    if !is_main_thread() {
        logging::log(
            "ERROR",
            &format!(
                "{} called from non-main thread; AppKit requires main thread",
                fn_name
            ),
        );
        return true;
    }
    false
}
```

### Safe AppKit Method Wrapper

```rust
pub fn configure_as_accessory_app() {
    // Pattern: Guard off-main-thread calls
    if require_main_thread("configure_as_accessory_app") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid after app launch.
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyAccessory = 1
        let _: () = msg_send![
            app,
            setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY
        ];
        logging::log(
            "PANEL",
            "Configured app as accessory (no Dock icon, no menu bar ownership)",
        );
    }
}
```

### Deferred Main Thread Execution

```rust
// Pattern: Defer ObjC calls from GPUI context
pub fn defer_hide_main_window(cx: &mut gpui::App) {
    cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
        hide_main_window();
    })
    .detach();
}

fn hide_main_window() {
    if require_main_thread("hide_main_window") {
        return;
    }
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log("PANEL", "hide_main_window: Main window not registered");
                return;
            }
        };

        let _: () = msg_send![window, orderOut: nil];
        logging::log("PANEL", "Main window hidden via orderOut:");
    }
}
```

---

## Example 3: Recursive View Traversal with Type Checking

**File**: `src/platform/vibrancy_config.rs`

```rust
unsafe fn configure_visual_effect_views_recursive(
    view: id,
    count: &mut usize,
    is_dark: bool,
) {
    // Pattern: Check if view is specific type
    let is_vev: bool = msg_send![view, isKindOfClass: class!(NSVisualEffectView)];
    if is_vev {
        // Pattern: Get integer properties
        let old_material: isize = msg_send![view, material];
        let old_state: isize = msg_send![view, state];
        let old_blending: isize = msg_send![view, blendingMode];
        let old_emphasized: bool = msg_send![view, isEmphasized];

        // Pattern: Set object property (nil is ok)
        let view_appearance: id = if is_dark {
            msg_send![
                class!(NSAppearance),
                appearanceNamed: NSAppearanceNameVibrantDark
            ]
        } else {
            msg_send![
                class!(NSAppearance),
                appearanceNamed: NSAppearanceNameVibrantLight
            ]
        };
        if !view_appearance.is_null() {
            let _: () = msg_send![view, setAppearance: view_appearance];
        }

        // Pattern: Set multiple properties
        let material = if is_dark {
            ns_visual_effect_material::HUD_WINDOW
        } else {
            ns_visual_effect_material::POPOVER
        };
        let _: () = msg_send![view, setMaterial: material];
        let state = if is_dark { 1isize } else { 0isize };
        let _: () = msg_send![view, setState: state];
        let _: () = msg_send![view, setBlendingMode: 0isize];
        let _: () = msg_send![view, setEmphasized: is_dark];

        // Pattern: Read state after change
        let new_material: isize = msg_send![view, material];
        let new_state: isize = msg_send![view, state];
        let new_blending: isize = msg_send![view, blendingMode];
        let new_emphasized: bool = msg_send![view, isEmphasized];

        *count += 1;
    }

    // Pattern: Recurse into subviews
    let subviews: id = msg_send![view, subviews];
    if !subviews.is_null() {
        let subview_count: usize = msg_send![subviews, count];
        for i in 0..subview_count {
            let child: id = msg_send![subviews, objectAtIndex: i];
            configure_visual_effect_views_recursive(child, count, is_dark);
        }
    }
}
```

---

## Example 4: Cursor Swizzling with dlsym

**File**: `src/platform/cursor.rs`

### Loading Private APIs Dynamically

```rust
pub type SetWindowTagsFn =
    unsafe extern "C" fn(CGSConnectionID, CGWindowID, *const CGSWindowTagBit, usize) -> CGError;

fn load_set_window_tags_fn() -> Option<SetWindowTagsFn> {
    static FN: OnceLock<Option<SetWindowTagsFn>> = OnceLock::new();

    *FN.get_or_init(|| unsafe {
        // Pattern: Try multiple symbol names
        for name in [b"CGSSetWindowTags\0", b"SLSSetWindowTags\0"] {
            let ptr = dlsym(RTLD_DEFAULT, name.as_ptr() as *const c_char);
            if !ptr.is_null() {
                return Some(std::mem::transmute::<*mut c_void, SetWindowTagsFn>(ptr));
            }
        }
        None
    })
}
```

### Method Swizzling

```rust
fn install_cursor_rects() {
    unsafe {
        let window = match crate::window_manager::get_main_window() {
            Some(w) => w,
            None => return,
        };

        let content_view: cocoa::base::id = msg_send![window, contentView];
        if content_view.is_null() {
            return;
        }

        let view_class: *const std::ffi::c_void = msg_send![content_view, class];
        if view_class.is_null() {
            return;
        }

        // Pattern: Implement method in Rust
        extern "C" fn reset_cursor_rects_impl(
            this: *mut std::ffi::c_void,
            _cmd: objc::runtime::Sel,
        ) {
            unsafe {
                let this = this as cocoa::base::id;
                let bounds: cocoa::foundation::NSRect = msg_send![this, bounds];
                let arrow: cocoa::base::id = msg_send![class!(NSCursor), arrowCursor];
                let _: () = msg_send![this, addCursorRect: bounds cursor: arrow];
            }
        }

        let method_sel = sel!(resetCursorRects);
        let new_imp: objc::runtime::Imp = std::mem::transmute::<
            _,
            objc::runtime::Imp,
        >(reset_cursor_rects_impl
            as extern "C" fn(*mut std::ffi::c_void, objc::runtime::Sel));

        // Pattern: Try add first, fall back to swizzle
        let existing = objc::runtime::class_getInstanceMethod(
            view_class as *mut objc::runtime::Class,
            method_sel,
        );

        if existing.is_null() {
            let types = c"v@:";
            let _added = objc::runtime::class_addMethod(
                view_class as *mut objc::runtime::Class,
                method_sel,
                new_imp,
                types.as_ptr(),
            );
        } else {
            let _old = objc::runtime::method_setImplementation(
                existing as *mut objc::runtime::Method,
                new_imp,
            );
        }

        let _: () = msg_send![window, invalidateCursorRectsForView: content_view];
    }
}
```

---

## Example 5: Window Iteration with NSArray

**File**: `src/platform/app_window_management.rs`

```rust
pub fn send_ai_window_to_back() {
    if require_main_thread("send_ai_window_to_back") {
        return;
    }
    // SAFETY: Main thread verified. NSApp() is always valid.
    unsafe {
        use std::ffi::CStr;

        let app: id = NSApp();

        // Pattern: Get array from object
        let windows: id = msg_send![app, windows];
        if windows.is_null() {
            return;
        }

        // Pattern: Get count from array
        let count: usize = msg_send![windows, count];

        // Pattern: Iterate with bounds
        for i in 0..count {
            // Pattern: Get element by index
            let window: id = msg_send![windows, objectAtIndex: i];

            // Pattern: Get object property
            let title: id = msg_send![window, title];

            if title != nil {
                // Pattern: Extract string from object
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Pattern: Call method with nil argument
                        let _: () = msg_send![window, orderBack: nil];
                        logging::log("PANEL", "AI window sent to back");
                        return;
                    }
                }
            }
        }
    }
}
```

---

## Example 6: Share Sheet with Multiple Item Types

**File**: `src/platform/visibility_focus.rs`

```rust
pub fn show_share_sheet(item: ShareSheetItem) {
    if require_main_thread("show_share_sheet") {
        return;
    }
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log("PANEL", "show_share_sheet: Main window not registered");
                return;
            }
        };

        let content_view: id = msg_send![window, contentView];
        if content_view == nil {
            logging::log("PANEL", "show_share_sheet: contentView is nil");
            return;
        }

        // Pattern: Match on enum and create appropriate ObjC objects
        let share_item: id = match item {
            ShareSheetItem::Text(text) => {
                // Pattern: Create NSString from Rust String
                let ns_string = CocoaNSString::alloc(nil).init_str(&text);
                if ns_string == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSString");
                    return;
                }
                ns_string
            }
            ShareSheetItem::ImagePng(png_bytes) => {
                if png_bytes.is_empty() {
                    logging::log("PANEL", "show_share_sheet: Empty PNG data");
                    return;
                }

                // Pattern: Create NSData from byte slice
                let data: id = msg_send![
                    class!(NSData),
                    dataWithBytes: png_bytes.as_ptr()
                    length: png_bytes.len()
                ];
                if data == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSData");
                    return;
                }

                // Pattern: Chain alloc + init
                let image: id = msg_send![class!(NSImage), alloc];
                let image: id = msg_send![image, initWithData: data];
                if image == nil {
                    logging::log("PANEL", "show_share_sheet: Failed to create NSImage");
                    return;
                }
                image
            }
        };

        // Pattern: Create array with single object
        let items: id = msg_send![class!(NSArray), arrayWithObject: share_item];
        if items == nil {
            logging::log("PANEL", "show_share_sheet: Failed to create NSArray");
            return;
        }

        // ... show share sheet ...
    }
}
```

---

## Key Patterns Summary

1. **Ownership**: alloc/init = you own it, convenience methods = autoreleased
2. **Null checks**: Always check before dereferencing pointers
3. **Type conversion**: Use CStr for C strings, explicit casting for numbers
4. **Thread safety**: Guard AppKit calls with main thread check
5. **Cleanup**: Use Drop impl for reliable resource cleanup
6. **Method signatures**: Include all colons in selector name
7. **Return types**: Always annotate return type in msg_send!
