# Objective-C 0.2 Quick Reference Card

**TL;DR Gotchas for AI Agents**

## Top 5 Crashes

1. **Forgot null check**
   ```rust
   // WRONG
   let view: id = msg_send![obj, someView];
   let _: () = msg_send![view, doSomething];

   // RIGHT
   if view.is_null() { return; }
   ```

2. **Wrong return type**
   ```rust
   // WRONG
   msg_send![window, windowNumber];  // Compile error, no type

   // RIGHT
   let num: i32 = msg_send![window, windowNumber];
   ```

3. **Forgot to release owned objects**
   ```rust
   let obj: *mut Object = msg_send![class!(MyClass), alloc];
   let obj: *mut Object = msg_send![obj, init];
   let _: () = msg_send![obj, release];  // MUST DO THIS
   ```

4. **Released autoreleased objects**
   ```rust
   // deviceInputWithDevice: returns autoreleased (don't release)
   let input: *mut Object = msg_send![class!(AVCaptureDeviceInput), deviceInputWithDevice: dev error: &mut err];
   // NO release needed
   ```

5. **AppKit called from non-main thread**
   ```rust
   // WRONG
   async { let _: () = msg_send![window, orderFront: nil]; }

   // RIGHT
   cx.spawn(async { hide_main_window(); }).detach();
   ```

## Pattern Templates

### Memory: Object Ownership
```rust
// I OWN it (alloc/init)
let obj: *mut Object = msg_send![class!(X), alloc];
let obj: *mut Object = msg_send![obj, init];
let _: () = msg_send![obj, release];  // MUST release

// I DON'T OWN it (convenience constructor, autoreleased)
let str: *mut Object = msg_send![class!(NSString), stringWithUTF8String: c"x".as_ptr()];
// NO release

// I OWN it (from factory returning "+1 reference")
let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
let session: *mut Object = msg_send![session, init];
let _: () = msg_send![session, release];
```

### Pointer Safety
```rust
// Always check null
let result: id = msg_send![obj, method];
if result.is_null() {
    logging::log("ERROR", "method returned nil");
    return;
}

// Extract NSString
let utf8: *const i8 = msg_send![ns_string, UTF8String];
if utf8.is_null() { return None; }
let s = CStr::from_ptr(utf8).to_string_lossy().into_owned();
```

### Thread Safety
```rust
// Check thread
fn require_main_thread(name: &str) -> bool {
    if !unsafe { msg_send![class!(NSThread), isMainThread] } {
        logging::log("ERROR", &format!("{} off main thread", name));
        return true;
    }
    false
}

// Defer from GPUI context (use cx.spawn, NOT direct msg_send)
pub fn defer_hide_main_window(cx: &mut gpui::App) {
    cx.spawn(async { hide_main_window(); }).detach();
}
```

### msg_send! Patterns
```rust
// Void method
let _: () = msg_send![obj, method];

// Single argument
let result: i32 = msg_send![obj, methodWithArg: value];

// Multiple arguments (colons in selector)
let dict: id = msg_send![
    class!(NSDictionary),
    dictionaryWithObject: obj
    forKey: key
];

// Custom type return
let rect: NSRect = msg_send![window, frame];

// Discard explicit type
let _: id = msg_send![obj, method];
```

## Imports (Always Together)

```rust
use objc::{class, msg_send, sel, sel_impl};  // BOTH sel and sel_impl
use cocoa::base::{id, nil};
use cocoa::foundation::NSRect;
use std::ffi::c_str;
```

## Type Mappings

| ObjC | Rust | Example |
|------|------|---------|
| void | () | `let _: () = msg_send![...]` |
| BOOL | bool | `let b: bool = msg_send![...]` |
| int | i32 | `let n: i32 = msg_send![...]` |
| NSInteger | isize | `let n: isize = msg_send![...]` |
| Object* | *mut Object | `let obj: *mut Object = msg_send![...]` |
| NSRect | cocoa::foundation::NSRect | `let r: NSRect = msg_send![...]` |

## Null/nil Checks

```rust
if ptr.is_null() { /* handle */ }
if ptr == nil { /* handle */ }  // Same thing
if !ptr.is_null() { /* safe to use */ }
```

## Common Methods (Reference)

```rust
// NSArray
let count: usize = msg_send![array, count];
let item: id = msg_send![array, objectAtIndex: 0];

// NSString
let utf8: *const i8 = msg_send![ns_str, UTF8String];
let ns_str: id = msg_send![class!(NSString), stringWithUTF8String: c"text".as_ptr()];

// NSWindow
let _: () = msg_send![window, orderFront: nil];
let _: () = msg_send![window, orderOut: nil];
let _: () = msg_send![window, release];
let title: id = msg_send![window, title];
let number: i32 = msg_send![window, windowNumber];

// NSApplication
let app: id = cocoa::appkit::NSApp();
let _: () = msg_send![app, setActivationPolicy: policy];
let windows: id = msg_send![app, windows];

// NSObject
let class_obj: *const _ = msg_send![obj, class];
let is_kind: bool = msg_send![obj, isKindOfClass: class!(NSView)];
```

## Dispatch Queues

```rust
extern "C" {
    fn dispatch_queue_create(label: *const i8, attr: *mut c_void) -> *mut c_void;
    fn dispatch_release(queue: *mut c_void);
    fn dispatch_sync_f(q: *mut c_void, ctx: *mut c_void, f: extern "C" fn(*mut c_void));
}

let queue = dispatch_queue_create(c"label".as_ptr(), std::ptr::null_mut());
// ... use queue ...
dispatch_release(queue);
```

## Selectors for Method Registration

```rust
use objc::declare::ClassDecl;

let mut decl = ClassDecl::new("MyClass", class!(NSObject)).unwrap();

// Add instance variable
decl.add_ivar::<*mut c_void>("_sender");

// Add method
unsafe {
    decl.add_method(
        sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
        callback as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
    );
}

decl.register();

// Later: look it up
let cls = Class::get("MyClass").unwrap();
let obj: *mut Object = msg_send![cls, alloc];
```

## CGS Private APIs (Cursor Handling)

```rust
use std::ffi::c_str;

extern "C" {
    fn CGSMainConnectionID() -> c_int;
    fn CGSSetConnectionProperty(
        cid: c_int,
        target_cid: c_int,
        key: id,
        value: id,
    ) -> c_int;
}

// Use dlsym for symbol loading (avoid linker failures)
fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void { ... }
const RTLD_DEFAULT: *mut c_void = (-2isize) as *mut c_void;
let ptr = dlsym(RTLD_DEFAULT, b"CGSSetWindowTags\0".as_ptr() as *const c_char);
```

## Testing/Debugging

```bash
# See ObjC method dispatch errors, crashes
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui

# Common errors logged:
# - "unrecognized selector sent to instance"
# - "NSArray index out of bounds"
# - "NSWindow does not have a contentView"
```

## Links

- Full guide: `.claude/objc-0.2-guide.md`
- Examples in codebase:
  - Camera: `src/camera/mod.rs`
  - Windows: `src/platform/app_window_management.rs`
  - Vibrancy: `src/platform/vibrancy_config.rs`
  - Cursor: `src/platform/cursor.rs`
