# 017 Form And Fields Prompt

```text
[form-fields-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 017: Form and Fields Prompt / form() / fields() / specialized field types / SDK form runtime.

This atlas must cover:

- TypeScript SDK APIs: `form()` and `fields()`.
- Current product truth: `form()` uses `AppView::FormPrompt`; `fields()` appears SDK-visible but GPUI backend support may be incomplete or intentionally coming soon.
- Rust prompt handler routing into `AppView::FormPrompt`, form ids, submit callbacks, focus state, native footer surface, and automation identity.
- HTML form parsing/rendering behavior: input fields, textarea, checkbox, submit, supported type handling, validation, Enter behavior, and field focus.
- Form actions and `ActionsDialogHost::FormPrompt`.
- `fields()` message shape, field definitions, field types, SDK tests, parity report, and current unimplemented/coming-soon behavior.
- Specialized field types: text, password, email, number, date, time, datetime-local, month, week, url, search, tel, color, checkbox/radio/file where current source or tests mention them.
- Protocol and automation receipts: getState, getElements, ForceSubmit, protocol submit, simulated input, form focus, and validation errors.
- Data/privacy boundaries: form HTML, field values, password fields, validation errors, submit payloads, action payloads.
- Error, empty, disabled, loading, validation, unsupported field, and unimplemented fields() states.

Important boundaries:

- Feature 016 covers `arg()`, `select()`, `div()`, and `md()`.
- Feature 015 covers SDK `term()`.
- Do not claim `fields()` is fully implemented if current source/tests show it is not.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing
- `lat.md`: design, surfaces, protocol, scripting, verification
- Source: SDK `form()`/`fields()`, prompt handler form route, app view state, prompt messages, form renderer/helpers/tests, element collection, simulateKey
- Tests/reports: SDK form all-types, specialized fields, fields basic/date-time, form/fields parity report, form smoke, protocol submit, fields audit

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 017 Form and Fields Prompt / form() / fields()

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
