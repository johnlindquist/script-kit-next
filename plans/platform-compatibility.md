# Platform Compatibility Audit

Date: 2026-02-07
Agent: `codex-platform-compat`
Scope: `src/**/*.rs`, `Cargo.toml`, `build.rs`

## Executive Summary

The codebase has many deliberate `#[cfg(...)]` guards and some good cross-platform helpers, but there are several high-impact macOS assumptions that currently leak into always-compiled paths. The most critical gaps are in the webcam and selected-text stacks, where macOS-specific crates/types are imported and exported unconditionally. This increases Linux/Windows build risk and makes behavior drift likely even when compile succeeds.

Primary risks:

1. Unconditional macOS modules/types in the webcam stack (`camera`, `prompts::webcam`, and consumers).
2. Unconditional macOS selected-text module imports despite non-mac message handlers existing.
3. Path actions that are macOS-only at execution time with no explicit non-mac fallback or error feedback.
4. Platform UX mismatch (Finder labels and mac-style keycaps) shown on all platforms.

## What Is Already Good

- `build.rs` is platform-neutral and simple (`build.rs:7`).
- `ocr` has explicit macOS/non-mac implementations (`src/ocr.rs:93`, `src/ocr.rs:118`).
- `login_item` provides explicit non-mac stubs with logging (`src/login_item.rs:48`, `src/login_item.rs:86`, `src/login_item.rs:113`).
- `file_search` has strong platform branching and explicit unsupported errors where needed (`src/file_search.rs:811`, `src/file_search.rs:820`, `src/file_search.rs:829`, `src/file_search.rs:921`).

## Findings (Prioritized)

### P0: Webcam stack is effectively macOS-specific but compiled broadly

Evidence:

- `camera` module imports Objective-C and AVFoundation-related symbols without module-level platform gating (`src/camera.rs:12`, `src/camera.rs:73`, `src/camera.rs:255`).
- `lib.rs` exports `camera` unconditionally (`src/lib.rs:17`).
- Prompt layer imports `core_video::CVPixelBuffer` and `CaptureHandle` unconditionally (`src/prompts/webcam.rs:6`, `src/prompts/webcam.rs:14`).
- `prompts` exports webcam prompt unconditionally (`src/prompts/mod.rs:29`, `src/prompts/mod.rs:45`).
- Runtime path always attempts webcam capture via `crate::camera::start_capture` (`src/app_execute.rs:1882`).
- Photo encoding path depends on `core_video` directly in app impl (`src/app_impl.rs:4279`, `src/app_impl.rs:4283`).

Impact:

- Elevated compile/link risk for Linux/Windows targets.
- Platform behavior is undefined/inconsistent for webcam commands outside macOS.

Recommendation:

- Introduce a platform facade for webcam operations (macOS backend + non-mac stub backend).
- Gate module exports and types with `#[cfg(target_os = "macos")]`, and provide non-mac placeholder prompt/state that returns an explicit "not supported" response.
- Keep UI/action entries platform-aware so unsupported actions are omitted or disabled with a reason.

### P0: `selected_text` module is unconditionally compiled despite macOS-only dependencies

Evidence:

- Module imports macOS-only crates and APIs without top-level `cfg` (`src/selected_text.rs:18`, `src/selected_text.rs:19`, `src/selected_text.rs:239`).
- `lib.rs` exports the module unconditionally (`src/lib.rs:41`).
- Executor already has platform-specific handling and only imports module on macOS (`src/executor/selected_text.rs:11`, `src/executor/selected_text.rs:102`, `src/executor/selected_text.rs:152`).

Impact:

- Duplication of platform logic and unnecessary non-mac compile pressure.
- Potential incompatibility if dependency support shifts.

Recommendation:

- Move all selected-text platform branching behind the module boundary:
  - macOS implementation in one file.
  - non-mac stub implementation in another file.
- Keep exported API identical and platform-safe; avoid importing macOS crates in shared paths.

### P1: Path actions in `app_impl` are macOS-only with no non-mac fallback behavior

Evidence:

- `open_in_finder` branch only has `#[cfg(target_os = "macos")]` code path (`src/app_impl.rs:5585`).
- `open_in_terminal` branch only has macOS AppleScript path (`src/app_impl.rs:5645`, `src/app_impl.rs:5667`).
- `move_to_trash` execution block only has macOS implementation (`src/app_impl.rs:5772`).

