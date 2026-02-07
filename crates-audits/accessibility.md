# Accessibility + Selected Text Crates Audit

## Scope
- Repository: `script-kit-gpui`
- Audit date: 2026-02-07
- Requested crates: `get-selected-text` (`0.1`), `macos-accessibility-client` (`0.0.1`)
- Files reviewed: `Cargo.toml`, `Cargo.lock`, `src/selected_text.rs`, `src/executor/selected_text.rs`, `src/app_execute.rs`, `src/executor/scriptlet.rs`, `scripts/kit-sdk.ts`, smoke/SDK tests, and crate sources under `~/.cargo/registry/src/...`

## Dependency Baseline
- Declared:
  - `get-selected-text = "0.1"` (`Cargo.toml:60`)
  - `macos-accessibility-client = "0.0.1"` (`Cargo.toml:63`)
- Resolved:
  - `get-selected-text 0.1.6` (`Cargo.lock:2881`-`Cargo.lock:2883`)
  - `macos-accessibility-client 0.0.1` (`Cargo.lock:4469`-`Cargo.lock:4471`)

## Direct Answers

### 1) Is `get-selected-text` hybrid AX + clipboard working reliably?
**Partially.**

What works:
- First-attempt strategy is AX-first with clipboard fallback if AX fails (`get-selected-text-0.1.6/src/macos.rs:31`-`47`).
- SDK hides Script Kit window before requesting text and waits 20ms for focus transfer (`scripts/kit-sdk.ts:4800`-`4808`).
- Runtime smoke flow showed success with empty selection (not crash/error):
  - `GetSelectedText request`
  - `GetSelectedText success: 0 chars`

Reliability risk identified:
- Per-app method cache can become brittle:
  - Cache chooses AX (`0`) or clipboard (`1`) by app name (`.../macos.rs:11`, `20`, `25`-`30`).
  - Once cached, it does **not** try the other method on future failures (`.../macos.rs:25`-`30`).
  - Fallback only happens on uncached first path (`.../macos.rs:31`-`47`).
- This can fail in real app state changes (focused field type changes, temporary AX failures, clipboard policy changes).

Additional mismatch:
- Local docs claim an AX range fallback (`AXSelectedTextRange + AXStringForRange`) in `src/selected_text.rs:127`-`130`, but current crate code only uses `kAXSelectedTextAttribute` then AppleScript clipboard (`.../macos.rs:66`-`79`, `119`-`139`).

### 2) Are accessibility permissions checked before use?
**Yes, in app code paths.**

- Permission gate before read: `src/selected_text.rs:143`-`145`.
- Permission gate before write: `src/selected_text.rs:192`-`194`.
- Permission APIs are exposed and routed in executor:
  - `CheckAccessibility`: `src/executor/selected_text.rs:169`-`185`
  - `RequestAccessibility`: `src/executor/selected_text.rs:205`-`220`
- `macos-accessibility-client` is a thin wrapper over `AXIsProcessTrusted` and `AXIsProcessTrustedWithOptions` (`macos-accessibility-client-0.0.1/src/lib.rs:27`-`43`).

Important nuance:
- `get-selected-text` crate itself does not perform permission checks; it assumes callers gate usage. In this repo, callers do gate via `selected_text` wrapper.

### 3) Are edge cases handled (no focused app, no selection)?
**No selection: handled. No focused app: treated as error.**

No selection:
- Wrapper returns `Ok("")` on empty result (`src/selected_text.rs:155`-`158`).
- AI command path shows friendly toast for empty text (`src/app_execute.rs:718`-`727`).
- Smoke run confirms this flow (`GetSelectedText success: 0 chars` and HUD `No text selected`).

No focused app:
- Crate returns `Err("No active window found")` (`get-selected-text-0.1.6/src/macos.rs:20`-`23`).
- Wrapper bubbles this as error (`src/selected_text.rs:163`-`166`).
- UX currently surfaces a generic failure toast in AI command flow (`src/app_execute.rs:729`-`742`).

## Test/Runtime Verification Performed
- Ran stdin JSON smoke tests with compact logs:
  - `tests/smoke/test-get-selected-text.ts`
  - `tests/smoke/test-accessibility-check.ts`
- Observed expected permission check + selected text calls and empty-selection handling.
- Build status in shared tree:
  - `cargo check` currently fails due unrelated in-progress work in `src/hotkeys.rs` (type mismatches with `Option<HotkeyConfig>`), not in selected-text modules.

## Recommendations (Priority)
1. Make cached method resilient in `get-selected-text` usage path.
- If cached AX fails, try clipboard once and update cache.
- If cached clipboard fails, try AX once and update cache.

2. Improve no-focus UX handling.
- Map known `"No active window found"` to a user-friendly message or empty result in non-critical flows.
- Keep hard error behavior for flows that must enforce active selection.

3. Align local docs with actual crate behavior.
- Update `src/selected_text.rs` comments that currently claim AX range fallback unsupported by current crate code.

4. Add targeted regression tests around edge behavior.
- Add a focused-app-missing scenario test (expected error/message contract).
- Add cache-failure recovery tests (requires mockable selected-text backend or integration harness).

## Bottom Line
- Permission gating is implemented correctly in repository code.
- Empty selection is handled well.
- The main reliability gap is method-cache behavior in `get-selected-text` once an app is cached to a single strategy.
