Implemented the platform-compatibility changes from `plans/platform-compatibility.md` with cfg-gated webcam/selected-text paths and shared platform action routing.

**Changed Files**
- `src/lib.rs`
- `src/prompts/mod.rs`
- `src/prompts/webcam_stub.rs` (new)
- `src/selected_text.rs`
- `src/app_execute.rs`
- `src/app_impl.rs`
- `src/file_search.rs`
- `src/app_actions.rs`

**What Changed**
- Added macOS gating for webcam module export (`camera`) and prompt wiring.
- Added non-mac webcam prompt stub (`WebcamPrompt`) so non-mac builds have a safe fallback UI.
- Split webcam open/capture flows with `#[cfg(target_os = "macos")]` and explicit unsupported behavior on non-mac.
- Moved selected-text platform branching inside `selected_text` API boundary with non-mac stubs returning explicit unsupported errors.
- Routed path actions in app logic through shared helpers:
  - `crate::file_search::reveal_in_finder(...)`
  - `crate::file_search::open_in_terminal(...)`
  - `crate::file_search::move_to_trash(...)`
- Added `file_search` helpers for terminal/trash plus focused unit tests for path resolution and non-mac unsupported responses.

**Verification Run**
- `cargo check` ✅ passed
- `cargo test --lib file_search::tests::test_terminal_working_directory_uses_directory_path_when_is_dir` ✅ passed
- `cargo clippy --all-targets -- -D warnings` ❌ failed due unrelated in-progress workspace issue:
  - `src/scriptlets.rs:147` missing lifetime specifier (`E0106`)

**How To Test**
1. `cargo check`
2. `cargo test --lib file_search::tests::test_terminal_working_directory_uses_directory_path_when_is_dir`
3. Optional runtime sanity:
   - macOS: invoke webcam/path actions and verify normal behavior
   - non-mac: invoke webcam/path actions and verify explicit unsupported HUD/error messaging

**Risks / Known Gaps**
- Full clippy/test gates are currently blocked by unrelated concurrent changes outside this task (`src/scriptlets.rs`).
- This repo is under heavy parallel-agent churn; touched files include other unrelated diffs already present in the working tree.

**Commits**
- No commits made.