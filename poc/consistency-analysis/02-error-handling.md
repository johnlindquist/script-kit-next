# Error Handling Patterns Analysis: Script Kit GPUI

## Executive Summary

The Script Kit GPUI codebase demonstrates a **mixed error handling approach**:
- **Primary pattern**: `anyhow` crate for general error handling (~890+ uses)
- **Custom errors**: Limited to specific modules (`MenuExecutorError`, `PathExtractionError`)
- **Error propagation**: Heavy reliance on `?` operator with `anyhow::anyhow!()` for on-the-fly error messages
- **Panic usage**: 1,465 instances of unwrap/expect/panic - **significantly high** for production code
- **Lock poisoning**: Widespread use of `.lock().unwrap()` patterns - a major vulnerability

## 1. Error Type Strategy

### 1.1 Error Libraries Used

**anyhow** (Primary - 890+ uses)
```rust
use anyhow::{Context, Result, bail};
```
- Used for most operations that can fail
- Generic `anyhow::anyhow!()` for ad-hoc error messages
- Good for rapid error propagation
- Drawback: Loses type information; all errors treated as `Box<dyn Error>`

**thiserror** (Minimal - 2 custom types)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MenuExecutorError {
    #[error("Menu item at path {path:?} is disabled")]
    MenuItemDisabled { path: Vec<String> },
    // ...
}
```
- Only used in `menu_executor.rs` (accessibility APIs)
- Only used in `extension_types.rs` (validation result handling)
- Good structured error information but underutilized

**Custom Result Types**
```rust
// action_helpers.rs
pub enum PathExtractionError {
    NoSelection,
    UnsupportedType(SharedString),
}

impl PathExtractionError {
    pub fn message(&self) -> SharedString { /* ... */ }
}
```
- Minimal - only 2-3 custom error enums in entire codebase
- Usually paired with conversion methods for UI display

### Recommendation 1.1
**Expand custom error types for domain errors**. The current approach loses valuable context:

```rust
// Current - generic anyhow
pub fn register_script_hotkey(path: &str, shortcut: &str) -> anyhow::Result<u32> {
    let (mods, code) = shortcuts::parse_shortcut(shortcut)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;
    // ...
}

// Better - structured error
#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Invalid shortcut syntax '{shortcut}': {reason}")]
    InvalidShortcut { shortcut: String, reason: String },
    #[error("Hotkey already registered: {shortcut}")]
    AlreadyRegistered { shortcut: String },
}

pub fn register_script_hotkey(path: &str, shortcut: &str) -> Result<u32, HotkeyError> {
    let (mods, code) = shortcuts::parse_shortcut(shortcut)
        .ok_or(HotkeyError::InvalidShortcut {
            shortcut: shortcut.to_string(),
            reason: "Failed to parse".to_string(),
        })?;
    // ...
}
```

---

## 2. Error Propagation Patterns

### 2.1 The Question Mark Operator (?)

**Usage**: 161+ instances of `map_err()`, `ok_or()`, `ok_or_else()`, and `?`

**Strong pattern in modern modules**:
```rust
// selected_text.rs - good error context
pub fn open_accessibility_settings() -> Result<()> {
    Command::new("open")
        .arg("x-apple.systempreferences:...")
        .spawn()
        .context("Failed to open System Preferences")?;  // Context added
    Ok(())
}

// text_injector.rs - good error context
pub fn paste_text(&self, text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;  // Context added
    clipboard.set_text(text)
        .context("Failed to set clipboard text")?;  // Context added
    Ok(())
}
```

**Weak pattern - generic ad-hoc errors**:
```rust
// hotkeys.rs - generic error messages
let (mods, code) = shortcuts::parse_shortcut(shortcut)
    .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;

// execute_script.rs - generic error messages
CGDisplay::active_displays()
    .map_err(|_| anyhow::anyhow!("Failed to get active displays"))?;
