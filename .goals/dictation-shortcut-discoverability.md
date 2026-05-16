# Dictation Shortcut Discoverability

## Objective

Make the live dictation overlay visibly surface its core controls and keyboard shortcuts by running an Oracle-backed implementation loop with project-appropriate verification.

**This loop is mandatory. `$oracle-packx` and project-appropriate verification are HARD REQUIREMENTS, not suggestions. Do not implement before Oracle research. Do not declare done before the dictation UI is verified with the best checks available in this project.**

## Feature Brief




Run `source context expansion` on this goal or the user prompt before coding. Run `source search "dictation UI shortcuts submit cancel overlay visual controls"` first; if the current `source search` embedding path fails on an overlong section, record the exact failure and fall back to `source lookup Dictation`, direct reads of `removed-docs`, `removed-docs`, `removed-docs`, `removed-docs`, and `removed-docs`.


- `tests/dictation_setup_nux_contract.rs`, `tests/dictation_tab_ai_harness.rs`, `tests/push_dictation_result_stub_contract.rs`, and `tests/portal_dictation_roundtrip_contract.rs` for adjacent delivery/setup safety.

Select the smallest verification plan that can fail if shortcuts are still hidden, clipped, or wrong. Prefer source contracts for ownership invariants, compile checks for GPUI render edits, and state-first or visual runtime proof for the overlay itself. Because this is a visual/discoverability change, runtime visual proof is required before completion.


- selected verification commands or proof routes
- why those checks cover visible shortcut/action discoverability
- known blockers such as the current `source search` overlong-section failure or the known local lib-test SIGBUS path




- the recommended overlay layout for always-visible shortcuts and actions across recording, confirming, transcribing, delivering, finished, and failed phases
- concrete changes to overlay dimensions, placement, radius, spacing, text sizing, and responsive fallback
- which keyboard/action labels should be visible in each phase and how they should map to existing behavior
- exact source contracts, compile checks, runtime receipts, screenshots, and cleanup receipts required to prove the change

Oracle must use browser mode and the remote browser config in `~/.oracle/config.json`; an `OPENAI_API_KEY` failure means the call used the wrong engine/default and must be retried through `$oracle-packx` browser mode. Oracle must return text only. The local agent owns any `.goals`, notes, code, commits, and verification logs.

Do **not** implement from local speculation before Oracle has reviewed the project context.




- Recording state shows the active recording affordance plus visible shortcut/action labels for submit/finalize, stop/cancel behavior, Enter behavior if applicable, and destination cycling when multiple targets are available.
- Confirming state clearly shows Stop and Continue with their keyboard shortcuts and mouse affordances without relying on hidden comments or tiny truncated chips.
- Transcribing, delivering, finished, and failed states show the available close/cancel shortcut when it exists.
- The target badge remains readable and, when cycling is available, visibly interactive.
- The overlay has enough width and/or height for all required controls without clipping, truncating important action copy, or overlapping timer/waveform/status/target controls.
- Small-display placement remains bottom-anchored and does not push the overlay offscreen.
- Storybook/preview helpers stay visually consistent with the runtime overlay.
- Existing keyboard behavior remains unchanged unless Oracle and local evidence identify a deliberate product change.

If local evidence contradicts Oracle, document the contradiction and adjust deliberately.



```bash
cargo test --test dictation_overlay_focus_hide_contract -- --nocapture
cargo test --test acp_dictation_keyboard_contract -- --nocapture
cargo check --lib
cargo fmt --check
git diff --check
source checks
```

Add a focused new source-contract test if the implementation introduces new layout constants, visible shortcut components, phase-action mapping, or preview parity that can regress without compile failure.


- start a dictation session through the real built-in path or the repo's agentic/devtools helper
- capture `getState`, `getElements`, `listAutomationWindows`, or `inspectAutomationWindow` receipts for the overlay if available
- capture a screenshot or strict-window visual artifact of recording and confirming states
- verify visible text includes the required shortcuts/actions and that timer, waveform/status, target badge, and shortcuts do not overlap or clip
- exercise Escape/Enter through the existing protocol/native path only enough to prove labels match behavior

If existing automation cannot inspect the dictation overlay, add the smallest missing receipt or explicitly document the gap and use screenshot proof plus source contracts. Do not treat screenshot proof alone as enough if state/elements instrumentation is available.

## Loop Rule


- this feature brief
- Oracle's prior recommendation
- implementation diff
- failed verification output or proof artifact
- screenshots, receipts, logs, or blocker details
- current hypothesis about the failure

Ask Oracle for the next feature plan, then repeat Implement and Verify. Do not keep guessing locally after failed verification.

## Done Criteria

- Remaining risk is documented explicitly.
