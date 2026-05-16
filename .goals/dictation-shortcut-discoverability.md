# Dictation Shortcut Discoverability

## Objective

Make the live dictation overlay visibly surface its core controls and keyboard shortcuts by running an Oracle-backed implementation loop with project-appropriate verification.

**This loop is mandatory. `$oracle-packx` and project-appropriate verification are HARD REQUIREMENTS, not suggestions. Do not implement before Oracle research. Do not declare done before the dictation UI is verified with the best checks available in this project.**

## Feature Brief

- Outcome: The dictation UI must show discoverable shortcuts and actions for submit, cancel, continue, stop, destination cycling, and close states instead of hiding them in tiny or transient copy.
- User value: A user recording dictation should be able to understand what will happen on Enter, Escape, mouse click, and destination change without guessing.
- Current gap: The overlay is a compact 312x32 capsule in `src/dictation/window.rs`; recording shows timer, waveform, and target badge, while confirming swaps in small Stop/Continue chips. The shortcut copy exists in constants such as `overlay_phase_copy`, but the primary runtime render does not consistently surface it.
- Entry path: Built-in Dictation and Dictation-to-AI flows through `BuiltInFeature::Dictation`, `BuiltInFeature::DictationToAiHarness`, `toggle_dictation`, and `open_dictation_overlay`.
- Scope: `src/dictation/window.rs`, `src/dictation/runtime.rs` only if state receipts are needed, dictation-focused tests under `tests/`, dictation storybook/preview render paths if present, and `lat.md/` pages for changed behavior or verification contracts.
- Constraints: Preserve the existing nonactivating popup lifecycle, hidden-main behavior, global Escape/Enter monitor behavior, target delivery semantics, ACP composer delivery no-auto-submit contract, and setup/download behavior. Do not change transcription/history privacy or microphone permission semantics.
- Non-goals: Do not redesign dictation setup, model download, dictation history, ACP attachment portals, or global hotkey configuration except where a shortcut label needs to reflect an existing binding.
- Research questions: What overlay dimensions can safely fit all controls without clipping on small displays? Should shortcuts be inline chips, a second row, or a compact toolbar? Which phases need which controls? What state/elements/screenshot receipts already expose the overlay, and what minimal instrumentation is missing?

## Discovery: Choose The Verification Surface

**HARD REQUIREMENT: Inspect the current project before implementation.** Start with `AGENTS.md`, `CLAUDE.md`, `.agents/skills/dictation-media/SKILL.md`, `.agents/skills/agentic-testing/SKILL.md`, `.agents/skills/protocol-automation/SKILL.md`, `.agents/skills/testing-quality-gates/SKILL.md`, and `.agents/skills/lat-md/SKILL.md`.

Run `lat expand` on this goal or the user prompt before coding. Run `lat search "dictation UI shortcuts submit cancel overlay visual controls"` first; if the current `lat search` embedding path fails on an overlong section, record the exact failure and fall back to `lat locate Dictation`, direct reads of `lat.md/acp-chat.md`, `lat.md/protocol.md`, `lat.md/automation.md`, `lat.md/verification.md`, and `lat.md/tests/acp-dictation.md`.

Relevant current source and contracts to inspect:

- `src/dictation/window.rs`: overlay geometry constants, phase copy, global Escape/Enter monitor, render tree, target badge, Storybook preview helper, and popup lifecycle.
- `src/dictation/runtime.rs`: active session phase, target cycle, and overlay snapshot state.
- `src/app_execute/builtin_execution.rs`: dictation start/stop edge, model/setup preflight, delivery target resolution, and overlay session start.
- `tests/dictation_overlay_focus_hide_contract.rs`: nonactivating popup and hidden-main contract.
- `tests/acp_dictation_keyboard_contract.rs`: ACP close/keyboard contracts after dictation delivery.
- `tests/dictation_setup_nux_contract.rs`, `tests/dictation_tab_ai_harness.rs`, `tests/push_dictation_result_stub_contract.rs`, and `tests/portal_dictation_roundtrip_contract.rs` for adjacent delivery/setup safety.

Select the smallest verification plan that can fail if shortcuts are still hidden, clipped, or wrong. Prefer source contracts for ownership invariants, compile checks for GPUI render edits, and state-first or visual runtime proof for the overlay itself. Because this is a visual/discoverability change, runtime visual proof is required before completion.

Record:

- available relevant skills: `$dictation-media`, `$agentic-testing`, `$protocol-automation`, `$testing-quality-gates`, `$lat-md`, and `$window-resizing` if overlay bounds or display placement change
- selected verification commands or proof routes
- why those checks cover visible shortcut/action discoverability
- known blockers such as the current `lat search` overlong-section failure or the known local lib-test SIGBUS path

## Oracle: Research The Feature Plan

**HARD REQUIREMENT: Use `$oracle-packx` before implementation.** Bundle this goal, `AGENTS.md`/`CLAUDE.md`, `.agents/skills/dictation-media/SKILL.md`, relevant verification/protocol/agentic skills, `lat.md/acp-chat.md`, `lat.md/protocol.md`, `lat.md/automation.md`, `lat.md/verification.md`, `lat.md/tests/acp-dictation.md`, and the relevant source/tests above.

