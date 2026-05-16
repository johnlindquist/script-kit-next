# 024 Confirm Prompt and Dialogs

```text
[confirm-prompt-dialogs-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 024: Confirm Prompt and Dialogs / SDK `confirm()` / in-window confirm state / parent confirm popup fallback.

This atlas must cover:

- TypeScript SDK API: `confirm()`, `confirm(message)`, `confirm(message, confirmText, cancelText)`, `confirm(config)`, `ConfirmConfig`, default text, return shape, auto-submit value, and cancellation behavior.
- Protocol message path: SDK `type: "confirm"` message, Rust prompt message routing, response channel behavior, and failing-closed behavior if a dialog cannot open.
- In-window confirm state: `AppView::ConfirmPrompt`, `ParentConfirmOptions`, previous-view restoration, sender resolution, footer Apply/Close mapping, focused-button model, key handling, and surface contract.
- Parent/native confirm popup fallback: `confirm_with_parent_dialog`, router registration, `open_parent_confirm_dialog`, attached popup window, focus colors, danger semantics, button routing, and close/teardown.
- Callers beyond SDK confirm: destructive built-ins, clipboard/file/script actions, notes delete/rename/create flows, ACP/chat delete actions, sharing trust prompt, and any Test Confirmation built-in.
- Keyboard and focus behavior: Enter, Escape, Tab, arrows/Space if implemented, physical vs `simulateKey`, stop propagation, popup-first routing, parent/child focus restoration, and launcher leakage prevention.
- Footer/windowing behavior: in-window confirm uses native footer; popup fallback is a separate attached popup; main-window visibility/root-context decides route; hide/reset paths remove stale `confirm-popup` registry entries.
- Automation/protocol surface: `getState`, `getElements`, `listAutomationWindows`, semantic ids (`panel:confirm-dialog`, `button:0:confirm`, `button:1:cancel`), `selectByValue`/batch helpers, target identity, and surface tags.
- Visual states: normal/destructive labels, focused button, keycap style, title/body rendering, empty/long text behavior, selected flags, and design-story coverage.
- Data/privacy boundaries: prompt message text, button labels, destructive action labels, no persistence except caller action, and automation exposure.
- Error/cancel/disabled states: missing labels, no main window, hidden main, dialog open while app hides, sender channel closed, user cancel, Escape, timeout/auto-submit fallback, and repeated key safety.

Important boundaries:

- Feature 016 covers the core prompt runtime; this feature should focus on confirm-specific API, view state, popup routing, and action confirmation behavior.
- Destructive built-in execution itself belongs to built-in/system action features; include only the confirm contract and handoff/result semantics needed for user understanding.
- Notes, ACP, sharing, clipboard, and file actions are adjacent feature owners; include their confirm entry points and return contracts without expanding their full domain.
- Dictation's "Confirming" overlay phase is not this confirm dialog feature; mention only as a naming boundary if needed.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: actions-popups, prompt-runtime, keyboard-focus-routing, protocol-automation, agentic-testing, launcher-surface-contracts
- `lat.md`: design, surfaces, automation, protocol, builtins, sharing, storybook, verification
- Source: SDK confirm declaration/runtime, protocol confirm variant, prompt handler confirm route, confirm module, in-window route, app view state, confirm render/footer, key routing, automation collector/registry, built-in confirmation, action callers, notes/chat/sharing callers, lifecycle hide teardown
- Tests/stories: confirm focus/tab/screenshot smoke scripts, semantic id tests, main-surface rekey tests, footer owner tests, hide-path confirm registry teardown, storybook confirm states, built-in confirmation source contracts

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 024 Confirm Prompt and Dialogs / confirm()

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
```