Impact:

- On non-mac, these actions can silently do nothing (poor UX, hard to debug).

Recommendation:

- Route these actions through `file_search` cross-platform helpers (`src/file_search.rs:841`, `src/file_search.rs:900`).
- Where unsupported, show a deterministic HUD/toast error rather than no-op.

### P1: Duplicate platform command logic exists across modules

Evidence:

- `app_actions` has a local `reveal_in_finder` helper shelling to `open -R` (`src/app_actions.rs:319`, `src/app_actions.rs:323`).
- `file_search::reveal_in_finder` already implements per-platform behavior (`src/file_search.rs:841`).
- `prompt_handler` has direct per-platform browser command branching (`src/prompt_handler.rs:470`, `src/prompt_handler.rs:482`, `src/prompt_handler.rs:494`) while dependency `open` is available (`Cargo.toml:108`).

Impact:

- Behavior drift risk and repeated platform branching.

Recommendation:

- Consolidate OS command launch behavior behind shared utility functions.
- Prefer one abstraction for "open URL", "reveal path", "open terminal", etc.

### P2: Platform terminology and shortcut glyphs are macOS-centric in shared action builders

Evidence:

- Finder-specific labels in shared action lists (`src/actions/builders.rs:106`, `src/actions/builders.rs:191`).
- macOS keycap glyphs hardcoded in generic actions (`src/actions/builders.rs:114`, `src/actions/builders.rs:196`, `src/actions/builders.rs:210`, `src/actions/builders.rs:226`).

Impact:

- Linux/Windows users see mismatched labels and shortcut hints.

Recommendation:

- Add platform-aware display labels (Finder/File Manager/Explorer).
- Use a platform-specific shortcut formatter that maps display glyphs per OS.

### P2: `Cargo.toml` keeps several macOS-oriented dependencies unconditional

Evidence:

- Unconditional entries include `cocoa`, `objc`, `core-graphics`, `core-video`, `core-foundation`, `smappservice-rs`, `macos-accessibility-client` (`Cargo.toml:34`, `Cargo.toml:35`, `Cargo.toml:37`, `Cargo.toml:67`, `Cargo.toml:130`, `Cargo.toml:141`).

Impact:

- Extra compile surface and possible target-specific dependency breakage.

Recommendation:

- Move macOS-specific crates into `[target.'cfg(target_os = "macos")'.dependencies]`.
- Keep shared crates in top-level `[dependencies]` only when truly cross-platform.

### P2: No-op platform stubs reduce capability discoverability

Evidence:

- `platform` intentionally no-ops on non-macOS (`src/platform.rs:105`, `src/platform.rs:141`, `src/platform.rs:175`, `src/platform.rs:314`).
- `window_manager` stubs always return `None`/`false` (`src/window_manager.rs:343`, `src/window_manager.rs:350`, `src/window_manager.rs:362`).

Impact:

- Callers cannot tell "unsupported" vs "failed" without extra logging and ad hoc checks.

Recommendation:

- Add a small capabilities API (`supports_window_space_ops`, `supports_selected_text`, `supports_webcam_capture`).
- Use typed errors (`UnsupportedPlatform`) for user-facing actions instead of silent no-op.

## Recommended Implementation Plan

### Phase 1 (High Impact, Low-Medium Effort)

1. Guard webcam/selected-text modules behind platform-safe facades.
2. Route `open_in_finder`, `open_in_terminal`, `move_to_trash` through shared cross-platform helpers.
3. Add explicit non-mac HUD/toast messages for unsupported operations.

### Phase 2 (Stability + Maintainability)

1. Consolidate duplicated platform shell commands into one module.
2. Move macOS-only dependencies to target-specific dependency sections.
3. Add platform-aware action labels and shortcut display mapping.

### Phase 3 (Compatibility Regression Prevention)

1. Add CI target checks:
   - `cargo check --target aarch64-apple-darwin`
   - `cargo check --target x86_64-unknown-linux-gnu`
   - `cargo check --target x86_64-pc-windows-msvc`
2. Add tests for platform capability routing (especially non-mac fallback behavior).

## Validation Gaps Noted

- Current audit was static analysis only; no non-mac target checks were executed in this pass.
- A follow-up implementation PR should include target-based `cargo check` in CI before merging.
