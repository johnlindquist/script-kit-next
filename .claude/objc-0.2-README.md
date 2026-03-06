# Objective-C 0.2 Documentation Index

Complete reference for using `objc = "0.2"` with `cocoa = "0.26"` in Script Kit GPUI.

## Start Here

1. **In a hurry?** → Read `objc-0.2-quick-ref.md` (5 minutes)
2. **Need deep knowledge?** → Read `objc-0.2-guide.md` (25 minutes)
3. **Want to see real code?** → Read `objc-0.2-codebase-examples.md` (20 minutes)

## Files in This Documentation Set

### objc-0.2-quick-ref.md
**Purpose**: Cheat sheet for common patterns and gotchas.

**Contains**:
- Top 5 crashes and how to prevent them
- Memory ownership patterns (alloc/init vs autoreleased)
- Pointer safety checks
- Thread safety guards
- msg_send! syntax patterns
- Common method references
- Type mappings

**Use when**: Coding quickly, need syntax reminder, debugging specific crash.

### objc-0.2-guide.md
**Purpose**: Complete, in-depth reference for all objc 0.2 features.

**Sections**:
1. msg_send! macro syntax and return type handling
2. sel! vs sel_impl! — when both are required (IMPORTANT)
3. Memory management (retain/release/autorelease)
4. Null pointer handling
5. NSString / UTF8String conversion
6. Thread safety (what must run on main thread)
7. objc 0.2 vs objc2 differences
8. class! macro usage
9. Autorelease pool management
10. Common runtime crashes and prevention

**Use when**: Learning the system, understanding WHY a pattern exists, implementing new ObjC interop.

### objc-0.2-codebase-examples.md
**Purpose**: Real, copy-paste-able code from Script Kit GPUI.

**Examples**:
1. **Complete Ownership Lifecycle** (Camera Module) - gold standard
   - Creating owned objects
   - Cleanup helper function
   - Drop implementation
   - Delegate class registration
   - NSString extraction

2. **Main Thread Safety** (Window Management)
   - Thread check guard
   - Safe AppKit method wrapper
   - Deferred main thread execution

3. **Recursive View Traversal** (Vibrancy Config)
   - Type checking with isKindOfClass:
   - Getting/setting properties
   - Recursing into subviews

4. **Cursor Swizzling** (Cursor Module)
   - Loading private APIs with dlsym
   - Method swizzling with method_setImplementation

5. **Window Iteration** (App Window Management)
   - Getting arrays from objects
   - Iterating with bounds checking
   - Extracting strings from objects

6. **Share Sheet** (Visibility Focus)
   - Multiple item types
   - Creating NSString from Rust String
   - Creating NSData from byte slice
   - Creating NSArray

**Use when**: Implementing similar functionality, copy-pasting pattern templates, understanding how real code does it.

## Quick Decision Tree

**I'm implementing a new ObjC interface...**

- [ ] Check if pattern exists in `objc-0.2-codebase-examples.md`
- [ ] If yes: copy the pattern
- [ ] If no: read relevant section in `objc-0.2-guide.md`
- [ ] Use `objc-0.2-quick-ref.md` for syntax while coding

**I hit a crash...**

1. Read "Common Runtime Crashes" in `objc-0.2-guide.md` (section 10)
2. Check `objc-0.2-quick-ref.md` "Top 5 Crashes"
3. Look for similar code in `objc-0.2-codebase-examples.md`
4. Enable logging: `SCRIPT_KIT_AI_LOG=1`

**I'm unsure about memory management...**

1. Read `objc-0.2-quick-ref.md` "Memory: Object Ownership"
2. Read section 3 of `objc-0.2-guide.md`
3. Copy Drop impl pattern from Example 1 in `objc-0.2-codebase-examples.md`

**I need to call a method...**

1. Check `objc-0.2-quick-ref.md` "Common Methods"
2. If not there, search `objc-0.2-guide.md` section 1 (msg_send! syntax)
3. Look for similar calls in `objc-0.2-codebase-examples.md`

## Critical Rules (NON-NEGOTIABLE)

1. **Always specify explicit return types in msg_send!**
   ```rust
   let _: i32 = msg_send![window, windowNumber];  // ✓
   msg_send![window, windowNumber];  // ✗ Compile error
   ```

2. **Import sel! and sel_impl! together (even if only using one)**
   ```rust
   use objc::{class, msg_send, sel, sel_impl};  // ✓
   use objc::{class, msg_send, sel};  // ✗ Won't compile
   ```

3. **Check pointers for null before dereferencing**
   ```rust
   if ptr.is_null() { return; }
   let _: () = msg_send![ptr, method];  // ✓ Safe
   ```

4. **Release objects you own (from alloc/init)**
   ```rust
   let obj: *mut Object = msg_send![class!(X), alloc];
   let obj: *mut Object = msg_send![obj, init];
   let _: () = msg_send![obj, release];  // ✓ Must do this
   ```

5. **Use defer_hide_main_window() instead of direct orderOut: from GPUI context**
   ```rust
   // Inside GPUI callback:
   defer_hide_main_window(cx);  // ✓ Safe
   hide_main_window();  // ✗ RefCell panic
   ```

## Files Using objc 0.2 in This Codebase

These are good reference implementations:

| File | Pattern | Level |
|------|---------|-------|
| `src/camera/mod.rs` | Complete ownership, Drop impl, delegates | Expert |
| `src/platform/app_window_management.rs` | Thread safety, AppKit wrappers | Intermediate |
| `src/platform/vibrancy_config.rs` | Recursive traversal, type checking | Intermediate |
| `src/platform/cursor.rs` | Private APIs, dlsym, swizzling | Advanced |
| `src/platform/visibility_focus.rs` | Window methods, null checks | Beginner |
| `src/platform/secondary_window_config.rs` | Configuration, simple msg_send | Beginner |

## Debugging Tips

### Enable Logging
```bash
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui
```

Common error messages:
- "unrecognized selector" → typo in method name
- "NSArray index out of bounds" → didn't check count before objectAtIndex:
- "NSWindow does not have X" → null check failed

### Crash Indicators
- "EXC_BAD_ACCESS" → dereferenced null pointer
- "RefCell already borrowed" → called orderOut: from GPUI context
- "Segmentation fault" → double-release or use-after-free
- "Calling non-thread-safe AppKit methods" → not on main thread

## Links

- **Apple ObjC Documentation**: https://developer.apple.com/documentation/
- **objc 0.2 Docs**: https://docs.rs/objc/0.2/objc/
- **cocoa 0.26 Docs**: https://docs.rs/cocoa/0.26/cocoa/
- **core-graphics 0.24 Docs**: https://docs.rs/core-graphics/0.24/

## Version Info

- **objc**: 0.2 (NOT objc2)
- **cocoa**: 0.26
- **core-graphics**: 0.24

## Contributing

When adding new ObjC interop code:

1. Follow patterns from `objc-0.2-codebase-examples.md`
2. Use Drop impl for resource cleanup (see Example 1)
3. Always check pointers for null
4. Always annotate msg_send! return types
5. Use logging (tracing::info!, etc.) for diagnostics
6. Test with SCRIPT_KIT_AI_LOG=1

---

**Last Updated**: 2026-03-01

**For AI Agents**: These docs are optimized for LLM agents. They include explicit patterns, real code examples, and common mistakes. When implementing ObjC interop, reference this documentation to avoid crashes and ensure correctness.
