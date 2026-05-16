# 018 Editor and Template Prompt

```text
[editor-template-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 018: Editor and Template Prompt / `editor()` / `template()` / snippet tabstops / full-height editing.

This atlas must cover:

- TypeScript SDK APIs: `editor()` and `template()`.
- Current product truth for `EditorPrompt` versus `TemplatePrompt`: which SDK calls create which Rust app view, and where editor template/snippet mode overlaps with dedicated template prompt.
- Rust prompt handler routing into `AppView::EditorPrompt` and `AppView::TemplatePrompt`, prompt ids, submit callbacks, focus state, native footer surfaces, window sizing, and automation identity.
- EditorPrompt internals: content, language, height, full-height editor sizing, gpui-component code editor mode, tabstop/snippet behavior, choice popups if present, selection/editing behavior, content extraction, and submit/cancel paths.
- TemplatePrompt internals: placeholder parsing, input list, preview rendering, validation, keyboard navigation, footer behavior, and submit result.
- Actions and menus: `ActionsDialogHost::EditorPrompt`, editor actions, action backdrop, focus restore, and whether TemplatePrompt has equivalent action support.
- Keystrokes: text input, Tab, Shift+Tab, Enter, Cmd+Enter, Escape, snippet tabstop navigation, and any special key routing.
- Protocol and automation receipts: getState, getElements, simulateKey, ForceSubmit, typed input, editor focus, template prompt elements, and tabstop state.
- Data/privacy boundaries: editor contents, selected text, template values, prompt responses, action payloads, and any logs/screenshots that could expose sensitive text.
- Error, empty, disabled, validation, unsupported language, no-tabstop, and cancel states.

Important boundaries:

- Feature 016 covers `arg()`, `select()`, `div()`, and `md()`.
- Feature 017 covers `form()` and `fields()`.
- Feature 015 covers SDK `term()`.
- Notes editor and scratchpad may reuse editor concepts but are separate features unless source proves shared runtime behavior.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing
- `lat.md`: scripting, protocol, surfaces, verification, acp-chat, design
- Source: SDK `editor()`/`template()`, prompt handler editor/template routes, app view state, render dispatch, editor entity, template prompt entity, editor renderer, template renderer, actions dialog ownership, focus coordinator, UI window footer/sizing, element collection, simulateKey
- Tests/reports: SDK editor/template tests, smoke editor/template scripts, protocol submit, tab AI input coverage, minimal chrome/source audits, resize contract audits

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 018 Editor and Template Prompt / editor() / template()

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
