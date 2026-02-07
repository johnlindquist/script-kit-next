Implemented the requested audit report at `crates-audits/macos-native.md`.

**Changed files**
- `crates-audits/macos-native.md`

**What’s in the report**
- Crate-by-crate status for `cocoa`, `core-graphics`, `core-video`, `core-foundation`, `objc`, `foreign-types`, `libc`
- Direct answer that `objc 0.2` is legacy vs `objc2`, with current dependency constraints
- Concrete retain/release findings with file references:
  - `src/platform.rs:624`
  - `src/platform.rs:643`
  - `src/platform.rs:659`
  - `src/app_launcher.rs:1054`
  - `src/clipboard_history/macos_paste.rs:67`
- Core-video metal utilization assessment (enabled and active, but no direct `CVMetalTextureCache` usage found)
- Safer alternatives and prioritized recommendations

**Verification run**
- Attempted gate: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Blocked at `cargo check` by unrelated in-progress workspace errors:
  - `src/watcher.rs:23` (private `config::defaults` module access, `E0603`)
  - `src/hotkeys.rs:367` (type mismatch `&Option<HotkeyConfig>` vs `&HotkeyConfig`, `E0308`)
- Because `cargo check` failed, `clippy` and `test` did not run.

**How to test**
1. Open `crates-audits/macos-native.md` and review the findings/recommendations.
2. Re-run the verification gate after current compile breakages in `src/watcher.rs` and `src/hotkeys.rs` are resolved:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / known gaps**
- This task delivered an audit document only; no runtime code fixes were applied.
- Identified ownership issues remain in code until separately patched.

**Commits**
- No commits were made.