[shortcut-refresh-atlas]

Project briefing:

This is Script Kit GPUI, a Rust/GPUI desktop app with a Bun/TypeScript SDK and config tooling. The repository uses `lat.md/` as the architecture/test knowledge graph. Repo rules require `lat expand`, `lat search`, relevant skill context, `lat.md/` updates for behavior/docs changes, and `lat check`. This feature-map project is building an operator-grade atlas for humans and AI agents. Every Oracle answer is preserved under `feature-map/raw-oracle/<feature-id>/`, then locally distilled into `feature-map/features/<feature-id>.md`.

Feature id:

`046-shortcut-assignment-config-refresh`

Goal:

Produce a complete operator-grade feature atlas chapter for shortcut assignment, shortcut removal, config-backed command shortcuts, `config.ts` mutation, refresh/reload behavior, live hotkey registration, shortcut display, conflicts, and verification. This is a focused follow-up because the broader main-menu and hotkey chapters are not enough to fully explain assigning shortcuts from launcher/actions and how those writes update `~/.scriptkit/config.ts` and the app.

Current evidence:

- `lat expand "046 Shortcut assignment config refresh: assign shortcuts from main menu actions config.ts update reload hotkeys shortcut recorder remove shortcut duplicate conflict refresh app"` was run.
- `lat search "shortcut assignment config.ts refresh app main menu actions assign remove shortcut duplicate conflict hotkey reload"` returned:
  - `lat.md/shortcuts#Shortcuts#Removal Writes`
  - `lat.md/shortcuts#Shortcuts#Key Facts`
  - `lat.md/shortcuts#Shortcuts#Recorder Writes`
  - `lat.md/tests/dictation-setup-nux#Dictation Setup NUX#Hotkey guidance does not invent default`
  - `lat.md/shortcuts#Shortcuts`
- The current shortcut contract says launcher shortcuts have one durable user-owned source: `~/.scriptkit/config.ts`.
- Command-specific launcher shortcuts are read from `config.ts.commands[commandId].shortcut`.
- Script and scriptlet metadata shortcuts remain defaults; `config.ts.commands` wins over metadata for the same command ID.
- `shortcuts.json` is legacy only and must not be an active source.
- Recorder saves call `scripts/update-config-shortcut.ts`, wrapping `scripts/config-cli.ts set-command-shortcut`.
- Removal calls `scripts/remove-config-shortcut.ts`, wrapping `scripts/config-cli.ts remove-command-shortcut`.

Bundle map:

The attached bundle includes process docs, the goal file, owning skills, `lat.md/shortcuts.md`, config CLI/wrappers/tests, shortcut recorder implementation, shortcut action handlers, shortcut type/compat modules, command id helpers, source-audit tests, popup-window contract tests, error-message tests, config-fingerprint tests, and adjacent feature-map chapters where shortcut behavior was previously mentioned.

Please map what is present in the attached repo snapshot. Mark uncertain claims as inference. Do not invent behavior. If a user story or proof path is not implemented, call it a gap.

Required deliverable:

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

```markdown
## 046 Shortcut Assignment And Config Refresh

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

### Config Write And Refresh Semantics

### Command IDs And Source Priority

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, Conflict, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps
```

Specific questions to answer:

1. How does a user assign a shortcut from the main menu or actions surfaces?
2. Which rows/actions expose assignment or removal, and which row types should not?
3. What is the shortcut recorder UI state machine from open -> recording modifiers/key -> conflict/no conflict -> save/cancel/clear?
4. How are shortcuts represented in recorder state, display glyphs, config payloads, `config.ts`, and row/action hints?
5. What exact command IDs are used for scripts, scriptlets, built-ins, and apps?
6. What writes to `~/.scriptkit/config.ts`, and what fields are preserved or removed?
7. What happens after a successful write: live hotkey registration, app refresh, menu row refresh, config fingerprint, or restart-required behavior?
8. What conflict detection exists, if any, and what remains unproven?
9. How do config-backed shortcuts override script/scriptlet metadata shortcuts?
10. What exactly is legacy about `shortcuts.json`, and what must not read/write it?
11. How are removals handled when a command has sibling config fields like `hidden` or `confirmationRequired`?
12. What error states can occur from bad keys, wrapper failures, config parse/write problems, missing command IDs, hidden commands, or conflicts?
13. How should agents verify this feature with source tests, Bun tests, protocol receipts, config fingerprint, and state-first runtime proof?
14. What unsafe claims should feature-map chapters avoid?

Repo-specific rules:

- Include the `lat.md/` update rule and required `lat check` in the verification plan.
- Treat screenshots as secondary; prefer state-first receipts where possible.
- Preserve raw Oracle output separately from the distilled feature chapter.
- Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
