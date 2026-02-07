Implemented error-handling hardening in the claimed scope with panic-path removals and regression tests.

**Changed Files**
- `src/setup.rs:1473`
  - Hardened `ensure_tsconfig_paths` against malformed JSON shapes.
  - Replaced unwrap-based object assumptions with safe normalization + warnings/logs.
  - Added `json_type_name(...)` for structured type diagnostics.
  - Added tests:
    - `test_ensure_tsconfig_paths_recovers_from_non_object_compiler_options` (`src/setup.rs:2411`)
    - `test_ensure_tsconfig_paths_recovers_from_non_object_paths` (`src/setup.rs:2443`)
- `src/execute_script.rs:107`
  - Added structured missing-session error path:
    - `format_missing_interactive_session_error(...)`
    - `take_active_script_session(...)`
  - Replaced `self.script_session.lock().take().unwrap()` with graceful error handling in interactive execution (`src/execute_script.rs:176`).
  - Added test:
    - `test_take_active_script_session_returns_error_when_session_missing` (`src/execute_script.rs:1663`)
- `src/frontmost_app_tracker.rs:82`
  - Added ObjC safety helpers:
    - `require_objc_class(...)`
    - `make_objc_cstring(...)`
  - Replaced runtime `Class::get(...).unwrap()` paths in observer setup with safe early returns + structured logs.
  - Replaced `CString::new(...).unwrap()` in `objc_nsstring` with fallible conversion and logging (`src/frontmost_app_tracker.rs:512`).
  - Added null checks for notification and userInfo key NSString creation in unsafe/ObjC flow.
  - Added tests:
    - `test_make_objc_cstring_rejects_interior_nul` (`src/frontmost_app_tracker.rs:656`)
    - `test_make_objc_cstring_accepts_valid_string` (`src/frontmost_app_tracker.rs:662`)

**Verification**
- Ran: `cargo check`
- Result: pass

**How To Test**
1. `cargo check`
2. `cargo test ensure_tsconfig_paths_recovers`
3. `cargo test take_active_script_session_returns_error_when_session_missing`
4. `cargo test make_objc_cstring`

**Risks / Known Gaps**
- `cargo test` is currently blocked by an unrelated existing compile error in `src/ai/window.rs`:
  - missing `AiApp::message_body_content` (around `src/ai/window.rs:8729`)
- Because of that upstream failure, the newly added tests could not be executed end-to-end in this shared tree, though `cargo check` is clean.

**Commits**
- No commits made.