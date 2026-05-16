# 022 Hotkey Prompt

```text
[hotkey-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 022: Hotkey Prompt / `hotkey()` / keyboard shortcut capture, including the current unimplemented SDK prompt status and the adjacent implemented shortcut recorder system.

This atlas must cover:

- TypeScript SDK API: `hotkey(placeholder?)`, `HotkeyInfo` shape, warning/stub behavior, and tests/docs that describe intended behavior.
- Current product truth: SDK `hotkey()` appears not implemented in GPUI; Rust has a coming-soon prompt path rather than a full `AppView::HotkeyPrompt`.
- Adjacent implemented shortcut recorder: modal/popup capture behavior, assigning shortcuts from actions/menu rows, config.ts update scripts, refresh/reload behavior, validation, duplicate handling, and hotkey registration updates.
- Distinguish SDK `hotkey()` prompt from app shortcut assignment and global hotkey registration.
- State/UI behavior for the coming-soon path: toast/warning, prompt id handling, SDK return/fallback if any, and unimplemented tests.
- Protocol and automation receipts for any hotkey/shortcut recorder surfaces: simulateKey, physical key capture, popup focus, shortcut display, and config refresh receipts.
- Data/privacy boundaries: captured key combinations, config.ts mutation, global hotkey registration, logs, and user shortcuts.
- Error, empty, unsupported, duplicate shortcut, reserved shortcut, invalid modifier, cancellation, and refresh-failure states.

Important boundaries:

- This feature covers SDK-visible `hotkey()` status and the adjacent shortcut recorder because user explicitly wants assigning shortcuts/config refresh interactions captured.
- Main Menu shortcut assignment is already covered in 001, but this chapter should cross-reference and deepen the hotkey/recorder boundary.
- Do not claim SDK `hotkey()` works if current source shows it is a stub.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing, keyboard-focus-routing, theme-config-preferences
- `lat.md`: scripting, protocol, surfaces, verification, design, builtins
- Source: SDK `hotkey()`, prompt handler hotkey/coming-soon route, prompt messages, protocol constructor, shortcut recorder component/app impl, config shortcut update scripts, hotkey registry, shortcuts types/compat, actions shortcut handlers, refresh scriptlets, simulateKey
- Tests/reports: SDK hotkey test, shortcut recorder smoke/source audits, shortcut config source audits, action shortcut aliases, shortcut error tests, menu shortcut transitions agentic script

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 022 Hotkey Prompt / hotkey()

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
