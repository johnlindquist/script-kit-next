# Research: Paste to Active App - Clipboard History Action

## Files Investigated

1. `/Users/johnlindquist/dev/script-kit-gpui/src/frontmost_app_tracker.rs`
   - Already tracks the frontmost/active application using NSWorkspace observer
   - Provides `get_last_real_app() -> Option<TrackedApp>` API
   - `TrackedApp` has `name`, `bundle_id`, and `pid` fields

2. `/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders.rs`
   - Contains `get_clipboard_history_context_actions(entry: &ClipboardEntryInfo) -> Vec<Action>`
   - Line 826: Currently hardcodes "Paste to WezTerm" 
   - `ClipboardEntryInfo` struct at line 50

3. `/Users/johnlindquist/dev/script-kit-gpui/src/actions/dialog.rs`
   - `ActionsDialog::with_clipboard_entry()` at line 313
   - Calls `get_clipboard_history_context_actions(entry_info)`

4. `/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins.rs`
   - Line 257-286: Handles "enter" key in ClipboardHistory view
   - Uses `clipboard_history::copy_entry_to_clipboard()` and `simulate_paste_with_cg()`

## Current Behavior

The current "Paste to WezTerm" action title is hardcoded in `builders.rs`:

```rust
actions.push(
    Action::new(
        "clipboard_paste",
        "Paste to WezTerm",  // <-- HARDCODED
        Some("Copy to clipboard and paste to focused app".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("↵"),
);
```

## Root Cause Analysis

The action builder function doesn't have access to the frontmost app information. It only receives `ClipboardEntryInfo` which contains clipboard entry details, not app context.

## Proposed Solution

### Option A: Pass App Name to Builder Function

1. Add optional `frontmost_app_name: Option<String>` parameter to `get_clipboard_history_context_actions`:
   - Or add it to `ClipboardEntryInfo` struct
   
2. Update the title dynamically:
   ```rust
   let paste_title = match frontmost_app_name {
       Some(name) => format!("Paste to {}", name),
       None => "Paste to Active App".to_string(),
   };
   ```

3. Callers need to call `get_last_real_app()` and pass the name

### Implementation Changes

1. **Modify `ClipboardEntryInfo`** (in `builders.rs` line 50):
   - Add field: `pub frontmost_app_name: Option<String>`

2. **Update `get_clipboard_history_context_actions`** (line 812):
   - Use `entry.frontmost_app_name` for dynamic title

3. **Update `ActionsDialog::with_clipboard_entry`** (dialog.rs line 313):
   - Already receives `entry_info`, no changes needed there

4. **Update callers** (wherever `ClipboardEntryInfo` is constructed):
   - Call `frontmost_app_tracker::get_last_real_app()` to get the name
   - Populate the new field

## Verification Plan

1. Run `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. Test with clipboard history:
   - Switch to different apps (Chrome, VS Code, Terminal)
   - Open clipboard history
   - Verify action shows "Paste to [AppName]" dynamically

## Verification

### What Was Changed

1. **`/Users/johnlindquist/dev/script-kit-gpui/src/actions/builders.rs`**:
   - Added `frontmost_app_name: Option<String>` field to `ClipboardEntryInfo` struct (line 62-63)
   - Updated `get_clipboard_history_context_actions` to use dynamic app name for paste action (lines 825-834)
   - Updated all test instances of `ClipboardEntryInfo` to include the new field
   - Added new test `test_clipboard_history_paste_to_dynamic_app_name` to verify:
     - Default behavior: "Paste to Active App" when no app name
     - Dynamic behavior: "Paste to Visual Studio Code" / "Paste to Google Chrome" etc.

### Test Results

```
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

- **cargo check**: PASSED
- **cargo clippy**: PASSED (no warnings)
- **cargo test**: All tests passed including new test

### Implementation Details

The solution uses a simple pattern match to generate the title:

```rust
let paste_title = match &entry.frontmost_app_name {
    Some(name) => format!("Paste to {}", name),
    None => "Paste to Active App".to_string(),
};
```

### Remaining Work

For full integration, callers of `ActionsDialog::with_clipboard_entry()` need to:

1. Import `frontmost_app_tracker::get_last_real_app()`
2. Populate `ClipboardEntryInfo.frontmost_app_name` with the app name:
   ```rust
   let app_name = frontmost_app_tracker::get_last_real_app().map(|a| a.name);
   let entry_info = ClipboardEntryInfo {
       // ... other fields ...
       frontmost_app_name: app_name,
   };
   ```

The `frontmost_app_tracker` module already provides:
- `start_tracking()` - called at app startup
- `get_last_real_app() -> Option<TrackedApp>` - returns the last non-Script-Kit app
- `TrackedApp.name` - the localized display name of the app

### Before/After Comparison

**Before:**
- Action title: `"Paste to WezTerm"` (hardcoded)

**After:**
- Action title: Dynamic based on `frontmost_app_name`:
  - `"Paste to Visual Studio Code"` when VS Code was frontmost
  - `"Paste to Google Chrome"` when Chrome was frontmost
  - `"Paste to Active App"` as fallback