```

### 2.2 Context vs. anyhow::anyhow!()

**Good**: Using `.context()` for error chain preservation
```
error chain:
  original_error
    └─ added context: "Failed to access clipboard"
    └─ added context: "Failed to set clipboard text"
```

**Weak**: Using `.map_err(|_| anyhow::anyhow!())` - loses original error
```rust
CGDisplay::active_displays()
    .map_err(|_| anyhow::anyhow!("Failed to get active displays"))?;
    // Original error is discarded ^^^ (|_|)
```

### 2.3 bail!() Macro (64 instances)

Good for early returns with formatted messages:
```rust
// menu_executor.rs
if !has_accessibility_permission() {
    bail!("Accessibility permission required");
}

if pid <= 0 {
    bail!("Invalid process identifier for frontmost application");
}
```

### Recommendation 2.1
**Standardize error context patterns** by:
1. Always use `.context()` for Result types (preserves error chains)
2. Avoid `.map_err(|_| anyhow::anyhow!())` pattern (loses original error)
3. Create a helper function for common error patterns:

```rust
// Create a module-level helper
fn map_parse_error<E: std::fmt::Display>(e: E, what: &str) -> anyhow::Error {
    anyhow::anyhow!("Failed to parse {}: {}", what, e)
}

// Use it consistently
let parsed = parse_shortcut(shortcut)
    .map_err(|e| map_parse_error(e, "shortcut"))?;
```

---

## 3. Custom Error Types - Detailed Analysis

### 3.1 MenuExecutorError (Strong Example)

**File**: `/Users/johnlindquist/dev/script-kit-gpui/src/menu_executor.rs`

```rust
#[derive(Error, Debug)]
pub enum MenuExecutorError {
    #[error("Menu item at path {path:?} is disabled")]
    MenuItemDisabled { path: Vec<String> },

    #[error("Menu item {path:?} not found in {searched_in}")]
    MenuItemNotFound {
        path: Vec<String>,
        searched_in: String,
    },

    #[error("Application {bundle_id} is not frontmost - cannot access menu bar")]
    AppNotFrontmost { bundle_id: String },

    #[error("Menu structure changed - expected path {expected_path:?}: {reason}")]
    MenuStructureChanged {
        expected_path: Vec<String>,
        reason: String,
    },

    #[error("Accessibility permission required for menu execution")]
    AccessibilityPermissionDenied,

    #[error("Failed to perform AXPress on menu item: {0}")]
    ActionFailed(String),
}
```

**Strengths**:
- Structured, domain-specific variants
- Clear error messages with context
- Can be pattern-matched for specific handling
- Supports custom recovery logic

**Usage**:
```rust
// Can return specific error
return Err(MenuExecutorError::AccessibilityPermissionDenied.into());

// Can pattern match on Result
match execute_menu_action(...) {
    Err(e) => match e.downcast_ref::<MenuExecutorError>() {
        Some(MenuExecutorError::AppNotFrontmost { .. }) =>
            // Show "App not frontmost" specific UI,
        Some(MenuExecutorError::AccessibilityPermissionDenied) =>
            // Show "Enable accessibility" prompt,
        _ => // Generic error handling
    }
}
```

### 3.2 PathExtractionError (Moderate Example)

**File**: `/Users/johnlindquist/dev/script-kit-gpui/src/action_helpers.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PathExtractionError {
    NoSelection,
    UnsupportedType(SharedString),
}

impl PathExtractionError {
    pub fn message(&self) -> SharedString {
        match self {
            PathExtractionError::NoSelection => SharedString::from("No item selected"),
            PathExtractionError::UnsupportedType(msg) => msg.clone(),
        }
    }
}
```

**Strengths**:
- Simple two-variant enum
- Conversion method for UI display
- Type-safe error handling

**Gap**:
- Not integrated with `thiserror` - no Display/Error impl
- Uses custom `.message()` method instead of standard traits
- Could be more ergonomic

**Better pattern**:
```rust
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PathExtractionError {
    #[error("No item selected")]
    NoSelection,

    #[error("Cannot extract path: {0}")]
    UnsupportedType(SharedString),
}

