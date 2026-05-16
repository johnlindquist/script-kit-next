# 020 Drop Prompt

```text
[drop-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 020: Drop Prompt / `drop()` / drag-and-drop file input / dropped file metadata submission.

This atlas must cover:

- TypeScript SDK API: `drop()` and its returned `FileInfo[]` shape.
- Current product truth for `DropPrompt`: AppView routing, prompt ids, submit callbacks, empty state, dropped file state, focus state, native footer surface, window sizing, and footer disabled state.
- DropPrompt internals: drop target rendering, file list rendering, dropped file metadata, drag/drop event handling, submit behavior, cancellation behavior, and empty-submit prevention.
- Footer and command routing: Submit/Run labeling, disabled `actionDisabled:"no_files"` state when empty, Cmd+K Actions, omitted launcher AI, and no launcher fallthrough.
- Actions and menus: whether DropPrompt supports actions, how footer Actions is wired, action host support if present, and any gaps between footer affordance and implementation.
- Relationship to other drag/drop flows: File Search drag-out rows, ACP/chat file drops, image drops, permission assistant drag source, and what is separate from SDK `drop()`.
- Protocol and automation receipts: getState, getElements, simulateKey, ForceSubmit if supported, dropped file element semantics, submit receipts, and empty-state receipts.
- Data/privacy boundaries: file paths, file names, sizes, MIME/type fields, selected files, logs/screenshots, and path exposure.
- Error, empty, disabled, loading, permission-denied, missing file, non-file drop, multiple-file, cancellation, and unsupported action states.

Important boundaries:

- Feature 002 covers File Search / Browser / Attachment Portals.
- Feature 003 covers Agent Chat context composer and file attachment drops.
- Feature 019 covers SDK `path()`.
- Feature 016 covers `arg()`, `select()`, `div()`, and `md()`.
- Do not claim ACP/chat drop behavior or File Search drag-out behavior belongs to SDK `drop()` unless source proves shared route.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing, file-search-portals
- `lat.md`: scripting, protocol, surfaces, verification, design, permissions, builtins
- Source: SDK `drop()`, prompt handler drop route, app view state, render dispatch, DropPrompt entity/render, native footer routing, focus coordinator, UI window footer, collect_elements, simulateKey, protocol constructors, adjacent drag/drop files
- Tests/reports: SDK drop test, protocol submit, minimal chrome audits, tab AI input coverage, file-search drag tests

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 020 Drop Prompt / drop()

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
