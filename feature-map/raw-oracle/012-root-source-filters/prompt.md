[root-source-filters-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 012: Root Unified Source Filters / source-chip query routing and lazy row paging.

This atlas must cover how root launcher source filters work for every committed source head and source-only browse state:

- Files: `files:` / `f:`
- Notes: `notes:` / `n:`
- Clipboard History: `clipboard:` / `c:`
- Browser Tabs: `tabs:` / `t:`
- Browser History: `history:` / `h:`
- Apps: `apps:` / `a:`
- Scripts: `scripts:` / `s:`
- Commands: `commands:` / `cmd:`
- AI Conversations: `conversations:` / `ai:`
- AI Vault: `vault:` and related aliases if implemented
- Dictation History: `dictation:` / `d:`
- Windows: `windows:` / `w:`
- Processes: `processes:` / `p:` as uncommitted/planned if that is the current contract

Map attached query syntax (`c:skip`, `files:s`), spaced syntax (`c: skip`), anywhere standalone source tokens (`png files:`), source-only browse (`f: `, `n: `, `c: `, `ai: `, `d: `, etc.), quoted/unknown heads, exclusion semantics, source-filter frame keys, source-chip/status rows, lazy source paging, source-filter suppression of primary/fallback rows, input-history blocking, main-window preflight receipts, getElements status rows, and source-specific proof scripts.

Important known requirements from current docs:

- Leading `:` is source discovery/filter discovery, not committed source syntax.
- Completed source heads must not open the unrelated menu-syntax power hint.
- Source-filter mode suppresses primary/fallback rows and disallowed sources.
- Positive source heads explicitly enable their source for the active stripped query, even if ordinary passive defaults are disabled.
- Source-only browse keeps empty stripped search text and returns the source's default browse rows where implemented.
- Files source filters can start at a 12-row source-chip page and expand near the bottom without Enter.
- Explicit Files source filters allow one-character ASCII alphanumeric stripped queries such as `f:s`, while plain `s` remains below the ordinary global file threshold.
- Source status metadata must be non-selectable and excluded from ScriptList row counts, scroll height, executable results, and action subjects, while still visible to `getElements` as status/sourceStatus.
- Source-filter mode blocks launcher input-history recall so Up/Down remain row navigation.
- Root file/passive frame keys must include the source-filter set so async provider/cache results cannot bleed across source modes.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: main-menu-search-selection, actions-popups, protocol-automation, agentic-testing, theme-config-preferences
- `lat.md`: builtins, menu-syntax, verification, surfaces, automation
- Source: menu source heads/payload/query/hints, filtering cache, root file search, filter input, grouping, result types, preflight, list/status row support, config
- Tests/scripts: menu syntax source filter tests, source audits, passive snapshot/stability/config audits, source-filter agentic scripts and source-chip pagination proof

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 012 Root Unified Source Filters / Source Chips / Lazy Paging

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