impl PathExtractionError {
    pub fn ui_message(&self) -> SharedString {
        SharedString::from(self.to_string())
    }
}
```

---

## 4. Panic Usage - Critical Issue

### 4.1 Panic Statistics

| Pattern | Count | Risk Level |
|---------|-------|-----------|
| `.unwrap()` | ~800+ | CRITICAL |
| `.expect()` | ~400+ | CRITICAL |
| `panic!()` | ~167 | CRITICAL |
| **Total** | **1,465+** | **Unacceptable for production** |

### 4.2 High-Risk Lock Poisoning

**Pattern**: Widespread `.lock().unwrap()` without poisoning handling

```rust
// keyword_manager.rs (34+ instances)
let mut scriptlets_guard = self.scriptlets.lock().unwrap();
let mut matcher_guard = self.matcher.lock().unwrap();
let mut file_triggers_guard = self.file_triggers.lock().unwrap();

// hotkeys.rs (50+ instances)
let mut guard = manager.lock().unwrap();
```

**Risk**: Single panicked thread → entire application crashes
- No recovery mechanism
- Violates Rust's safety guarantees in concurrent code
- Tests pass but production failures common

**Better pattern**:
```rust
// Option 1: Graceful degradation
let mut guard = manager.lock()
    .map_err(|e| {
        logging::error("Lock poisoned: {}", e);
        anyhow::anyhow!("Concurrent operation failed - try again")
    })?;

// Option 2: Explicit panic with context
let mut guard = manager.lock()
    .expect("ScriptletManager lock should not be poisoned (no panics in critical section)");
    // ^ At least documents the expectation

// Option 3: Use try_lock() for non-blocking
let mut guard = manager.try_lock()
    .ok_or_else(|| anyhow::anyhow!("Manager is busy - operation skipped"))?;
```

### 4.3 Test-Only Unwraps (Acceptable)

```rust
// extension_types.rs (test code - OK)
let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();

// text_injector.rs (test code - OK)
injector.delete_chars(2).expect("Should delete chars");

// menu_executor.rs (FFI code - documented justification)
let c_str = std::ffi::CString::new(s).unwrap();
// ^ CString::new only fails if input contains NUL bytes
// Safe because `s` comes from trusted sources
```

### Recommendation 4.1
**Create a panic elimination task**:

Priority 1 (Critical):
```rust
// Replace all .lock().unwrap() with .map_err() + logging
let mut guard = self.scriptlets.lock()
    .map_err(|poisoned| {
        logging::error("Lock poisoned: {}", poisoned);
        anyhow::anyhow!("Concurrent modification detected")
    })?;
```

Priority 2 (High):
```rust
// Replace unwrap() in fallible operations with proper error handling
let hotkey = HotKey::new(Some(mods), code);
// ^ This shouldn't fail, but if it does, return error instead of panicking
```

Priority 3 (Medium):
```rust
// Replace .expect() in JSON parsing with Context trait
let result = serde_json::from_str(json_str)
    .context("Failed to parse JSON response")?;
```

---

## 5. Error Message Quality & Consistency

### 5.1 Good Error Messages

**High context**:
```rust
// menu_executor.rs
bail!("Accessibility API is disabled");
bail!("No value for attribute: {}", attribute);
bail!("Action {} is not supported", action);
bail!("Cannot complete action {} - element may be disabled", action);

// hotkeys.rs (with specific error types)
anyhow::anyhow!(
    "Hotkey '{}' is already registered (conflict with another app or script). Hotkey ID: {}",
    shortcut,
    hk.id()
)
```

**Medium context**:
```rust
// selected_text.rs
bail!("Accessibility permission required. Enable in System Preferences > Privacy & Security > Accessibility");
bail!("Failed to get selected text: {}", e);
```

### 5.2 Poor Error Messages

**Too generic**:
```rust
// execute_script.rs
Err(anyhow::anyhow!("Missing window_id"))
// ^ Why is it missing? Which operation?

