# Objective-C (objc 0.2) Interop Guide for Script Kit GPUI

This guide documents practical gotchas and best practices for using `objc = "0.2"` with `cocoa = "0.26"` and `core-graphics = "0.24"` in Script Kit GPUI. Focuses on real issues that AI coding agents encounter.

**Key Dependency Versions:**
- `objc = "0.2"` (NOT objc2 — very different API)
- `cocoa = "0.26"`
- `core-graphics = "0.24"`

---

## 1. msg_send! Macro: Syntax and Return Types

### Basic Syntax

```rust
// Return value MUST match actual method return type
let result: ReturnType = msg_send![receiver, method_name: arg1, arg2];

// No-return methods use `()`
let _: () = msg_send![window, orderOut: nil];

// Multiple args with colons (part of selector)
let _: () = msg_send![delegate, setSampleBufferDelegate: output queue: queue];
```

### Critical: Explicit Return Type Annotations

The compiler **cannot infer** the return type. You MUST always specify it, even when discarding:

```rust
// WRONG - will not compile
let _ = msg_send![window, windowNumber];

// RIGHT - explicit type
let window_number: i32 = msg_send![window, windowNumber];

// RIGHT - discard with explicit type
let _: i32 = msg_send![window, windowNumber];

// RIGHT - void method
let _: () = msg_send![window, release];
```

### Return Type Mapping

Common ObjC → Rust type mappings:

| ObjC Type | Rust Type | Notes |
|-----------|-----------|-------|
| `void` | `()` | Always use `let _: () = ...` |
| `BOOL` (ObjC)  | `bool` | YES/NO map to true/false |
| `NSInteger` / `NSUInteger` | `isize` / `usize` | 64-bit on modern macOS |
| `CGFloat` | `f64` | Core Graphics floats |
| `int` / `unsigned int` | `i32` / `u32` | Explicit size |
| `long` / `long long` | `i64` | Clarify with explicit casting |
| Object pointers | `*mut Object` or `id` | See "Pointer Handling" section |
| Structs (NSRect, etc.) | Use `cocoa::foundation::NSRect` | Imported from `cocoa` crate |

### Selector with Multiple Arguments

Colons are **part of the selector name** in ObjC. Map them to named parameters:

```rust
// ObjC: [output setSampleBufferDelegate:delegate queue:queue]
// Maps to: setSampleBufferDelegate:queue: (two colons, two args)
let _: () = msg_send![
    output,
    setSampleBufferDelegate: delegate
    queue: queue
];

// ObjC: [dict dictionaryWithObject:obj forKey:key]
let dict: *mut Object = msg_send![
    class!(NSDictionary),
    dictionaryWithObject: obj
    forKey: key
];
```

---

## 2. sel! vs sel_impl! — When Both Are Required

### Import Rule (NON-NEGOTIABLE)

**Both `sel!` and `sel_impl!` must be imported together**, even if you only use one:

```rust
// WRONG - will not compile (missing sel_impl!)
use objc::{class, msg_send, sel};

// RIGHT - both together
use objc::{class, msg_send, sel, sel_impl};
```

### When to Use Each

#### sel! — Most Common

Use `sel!` when you **already have a method pointer** and just need the selector:

```rust
extern "C" fn my_callback(this: &mut Object, _sel: Sel, ...) {
    // Use sel! to create selector for msg_send
    let count: usize = msg_send![this, countOfItems];
}

// Registering methods in ClassDecl
decl.add_method(
    sel!(myMethod:),
    my_method_impl as extern "C" fn(...),
);
```

#### sel_impl! — Rarely Needed

Use `sel_impl!` **only** when you need to call a specific method implementation directly (almost never in this codebase). It's an internal detail; stick with `sel!`.

### Real Example from Camera Code

```rust
use objc::{class, msg_send, sel, sel_impl};

// sel! used to create the selector
decl.add_method(
    sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
    capture_callback as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
);
```

---

## 3. Memory Management: retain/release/autorelease

### Reference Counting Basics

