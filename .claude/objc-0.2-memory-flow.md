# Objective-C 0.2 Memory Management Flow Chart

Visual guide to understanding who owns what in objc 0.2.

## The Golden Rule

```
+1 retain count = YOU OWN IT = YOU MUST RELEASE IT
```

## Ownership Decision Tree

```
Does the method name start with:

├─ "alloc" or contain "init"?
│  └─ YES → +1 retained count, YOU OWN IT
│           let obj: *mut Object = msg_send![class!(X), alloc];
│           let obj: *mut Object = msg_send![obj, init];
│           // ... use obj ...
│           let _: () = msg_send![obj, release];  // MUST DO THIS
│
├─ Convenience constructor (e.g., "stringWith", "numberWith", "dictionaryWith")?
│  └─ YES → autoreleased, YOU DON'T OWN IT
│           let str: *mut Object = msg_send![class!(NSString), stringWithUTF8String: c"x".as_ptr()];
│           // ... use str ...
│           // NO release needed
│
└─ Unclear? Check Apple docs for "+1" or "retained" in description
   └─ If unsure, release it (conservative approach)
```

## Reference Counting Examples

### Pattern 1: Simple Object Ownership

```
START
  │
  ├─ Class alloc
  │  │ Retain: 1 (you own it)
  │  ├─ sendMessage: init
  │  │  Retain: 1 (you own it)
  │  ├─ configure object
  │  │  Retain: 1 (you still own it)
  │  │
  │  └─ sendMessage: release
  │     Retain: 0 (freed)
  │
  └─ END
```

### Pattern 2: Object Transfer (Session Retains Output)

```
START
  │
  ├─ Create AVCaptureVideoDataOutput
  │  │ Retain: 1 (you own it)
  │  ├─ Configure it
  │  │  Retain: 1 (you own it)
  │  ├─ [session addOutput: output]
  │  │  Retain: 2 (you own 1, session owns 1)
  │  │
  │  └─ sendMessage: release
  │     Retain: 1 (session still owns it)
  │
  └─ When session is released:
     Retain: 0 (freed)
```

### Pattern 3: Autoreleased Object (Don't Release)

```
START
  │
  ├─ [NSString stringWithUTF8String: "x"]
  │  │ Retain: 1 (autoreleased)
  │  │ (will be released at autorelease pool drain)
  │  │
  │  ├─ Use the string
  │  │  Retain: 1 (still autoreleased)
  │  │
  │  └─ END of scope
  │     Retain: 1 (not freed, still in autorelease pool)
  │
  └─ Autorelease pool drains:
     Retain: 0 (finally freed)
```

### Pattern 4: Error Output Parameter (Autoreleased)

```
START
  │
  ├─ let mut error: *mut Object = null_mut();
  │  │ error pointer: nil
  │  │
  │  ├─ [AVCaptureDeviceInput deviceInputWithDevice: dev error: &mut error]
  │  │  │ If fails:
  │  │  │   error pointer: -> NSError object
  │  │  │   NSError Retain: 1 (autoreleased)
  │  │  │
  │  │  │ If succeeds:
  │  │  │   error pointer: nil
  │  │  │   input Retain: 1 (autoreleased)
  │  │  │
  │  │  ├─ Check if input is null
  │  │  │  ├─ Is null: error parameter has NSError
  │  │  │  │  Retain: 1 (autoreleased)
  │  │  │  │  nserror_summary(error)  // Extract strings
  │  │  │  │  // NO release needed
  │  │  │  │
  │  │  │  └─ Not null: no error
  │  │  │     input Retain: 1
  │  │  │     Use it or release
  │  │  │
  │  │  └─ END
  │
  └─ Autorelease pool drains:
     All autoreleased objects freed
```

## Drop Implementation Pattern (Safe Cleanup)

```rust
pub struct Handle {
    obj1: *mut Object,  // We own this (+1 retain)
    obj2: *mut Object,  // We own this (+1 retain)
    queue: *mut c_void, // We own this (from dispatch_queue_create)
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            // STEP 1: Stop any async operations
            let _: () = msg_send![self.obj1, stop];

            // STEP 2: Drain queues (ensure no in-flight callbacks)
            if !self.queue.is_null() {
                dispatch_sync_f(self.queue, std::ptr::null_mut(), noop);
            }

            // STEP 3: Release owned objects in reverse creation order
            if !self.obj2.is_null() {
                let _: () = msg_send![self.obj2, release];
            }

            if !self.obj1.is_null() {
                let _: () = msg_send![self.obj1, release];
            }

            // STEP 4: Release dispatch queue
            if !self.queue.is_null() {
                dispatch_release(self.queue);
            }
        }
    }
}
```

## Memory Leak Prevention

### Leak Pattern 1: Forgot to Release

```rust
// WRONG - leak
unsafe {
    let obj: *mut Object = msg_send![class!(X), alloc];
    let obj: *mut Object = msg_send![obj, init];
    // ... use obj ...
}  // obj dropped, memory leaked!

// RIGHT - released
unsafe {
    let obj: *mut Object = msg_send![class!(X), alloc];
    let obj: *mut Object = msg_send![obj, init];
    // ... use obj ...
    let _: () = msg_send![obj, release];
}  // Retain = 0, freed
```

### Leak Pattern 2: Released in Drop but Not Earlier