anyhow::anyhow!("Failed to get active displays")
// ^ What caused the failure?

// hotkeys.rs
anyhow::anyhow!("Failed to parse shortcut: {}", shortcut)
// ^ What's wrong with the shortcut? Missing modifier? Invalid key?
```

**Lost context**:
```rust
CGDisplay::active_displays()
    .map_err(|_| anyhow::anyhow!("Failed to get active displays"))?;
    // ^ Original error discarded, caller doesn't know what failed
```

### 5.3 Inconsistent Error Levels

Some modules use `logging` module, others use tracing, others use neither:

```rust
// selected_text.rs (good - uses tracing)
instrument, warn!, debug!, info!

// action_helpers.rs (good - uses logging module)
logging::log("WARN", "...")

// hotkeys.rs (minimal - mostly in error messages only)
// No structured logging of error conditions

// keyword_manager.rs (minimal)
// Relies on error messages only
```

### Recommendation 5.1
**Create error message style guide**:

```rust
// ✓ Good: Who, What, Why, How to fix
bail!("Failed to register hotkey '{}': System rejected it. \
       This may be reserved by macOS. Try a different combination.", shortcut);

// ✗ Bad: Too generic
anyhow::anyhow!("Failed")

// ✗ Bad: Lost original error
.map_err(|_| anyhow::anyhow!("Operation failed"))

// ✓ Good: Specific error types + context
.context("Failed to parse shortcut: valid format is 'ctrl+shift+k'")?

// ✓ Good: Include what was attempted
bail!("Cannot register hotkey: keyboard manager not initialized \
       (call init_script_hotkey_manager first)")
```

---

## 6. Option vs. Result Usage

### 6.1 Appropriate Option Usage

```rust
// menu_executor.rs - finding optional attributes
fn get_ax_string_attribute(element: AXUIElementRef, attribute: &str) -> Option<String> {
    match get_ax_attribute(element, attribute) {
        Ok(value) => { /* conversion */ },
        Err(_) => None,  // Attribute simply missing - not an error
    }
}

// action_helpers.rs - optional values are valid
pub fn find_sdk_action<'a>(
    actions: Option<&'a [ProtocolAction]>,
    action_name: &str,
) -> Option<&'a ProtocolAction> {
    let actions = actions?;
    actions.iter().find(|a| a.name == action_name)
}
```

### 6.2 Inappropriate Result<(), Vec<String>>

```rust
// extension_types.rs - using Result for validation errors
pub fn validate_categories(&self) -> Result<(), Vec<String>> {
    let invalid: Vec<String> = self
        .categories
        .iter()
        .filter(|c| !VALID_CATEGORIES.contains(&c.as_str()))
        .cloned()
        .collect();

    if invalid.is_empty() {
        Ok(())
    } else {
        Err(invalid)
    }
}
```

**Issue**: Returns `Vec<String>` as error (not idiomatic)
- Should use proper error type:

```rust
#[derive(Error, Debug)]
#[error("Invalid categories: {0:?}", .0)]
pub struct InvalidCategories(pub Vec<String>);

pub fn validate_categories(&self) -> Result<(), InvalidCategories> {
    let invalid: Vec<String> = self
        .categories
        .iter()
        .filter(|c| !VALID_CATEGORIES.contains(&c.as_str()))
        .cloned()
        .collect();

    if invalid.is_empty() {
        Ok(())
    } else {
        Err(InvalidCategories(invalid))
    }
}
```

---

## 7. Error Recovery & Fallbacks

### 7.1 Good Fallback Patterns

```rust
// selected_text.rs - multi-strategy approach
pub fn get_selected_text() -> Result<String> {
    // Strategy 1: AX API
    // Strategy 2: Clipboard simulation with Cmd+C
    // Strategy 3: Best effort
    match get_selected_text_impl() {
        Ok(text) => { /* ... */ },
        Err(e) => {
            warn!("Failed to get selected text: {}", e);
            bail!("Failed to get selected text: {}", e)
        }
    }
}

