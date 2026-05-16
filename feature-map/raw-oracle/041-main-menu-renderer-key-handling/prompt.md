# Oracle Prompt: 041 Main Menu Renderer Key Handling

[main-menu-renderer-keys]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK scripts, a stdin JSON automation protocol, `lat.md` architecture docs, and repo-local `.agents/skills` ownership rules.

Goal: produce a complete operator-grade feature atlas chapter for feature `041-main-menu-renderer-key-handling`. This is a follow-up to existing feature-map chapters 001, 011, 012, 013, and 022. Do not repeat their broad coverage unless needed for context. Focus on the specific open gap from chapter 001:

- full `src/render_script_list/mod.rs` key handling after the clipped previous bundle area
- exact Cmd+Enter behavior
- non-file Tab behavior
- action shortcut execution from the main list and actions popup
- popup-first ordering between menu syntax popups, actions dialogs, shortcut recorder, and main-list keys
- physical-vs-simulated key parity where relevant
- how key handling resolves the selected visible row versus stale selection/cache state
- how failures should be verified with state-first receipts

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.

Use this output shape:

## 041 Main Menu Renderer Key Handling

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

Bundle map: the attached bundle includes repo process docs, the relevant owner skills, existing feature-map chapters that identify the gap, `lat.md` pages for surfaces/automation/verification/menu syntax/shortcuts, main ScriptList renderer/key handling source, action popup and shortcut code, protocol simulate-key handling, and tests/source audits around action shortcuts and menu/filter behavior.