```rust
// WRONG - may leak on early return
fn setup() -> Result<Handle> {
    let obj: *mut Object = msg_send![class!(X), alloc];
    let obj: *mut Object = msg_send![obj, init];

    if error_condition {
        return Err(...);  // LEAK! obj still alive, will be dropped later
    }

    Ok(Handle { obj })
}

// RIGHT - use cleanup function on error
fn setup() -> Result<Handle> {
    let obj: *mut Object = msg_send![class!(X), alloc];
    let obj: *mut Object = msg_send![obj, init];

    if error_condition {
        let _: () = msg_send![obj, release];  // Cleanup
        return Err(...);
    }

    Ok(Handle { obj })
}

// BETTER - move into Handle immediately
fn setup() -> Result<Handle> {
    let obj: *mut Object = msg_send![class!(X), alloc];
    let obj: *mut Object = msg_send![obj, init];

    let handle = Handle { obj };  // Now Drop will handle cleanup

    if error_condition {
        return Err(...);  // Handle dropped, release called
    }

    Ok(handle)
}
```

## Dispatch Queue Ownership

```
dispatch_queue_create(label, attr)
         │
         ├─ Retain: 1 (you own it)
         │
         ├─ Use the queue
         │  dispatch_async_f / dispatch_sync_f
         │  Retain: 1 (you still own it)
         │
         └─ dispatch_release(queue)
            Retain: 0 (freed)
```

## Common Retain Count States

| State | Example | Action |
|-------|---------|--------|
| Retain: 0 | Freed memory | DO NOT ACCESS |
| Retain: 1 | You own it | MUST RELEASE |
| Retain: 2+ | You + others own it | Release your reference |
| Autoreleased | From convenience constructor | DON'T RELEASE |
| nil/null | Method returned nil | MUST CHECK |

## Autorelease Pool Timeline

```
Autorelease Pool Push
         │
         ├─ Create: [NSString stringWithUTF8String: "x"]
         │  │ In pool: YES
         │  │ Retain: 1
         │  │
         │  ├─ Use string
         │  │
         │  └─ Leave scope (string still alive in pool)
         │
         └─ Autorelease Pool Pop
            │ All pooled objects released
            │ Retain: 0
            │
            └─ Memory freed
```

## Debug Checklist

When you hit a memory crash:

- [ ] Does the method name start with "alloc" or "init"?
  - YES → Did you call release? (If no, that's your leak/crash)
  - NO → Is it a convenience constructor (stringWith, numberWith, etc.)?
    - YES → Did you call release? (If yes, double release!)
    - NO → Check Apple docs

- [ ] Did you check for null?
  - NO → That's your segfault

- [ ] Did you call msg_send! from a background thread on AppKit object?
  - YES → Use defer_hide_main_window() pattern instead

- [ ] Did you call orderOut: from inside GPUI callback?
  - YES → Use defer_hide_main_window(cx) instead

- [ ] Does Drop impl release all owned objects?
  - NO → Memory leaks or crashes on Handle drop

---

## Quick Reference Ownership Chart

```
Method Call                      | Ownership    | Must Release?
-----------------------------------------------------------
[X alloc]                        | You own it   | YES (in Drop)
[obj init]                       | You own it   | YES (in Drop)
[class!(NSString)                |              |
  stringWithUTF8String: ptr]    | Autoreleased | NO
[AVCaptureDeviceInput            |              |
  deviceInputWithDevice: d       |              |
  error: &mut e]                | Both         | input: NO, error: NO
[session addOutput: output]      | Still you own| YES (after add)
[queue dispatch_queue_create]    | You own it   | YES (dispatch_release)
[app windows]                    | NSApp owns   | NO
[array objectAtIndex: 0]         | Array owns   | NO
```

## Real Codebase Example

```rust
// From camera/mod.rs - gold standard

pub fn start_capture() -> Result<(..., CaptureHandle)> {
    // Create session (+1)
    let session: *mut Object = msg_send![class!(AVCaptureSession), alloc];
    let session: *mut Object = msg_send![session, init];

    // Create output (+1)
    let output: *mut Object = msg_send![class!(AVCaptureVideoDataOutput), alloc];
    let output: *mut Object = msg_send![output, init];

    // Create queue (+1)
    let queue: *mut c_void = dispatch_queue_create(...);

    // Create delegate (+1)
    let delegate: *mut Object = msg_send![delegate_class, alloc];
    let delegate: *mut Object = msg_send![delegate, init];

    // Add output to session (session retains it)
    let _: () = msg_send![session, addOutput: output];

    // Release our reference (session owns it now)
    let _: () = msg_send![output, release];

    // Return handle (owns session, delegate, queue)
    let handle = CaptureHandle {
        session,    // You: 1, Total: 2 (handle + session holds it)
        delegate,   // You: 1
        queue,      // You: 1
        sender_ptr, // Raw pointer to Box
    };

    Ok((rx, handle))
}

// Handle Drop: releases session, delegate, queue
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        unsafe {
            // Session was retaining output, but we released our ref
            // Session is freed here: Retain goes 1 -> 0
            let _: () = msg_send![self.session, release];

            // Delegate freed: Retain goes 1 -> 0
            let _: () = msg_send![self.delegate, release];

            // Queue freed: Retain goes 1 -> 0
            dispatch_release(self.queue);
        }
    }
}
```

---

Print this page for quick reference during coding!