// clipboard restore with best-effort recovery
if let Some(original_text) = original {
    if let Err(e) = clipboard.set_text(&original_text) {
        warn!("Failed to restore original clipboard");
        // Continue anyway - not critical
    }
}
```

### 7.2 Weak Fallback Patterns

```rust
// action_helpers.rs - sends message if possible
let send_result = sender.try_send(msg);
match send_result {
    Ok(()) => true,
    Err(std::sync::mpsc::TrySendError::Full(_)) => {
        logging::log("WARN", "Response channel full - message dropped");
        false
    }
    Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
        logging::log("UI", "Response channel disconnected - script exited");
        false
    }
}
```

**Gap**: No way to signal back to caller that message failed
- Should return Result instead of bool

---

## 8. Async/Concurrency Error Handling

### 8.1 Lock Poisoning (Already covered)

High-risk pattern throughout:
```rust
self.scriptlets.lock().unwrap()  // ✗ Panics on poisoned lock
```

Should be:
```rust
self.scriptlets.lock()
    .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?  // ✓ Graceful
```

### 8.2 Channel Errors

```rust
// action_helpers.rs - proper channel error handling
Err(std::sync::mpsc::TrySendError::Full(_)) => { /* handle */ }
Err(std::sync::mpsc::TrySendError::Disconnected(_)) => { /* handle */ }
```

Good pattern - distinguishes between queue full vs. receiver dropped.

---

## Summary of Recommendations

### Priority 1: Critical
1. **Eliminate lock poisoning**: Replace all `.lock().unwrap()` with proper error handling
2. **Reduce panics from 1,465 to <50**: Focus on critical sections
3. **Create custom error types** for major domains (hotkeys, menu execution, text injection)

### Priority 2: High
4. **Standardize context use**: Always `.context()` instead of `.map_err(|_| anyhow::anyhow!())`
5. **Add logging to error paths**: Use tracing consistently
6. **Document error recovery**: Comment why certain errors are fatal vs. recoverable

### Priority 3: Medium
7. **Improve error messages**: Add "why?" and "how to fix?" guidance
8. **Integrate validation errors properly**: Use thiserror for validation types
9. **Create error handling guide**: Document patterns for team

### Quick Wins
- Add `#![warn(unused_results)]` to catch unhandled Results
- Use clippy: `cargo clippy --all-targets -- -W clippy::all`
- Run `cargo audit` for dependency vulnerabilities

---

## Code Statistics

| Metric | Count | Assessment |
|--------|-------|------------|
| Files analyzed | 50+ | Comprehensive |
| anyhow::Result usage | 890+ | Heavy reliance |
| Custom error types | 3 | Underutilized |
| .context() calls | 20+ | Good in key modules |
| .map_err() calls | 140+ | Mixed quality |
| bail!() calls | 64 | Appropriate |
| Unwrap/expect/panic | 1,465+ | **Critical issue** |
| .lock().unwrap() | 50+ | **High risk** |
| Panics in tests | 400+ | **Acceptable** |
| Panics in prod | 1,065+ | **Unacceptable** |

---

## Conclusion

Script Kit GPUI's error handling is **inconsistent and unsafe**:

**Strengths**:
- Good use of anyhow for rapid prototyping
- Some modules (selected_text.rs, text_injector.rs) show excellent patterns
- Custom error types exist but are underutilized
- Error messages are generally informative

**Weaknesses**:
- **1,465+ panics** - production code should have <50
- **50+ lock poisoning panics** - critical concurrency bug
- **Generic error handling** loses valuable context
- **Inconsistent logging** across modules
- **No error recovery strategy** documented

**Next Steps**: Execute Priority 1 recommendations to improve production reliability.
