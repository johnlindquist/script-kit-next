# macOS Native Crates Audit

## Scope
- Repository: `script-kit-gpui`
- Audit date: 2026-02-07
- Requested crates: `cocoa`, `core-graphics`, `core-video` (with `metal`), `core-foundation`, `objc`, `foreign-types`, `libc`
- Files reviewed: `Cargo.toml`, `Cargo.lock`, and macOS-heavy modules under `src/**/*.rs`

## Dependency Baseline

### Declared versions
- `cocoa = "0.26"` (`Cargo.toml:35`)
- `core-graphics = "0.24"` (`Cargo.toml:34`)
- `core-video = { version = "0.4.3", features = ["metal"] }` (`Cargo.toml:36`)
- `core-foundation = "0.10"` (`Cargo.toml:119`)
- `objc = "0.2"` (`Cargo.toml:37`, resolves to `0.2.7` in lockfile)
- `foreign-types = "0.5"` (`Cargo.toml:120`)
- `libc = "0.2.178"` (`Cargo.toml:90`)

### Version posture (from `cargo info`)
- `objc`: `0.2.7`; modern successor exists as `objc2 0.6.3`.
- `cocoa`: pinned `0.26.0` (latest `0.26.1`).
- `core-graphics`: pinned `0.24.0` (latest `0.25.0`).
- `core-video`: pinned `0.4.3` (latest `0.5.2`).
- `core-foundation`: pinned `0.10.0` (latest `0.10.1`).
- `foreign-types`: `0.5.0` (current line).
- `libc`: pinned `0.2.178` (latest in registry includes `1.0.0-alpha.2`; `0.2.x` remains common ecosystem baseline).

### Transitive reality
- `objc 0.2.7` is heavily used by direct + transitive macOS dependencies (`cargo tree -i objc@0.2.7`).
- `objc2` is already present transitively (`0.6.3`, plus `0.5.2` via `icrate`) through crates like `arboard`, `global-hotkey`, `xcap`, etc.
- Multiple generations coexist (`cocoa`/`objc` and `objc2`) in the same dependency graph.

## Direct Answers To Requested Checks

### 1) Is `objc 0.2` outdated?
Yes, relative to current ecosystem direction.
- `objc 0.2.7` is the legacy API surface and predates modern Rust safety/typing patterns used in `objc2`.
- This repository already pulls in `objc2` transitively, so the modern stack is available in the graph.
- Practical constraint: `cocoa 0.26` and upstream dependencies (including `gpui` chain) still depend on `objc 0.2`, so immediate full removal is not realistic without broader dependency migration.

### 2) Are Cocoa APIs used safely?
Mixed.
- Positive: many callsites include nil checks and defensive early returns.
- Positive: some modules show solid lifecycle handling (`src/camera.rs`, `src/window_control.rs`, `src/ocr.rs`).
- Risk: several manual `alloc/init` callsites miss balancing `release` calls (detailed below).

### 3) Any missing retain/release?
Yes. Concrete issues identified.

#### High: leaked objects in share sheet path
- `src/platform.rs:624` creates `NSString` via `alloc/init_str` and never releases.
- `src/platform.rs:643` + `src/platform.rs:644` creates `NSImage` via `alloc/initWithData` and never releases.
- `src/platform.rs:659` + `src/platform.rs:660` creates `NSSharingServicePicker` via `alloc/initWithItems` and never releases.
- Function exits at `src/platform.rs:677` with no cleanup branch.

#### Medium: leaked NSString in app icon extraction
- `src/app_launcher.rs:1054` allocates `CocoaNSString` for path and does not release before return at `src/app_launcher.rs:1103`.

#### Medium: leaked NSString in pasteboard file-url helper
- `src/clipboard_history/macos_paste.rs:67` allocates `NSString` and does not release on success/failure paths.
- `NSImage` is correctly released (`src/clipboard_history/macos_paste.rs:92`), which highlights the imbalance for `ns_string`.