ObjC uses manual reference counting (objc 0.2 does NOT auto-manage):
- `alloc` / `init` → retain count +1 (YOU own it)
- `class methods` (like `stringWithUTF8String:`) → autoreleased (YOU don't own it)
- `release` → retain count -1 (if you own it)
- `autorelease` → return to pool, released later (rarely used in Rust)

### The Golden Rule

**If you own it (from `alloc` / `init`), you must `release` it.**

```rust
// Pattern 1: alloc + init (retain count = 1, YOU own it)
let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
let session: *mut Object = msg_send![session, init];
// ... use session ...
let _: () = msg_send![session, release];  // MUST do this

// Pattern 2: Convenience constructor (autoreleased, DON'T release)
// Example: deviceInputWithDevice:error: returns autoreleased object
let input: *mut Object = msg_send![
    class!(AVCaptureDeviceInput),
    deviceInputWithDevice: device
    error: &mut error
];
// NO release needed - AVFoundation will autorelease it

// Pattern 3: Check documentation or error path
// When in doubt about ownership, look at Apple docs or error handling
```

### Common ObjC Patterns in This Codebase

From `camera/mod.rs`:

```rust
// AVCaptureSession (we own it)
let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
let session: *mut Object = msg_send![session, init];
// ... use ...
let _: () = msg_send![session, release];  // Release in Drop impl

// AVCaptureVideoDataOutput (we own it)
let output: *mut Object = msg_send![class!(AVCaptureVideoDataOutput), alloc];
let output: *mut Object = msg_send![output, init];
// ... configure ...
let _: () = msg_send![session, addOutput: output];  // Session retains it
let _: () = msg_send![output, release];  // Release our reference

// AVCaptureDeviceInput (autoreleased convenience constructor)
let input: *mut Object = msg_send![
    class!(AVCaptureDeviceInput),
    deviceInputWithDevice: device
    error: &mut error
];
// NO release - already autoreleased by AVFoundation
let _: () = msg_send![session, addInput: input];  // Session retains it
```

### Drop Impl Pattern

Always implement `Drop` for structs holding ObjC object pointers:

```rust
struct CaptureHandle {
    session: *mut Object,
    delegate: *mut Object,
    queue: *mut c_void,
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        unsafe {
            // Stop the session FIRST (ensures no new callbacks)
            let _: () = msg_send![self.session, stopRunning];

            // Drain the dispatch queue (ensures in-flight callbacks finish)
            extern "C" fn noop(_ctx: *mut c_void) {}
            dispatch_sync_f(self.queue, std::ptr::null_mut(), noop);

            // Release ObjC objects we own
            let _: () = msg_send![self.delegate, release];
            let _: () = msg_send![self.session, release];

            // Release dispatch queue
            dispatch_release(self.queue);
        }
    }
}
```

### Dispatch Queue Ownership

Dispatch queues from `dispatch_queue_create` are like objects:
- `dispatch_queue_create` → retain count +1 (YOU own it)
- `dispatch_release` → decrement count

```rust
let queue: *mut c_void = dispatch_queue_create(c"label".as_ptr(), std::ptr::null_mut());
// ... use queue ...
dispatch_release(queue);  // Always release
```

---

## 4. Null Pointer Handling After msg_send!

### Always Check for Null

msg_send! can return null pointers. Accessing null is **undefined behavior**.

```rust
// WRONG - crashes if result is null
let window: id = msg_send![app, windows];
let first: id = msg_send![window, objectAtIndex: 0];  // Segfault if window is null

// RIGHT - check for null first
let window: id = msg_send![app, windows];
if window.is_null() {
    return;
}
let first: id = msg_send![window, objectAtIndex: 0];
```

### Null Check Patterns

Use `is_null()` for pointer checks (imported from `cocoa::base`):

```rust
use cocoa::base::{id, nil};

// nil is the ObjC equivalent of null pointer
let result: id = msg_send![someObject, method];
if result == nil {
    // Handle nil
}

// Or use is_null() (equivalent)
if result.is_null() {
    // Handle null
}
```

### Real Example from visibility_focus.rs

```rust
unsafe {
    let window = match window_manager::get_main_window() {
        Some(w) => w,
        None => return,  // Window not registered
    };

    let content_view: id = msg_send![window, contentView];
    if content_view == nil {
        logging::log("PANEL", "show_share_sheet: contentView is nil");
        return;
    }

    // Now safe to use content_view
    let _: () = msg_send![content_view, addSubview: some_view];
}
```

---

## 5. NSString / UTF8String Conversion

### Creating NSString

Use `msg_send!` with `stringWithUTF8String:` (convenience, returns autoreleased):

```rust
use std::ffi::c_str;

// Method 1: c_str literals (preferred, no allocation)
let ns_str: *mut Object = msg_send![
    class!(NSString),
    stringWithUTF8String: c"Hello".as_ptr()
];

// Method 2: From Rust String
let rust_str = "Hello".to_string();
let ns_str: *mut Object = msg_send![
    class!(NSString),
    stringWithUTF8String: rust_str.as_ptr() as *const i8
];

// Method 3: Using cocoa crate convenience (also autoreleased)
use cocoa::foundation::NSString as CocoaNSString;
let ns_str = CocoaNSString::alloc(nil).init_str("Hello");
```

### Extracting NSString → Rust String

Use `UTF8String` selector to get C string, then convert:

```rust
use std::ffi::CStr;

unsafe {
    // Get the UTF8String (C string pointer)
    let utf8_ptr: *const i8 = msg_send![ns_string, UTF8String];

    // NULL check BEFORE CStr::from_ptr
    if utf8_ptr.is_null() {
        return None;
    }

    // Convert to Rust string
    let rust_str = CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned();
    Some(rust_str)
}
```

### Real Example from camera/mod.rs

```rust
unsafe fn nsstring_to_string(value: *mut Object) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let utf8: *const i8 = msg_send![value, UTF8String];
    if utf8.is_null() {
        return None;  // NSString had no UTF8 representation
    }

    Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
}
```

### NSString Lifetime

NSString pointers returned by msg_send! are valid as long as the NSString object lives:

```rust
// WRONG - dangling pointer after let binding scope
let name: *const i8 = {
    let ns_name: *mut Object = msg_send![obj, name];  // Autoreleased
    msg_send![ns_name, UTF8String]  // Pointer valid here
};
// Autorelease pool drained, pointer now dangling

// RIGHT - keep NSString alive
let ns_name: *mut Object = msg_send![obj, name];
let name: *const i8 = msg_send![ns_name, UTF8String];  // Safe, NSString still alive
```

---

## 6. Thread Safety: What Must Run on Main Thread?

### Rule: AppKit Methods Must Run on Main Thread

All NSWindow, NSView, NSApplication methods **must** run on the main thread. Violating this causes:
- Crashes with "Calling non-thread-safe AppKit methods"
- Deadlocks
- Data corruption

### Checking Main Thread

Use `NSThread.isMainThread`:

```rust
fn is_main_thread() -> bool {
    unsafe {
        let is_main: bool = msg_send![class!(NSThread), isMainThread];
        is_main
    }
}

fn require_main_thread(fn_name: &str) -> bool {
    if !is_main_thread() {
        logging::log("ERROR", &format!("{} called from non-main thread", fn_name));
        return true;  // Signal caller to bail
    }
    false
}
```

### Running Code on Main Thread

Use `dispatch_async_f` or `dispatch_sync_f`:

```rust
extern "C" {
    fn dispatch_sync_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
    fn dispatch_async_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

// In GPUI code, use cx.spawn() instead:
pub fn defer_hide_main_window(cx: &mut gpui::App) {
    cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
        hide_main_window();  // Runs on foreground executor (main thread)
    })
    .detach();
}
```

### Real Example from app_window_management.rs

```rust
pub fn configure_as_accessory_app() {
    if require_main_thread("configure_as_accessory_app") {
        return;  // Bail if not on main thread
    }
    unsafe {
        let app: id = NSApp();
        let _: () = msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY];
    }
}
```

---

## 7. How objc 0.2 Differs from objc2

### objc 0.2 (This Project)

- **Manual reference counting**: You must call `release` for objects you own
- **msg_send! macro**: Explicit return type annotations required
- **sel! / sel_impl!** : Both must be imported together
- **Runtime safety**: Bounds checking and nil-checking required (no compile-time safety)
- **Syntax**: Looks like real ObjC selectors with colons

```rust
use objc::{class, msg_send, sel, sel_impl};

let _: () = msg_send![obj, method: arg];
let _: () = msg_send![obj, release];
```

### objc2 (NOT Used Here)

- **Automatic reference counting**: Uses Rust's ownership model
- **Type-safe**: Compiler prevents many errors
- **Modern Rust API**: `msg_send_id!`, `msg_send!` with inference
- **No manual release**: ARC handles cleanup
- **Syntax**: More Rusty, less like ObjC

```rust
use objc2::{class, msg_send};

let result = msg_send![&obj, method: arg];  // Type inference
// No manual release needed
```

**CRITICAL: Do NOT mix them.** Script Kit uses `objc 0.2` exclusively.

---

## 8. class! Macro Usage

### Basic Usage

```rust
use objc::class;

// Get the ObjC class by name
let nsstring_class = class!(NSString);
let nsapp_class = class!(NSApp);

// Use in msg_send!
let obj: *mut Object = msg_send![class!(NSString), stringWithUTF8String: c"test".as_ptr()];
```

### class! Returns a Pointer to the Class Object

```rust
// class!(NSString) returns: *const Class
// It's a pointer to the runtime Class metadata
let nsstring: *const Class = class!(NSString);

// Use directly in msg_send! — the macro handles the casting
let _: () = msg_send![class!(NSString), stringWithUTF8String: ptr];
```

### Class Registration (One-Time Setup)

Use `Once` to register custom classes:

```rust
use std::sync::Once;
use objc::declare::ClassDecl;

fn register_delegate_class() {
    let superclass = class!(NSObject);
    let Some(mut decl) = ClassDecl::new("SKWebcamDelegate", superclass) else {
        return;  // Name collision or class already registered
    };

    decl.add_ivar::<*mut c_void>("_sender");

    unsafe {
        decl.add_method(
            sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
            capture_callback as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
        );
    }

    decl.register();  // Register with ObjC runtime
}

// Call once at startup
static REGISTER: Once = Once::new();
REGISTER.call_once(register_delegate_class);

// Later, look up the class
let delegate_class = Class::get("SKWebcamDelegate").expect("Class not registered");
```

---

## 9. Autorelease Pool Management

### Autorelease Pools in Rust

objc 0.2 **does not** automatically manage autorelease pools. Most convenience constructors return **autoreleased** objects (like `stringWithUTF8String:`).

### When Do You Need an Autorelease Pool?

- **Spawning threads**: If you create new threads and call ObjC methods, push a pool first
- **Loops calling many ObjC methods**: Temporary pools inside loops prevent memory bloat
- **Most GPUI code**: Already running inside an autorelease pool (GPUI manages it)

### Autorelease Pool Pattern

```rust
extern "C" {
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

// Manual pool management (rare in this codebase)
unsafe {
    let pool = objc_autoreleasePoolPush();

    // Call many ObjC methods here
    for i in 0..1000 {
        let _: *mut Object = msg_send![class!(NSString), stringWithUTF8String: c"x".as_ptr()];
    }

    objc_autoreleasePoolPop(pool);  // Drain pool, autoreleased objects freed
}
```

### In This Codebase

Most code runs inside GPUI's autorelease pool, so explicit pools are rare. The camera module creates a separate dispatch queue that needs pool management:

```rust
// dispatch_queue_create + callbacks need autorelease pools
let queue: *mut c_void = dispatch_queue_create(c"com.scriptkit.webcam".as_ptr(), std::ptr::null_mut());

// Inside the callback (capture_callback), autoreleased objects are safe
// because dispatch queues automatically push/pop pools
```

---

## 10. Common Runtime Crashes and Prevention

### Crash: Accessing Null Pointer (Segmentation Fault)

**Cause:** msg_send! returned null, not checked before use.

```rust
// WRONG
let window: id = msg_send![app, windows];
let count: usize = msg_send![window, count];  // Crashes if window is null

// RIGHT
let window: id = msg_send![app, windows];
if window.is_null() {
    return;
}
let count: usize = msg_send![window, count];
```

### Crash: Double Release (BAD_ACCESS)

**Cause:** Called `release` twice on the same object.

```rust
// WRONG
let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
let session: *mut Object = msg_send![session, init];
let _: () = msg_send![session, release];
let _: () = msg_send![session, release];  // Crash! Double release

// RIGHT
let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
let session: *mut Object = msg_send![session, init];
let _: () = msg_send![session, release];
// null out the pointer to prevent accidental reuse
// or move into a struct with Drop impl
```

### Crash: Memory Not Released (Leaked Objects)

**Cause:** Forgot to release an object you own.

```rust
// WRONG - memory leak
let delegate: *mut Object = msg_send![delegate_class, alloc];
let delegate: *mut Object = msg_send![delegate, init];
// ... use delegate ...
// Forgot: let _: () = msg_send![delegate, release];

// RIGHT
let delegate: *mut Object = msg_send![delegate_class, alloc];
let delegate: *mut Object = msg_send![delegate, init];
// ... use delegate ...
let _: () = msg_send![delegate, release];
```

### Crash: Calling AppKit from Non-Main Thread

**Cause:** Called msg_send! on NSWindow/NSView/NSApp from background thread.

```rust
// WRONG - called from async block
async {
    let _: () = msg_send![window, orderFront: nil];  // Crash!
}

// RIGHT - defer to main thread
pub fn defer_hide_main_window(cx: &mut gpui::App) {
    cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
        hide_main_window();  // Runs on main thread
    })
    .detach();
}
```

### Crash: RefCell Already Borrowed (from orderOut:)

**Cause:** Called `orderOut:` from inside GPUI RefCell borrow context.

```rust
// WRONG - causes RefCell already borrowed panic
listener::on(move |_, _| {
    hide_main_window();  // orderOut: triggers window_did_change_key_status, re-enters RefCell
});

// RIGHT - defer the call
listener::on(move |cx, _| {
    defer_hide_main_window(cx);  // Deferred, RefCell released first
});
```

### Crash: Invalid Selector

**Cause:** Typo in selector name or wrong method signature.

```rust
// WRONG - selector doesn't exist
let _: () = msg_send![obj, nosuchMethod];  // Crashes with "unrecognized selector"

// RIGHT - verify the method exists in Apple docs
let count: usize = msg_send![array, count];  // NSArray has -count
```

### Prevention: Logging and Tracing

Use `tracing` crate for diagnostic logs (see `src/platform/cursor.rs` for examples):

```rust
use tracing::{info, warn, error};

unsafe {
    let window = match window_manager::get_main_window() {
        Some(w) => {
            info!("Using main window");
            w
        }
        None => {
            error!("Main window not registered");
            return;
        }
    };
}
```

---

## Summary Checklist

- [ ] Always specify explicit return types in `msg_send!`
- [ ] Import `sel` and `sel_impl` together
- [ ] Check pointers for null before dereferencing
- [ ] Call `release` on objects you own (from `alloc`/`init`)
- [ ] Don't release autoreleased objects (from convenience constructors)
- [ ] Verify thread safety (main thread required for AppKit)
- [ ] Use `Drop` impl for cleanup of owned ObjC objects
- [ ] Use `defer_hide_main_window()` instead of direct `orderOut:` from GPUI context
- [ ] Test with `SCRIPT_KIT_AI_LOG=1` to see ObjC error messages

---

## References

- **Apple ObjC Documentation**: https://developer.apple.com/documentation/
- **objc 0.2 Docs**: https://docs.rs/objc/0.2/objc/
- **cocoa 0.26 Docs**: https://docs.rs/cocoa/0.26/cocoa/
- **Script Kit Source Examples**:
  - Camera capture: `/src/camera/mod.rs` (complete AVFoundation example)
  - Window management: `/src/platform/app_window_management.rs`
  - Vibrancy config: `/src/platform/vibrancy_config.rs`
  - Cursor handling: `/src/platform/cursor.rs` (CGS private APIs)
  - Visibility: `/src/platform/visibility_focus.rs`