Ask Oracle for:

- the recommended overlay layout for always-visible shortcuts and actions across recording, confirming, transcribing, delivering, finished, and failed phases
- concrete changes to overlay dimensions, placement, radius, spacing, text sizing, and responsive fallback
- which keyboard/action labels should be visible in each phase and how they should map to existing behavior
- invariants that must not regress: nonactivating popup lifecycle, hidden-main behavior, target delivery, setup privacy, ACP no-auto-submit, and global Escape/Enter monitor behavior
- exact source contracts, compile checks, runtime receipts, screenshots, and cleanup receipts required to prove the change

Oracle must use browser mode and the remote browser config in `~/.oracle/config.json`; an `OPENAI_API_KEY` failure means the call used the wrong engine/default and must be retried through `$oracle-packx` browser mode. Oracle must return text only. The local agent owns any `.goals`, notes, code, commits, and verification logs.

Do **not** implement from local speculation before Oracle has reviewed the project context.

## Implement: Build Oracle's Plan

**HARD REQUIREMENT: Implement from the Oracle-backed plan.** Use the current dictation overlay ownership boundaries and GPUI style.

Implementation must satisfy these product requirements:

- Recording state shows the active recording affordance plus visible shortcut/action labels for submit/finalize, stop/cancel behavior, Enter behavior if applicable, and destination cycling when multiple targets are available.
- Confirming state clearly shows Stop and Continue with their keyboard shortcuts and mouse affordances without relying on hidden comments or tiny truncated chips.
- Transcribing, delivering, finished, and failed states show the available close/cancel shortcut when it exists.
- The target badge remains readable and, when cycling is available, visibly interactive.
- The overlay has enough width and/or height for all required controls without clipping, truncating important action copy, or overlapping timer/waveform/status/target controls.
- Small-display placement remains bottom-anchored and does not push the overlay offscreen.
- Storybook/preview helpers stay visually consistent with the runtime overlay.
- Existing keyboard behavior remains unchanged unless Oracle and local evidence identify a deliberate product change.

If local evidence contradicts Oracle, document the contradiction and adjust deliberately.

## Verify: Prove The Feature

**HARD REQUIREMENT: Run the selected project-appropriate verification after implementation.** At minimum, verification must include:

```bash
cargo test --test dictation_overlay_focus_hide_contract -- --nocapture
cargo test --test acp_dictation_keyboard_contract -- --nocapture
cargo check --lib
cargo fmt --check
git diff --check
lat check
```

Add a focused new source-contract test if the implementation introduces new layout constants, visible shortcut components, phase-action mapping, or preview parity that can regress without compile failure.

Because this is visual UI work, also capture real overlay proof against the running app. Prefer state-first receipts before screenshots:

- start a dictation session through the real built-in path or the repo's agentic/devtools helper
- capture `getState`, `getElements`, `listAutomationWindows`, or `inspectAutomationWindow` receipts for the overlay if available
- capture a screenshot or strict-window visual artifact of recording and confirming states
- verify visible text includes the required shortcuts/actions and that timer, waveform/status, target badge, and shortcuts do not overlap or clip
- exercise Escape/Enter through the existing protocol/native path only enough to prove labels match behavior
- stop the session and verify `windowVisible:false` or equivalent cleanup/session stopped receipt for anything launched

If existing automation cannot inspect the dictation overlay, add the smallest missing receipt or explicitly document the gap and use screenshot proof plus source contracts. Do not treat screenshot proof alone as enough if state/elements instrumentation is available.

## Loop Rule

**HARD REQUIREMENT: If implementation or verification fails, stop broadening locally. Return to `$oracle-packx` with:**

- this feature brief
- Oracle's prior recommendation
- implementation diff
- failed verification output or proof artifact
- screenshots, receipts, logs, or blocker details
- current hypothesis about the failure

Ask Oracle for the next feature plan, then repeat Implement and Verify. Do not keep guessing locally after failed verification.

## Done Criteria

- Required: Project instructions, skills, source owner, and verification surface inspected.
- Required: Oracle research completed before implementation.
- Required: Dictation overlay visibly surfaces shortcuts/actions for recording, confirming, transcribing/delivering, finished, and failed states.
- Required: Overlay dimensions/layout leave enough room for those controls without clipping or overlap on normal and small-display bounds.
- Required: Existing dictation delivery, ACP composer no-auto-submit, setup/download, and nonactivating popup lifecycle contracts still pass.
- Required: Real runtime visual proof shows the shortcuts/actions on the live overlay.
- Required: Project checks pass, including `lat check`.
- Required: `lat.md/` is updated if behavior, architecture, tests, or verification contracts change.
- Required: Any launched sessions, app windows, or temporary proof artifacts are stopped or intentionally documented.
- Remaining risk is documented explicitly.