#### Low: observer lifetime is effectively leaked/immortal by design
- `src/frontmost_app_tracker.rs:391` + `src/frontmost_app_tracker.rs:392` alloc/init observer.
- Observer is registered then thread run loop is started forever (`src/frontmost_app_tracker.rs:411-427`).
- This is likely intentional long-lived ownership, but there is no explicit shutdown/removeObserver path; early-return paths after allocation can leak during setup failures.

### 4) Are `core-video` metal features fully utilized?
Partially.
- Feature is enabled in dependency config (`Cargo.toml:36`) and active in resolved feature graph (`cargo tree -e features -i core-video@0.4.3`).
- App uses `CVPixelBuffer` and GPUI surface rendering (`src/prompts/webcam.rs:6`, `src/prompts/webcam.rs:95`), and capture output is configured for NV12 specifically for GPUI Metal rendering (`src/camera.rs:397-401`).
- No direct usage of `CVMetalTextureCache` / `core_video::metal` APIs was found in repo search.
- Conclusion: metal path is leveraged indirectly via GPUI/media stack, not fully exploited directly in this codebase.

### 5) Are there safer alternatives to raw `objc` calls?
Yes.
- `objc2` family crates (`objc2`, `objc2-foundation`, `objc2-app-kit`, etc.) provide stronger typing and ownership semantics than raw `msg_send!`-heavy code.
- Incremental RAII wrappers around Objective-C owned objects would reduce leak risk even before full migration.
- In CoreFoundation-heavy paths, existing wrappers already show safer patterns (`cf_retain`/`cf_release` and ownership tests in `src/window_control.rs:277-294`, `src/window_control.rs:1634-1686`). Similar ownership modeling can be applied to Cocoa objects.

## Notable Safe Patterns Already Present
- Camera capture teardown is disciplined and explicit (`src/camera.rs:232-258`, `src/camera.rs:262-288`).
- OCR path balances releases on success and failure (`src/ocr.rs:223-224`, `src/ocr.rs:255-257`, `src/ocr.rs:307-309`).
- Window-control CoreFoundation ownership discipline is strong and tested (`src/window_control.rs:461-513`, tests at `src/window_control.rs:1634-1686`).
- Keyboard monitor uses `foreign-types` appropriately to bridge `CGEvent` raw pointer for missing API coverage (`src/keyboard_monitor.rs:513-543`).

## Risk Assessment
- **Primary risk:** memory leaks in frequently used UI paths involving share sheet and image/file handling.
- **Secondary risk:** broad raw Objective-C surface area (`msg_send!` usage is high), making ownership bugs easy to reintroduce.
- **Performance risk:** core-video metal support exists, but no direct texture-cache path means optimization headroom remains for high-throughput camera workflows.

## Recommendations (Priority Order)

1. Fix current ownership leaks first.
- `src/platform.rs` share sheet: explicitly release owned `NSString`/`NSImage`/`NSSharingServicePicker` or use autoreleased constructors consistently.
- `src/app_launcher.rs` and `src/clipboard_history/macos_paste.rs`: release path `NSString` allocations.

2. Introduce a small RAII utility layer for Objective-C owned objects.
- Add a minimal owned-wrapper pattern (drop => release) for `id` values created via `alloc/init`.
- Use it in leaf modules first (share sheet, app icon extraction, pasteboard path helpers).

3. Begin incremental `objc2` migration where low-risk.
- Start with modules already isolated behind helper functions.
- Avoid wide rewrites in high-churn GPUI integration until upstream dependency constraints are clearer.

4. Clarify camera metal strategy.
- If current throughput is acceptable, keep indirect GPUI path.
- If webcam rendering/capture is a hotspot, evaluate direct `CVMetalTextureCache` integration to avoid extra conversions and make metal usage explicit.

## Bottom Line
- `objc 0.2` is legacy/outdated relative to `objc2`, but still structurally required by current dependency stack.
- Cocoa usage is partly safe but not fully ownership-safe today.
- There are concrete retain/release leaks to fix now.
- `core-video` metal feature is enabled and active, but direct metal API utilization is not comprehensive.
