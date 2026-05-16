# 019 Path Prompt

```text
[path-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 019: Path Prompt / `path()` / file and directory selection / path actions.

This atlas must cover:

- TypeScript SDK API: `path(options?)` and all visible PathOptions fields.
- Current product truth for `PathPrompt`: AppView routing, prompt ids, submit callbacks, file/directory mode, selection semantics, focus state, native footer surface, and window sizing.
- PathPrompt internals: current directory, selected entry, typed path input if present, file rows, directory rows, hidden files, parent/up navigation, search/filter behavior, extension filtering, and empty/error states.
- Footer and command routing: Select/Run labeling, Enter handling, Cmd+K actions, path prompt action dispatcher, safe typed action ids, and no launcher fallthrough.
- Path actions: select file, copy path, reveal/open, move to trash, Quick Terminal path action if relevant, action registry teardown, and destructive confirmation boundaries.
- Relationship to File Search: shared file row behavior, full file search, root files source, `~` trigger, attachment portals, drag-out rows, and what is separate from SDK `path()`.
- Protocol and automation receipts: getState, getElements, simulateKey, ForceSubmit if supported, select row, path prompt element semantics, and path-action receipts.
- Data/privacy boundaries: filesystem paths, selected file names, hidden paths, action payloads, destructive actions, and logs/screenshots.
- Error, empty, loading, permission-denied, missing directory, hidden-file, unsupported option, and cancellation states.

Important boundaries:

- Feature 002 covers File Search / Browser / Attachment Portals.
- Feature 013 covers first-character route handoffs such as `~`.
- Feature 016 covers `arg()`, `select()`, `div()`, and `md()`.
- Feature 017 covers `form()` and `fields()`.
- Do not claim file-search behavior belongs to SDK `path()` unless current source proves the same route or shared helper.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing, file-search-portals
- `lat.md`: scripting, protocol, surfaces, builtins, verification, design, acp-chat
- Source: SDK `path()`, prompt handler path route, app view state, render dispatch, PathPrompt entity/render/types, path render prompt wrapper, file search shared modules, path action dispatcher, UI window footer, focus coordinator, element collection, simulateKey
- Tests/reports: SDK path test, path key events, path actions visual, path visual consistency, protocol submit, path action registry teardown contracts, file search action/path audits

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 019 Path Prompt / path()

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
