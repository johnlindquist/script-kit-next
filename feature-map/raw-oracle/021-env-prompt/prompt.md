# 021 Env Prompt

```text
[env-prompt-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 021: Env Prompt / `env()` / environment variable and secret prompt / keyring storage.

This atlas must cover:

- TypeScript SDK API: `env(key, promptFn?)`, visible overloads or legacy usage, and how current smoke tests use options objects if source supports them.
- Current product truth for EnvPrompt: AppView routing, prompt ids, submit callbacks, existing env value handling, keyring lookup, auto-submit behavior, focus state, native footer surface, window sizing, and completion callback behavior.
- Secret detection/storage: which keys are secret, masking behavior, keyring storage, non-secret handling, deletion behavior, modified date/existing value display, and privacy/logging boundaries.
- EnvPrompt internals: input handling, validation/empty submit, prompt title/key/hint copy, keychain choice, existing value/update mode, and submit/cancel behavior.
- Footer and command routing: Submit footer ownership, Cmd+K/Actions if present, omitted launcher AI, native Run dispatch to `EnvPrompt::submit`, and no launcher fallthrough.
- Protocol and automation receipts: getState, getElements, simulateKey, ForceSubmit if supported, masked input values, activeFooter, key/value elements, and secret redaction.
- Relationship to script execution and config: process.env values, script environment, API key setup flows, `show_api_key_prompt`, startup completion channels, and what belongs outside EnvPrompt.
- Data/privacy boundaries: secret values, env var names, keychain entries, logs, screenshots, automation state, and persistence.
- Error, empty, disabled, loading, keyring failure, delete failure, existing secret, missing key, cancellation, and unsupported option states.

Important boundaries:

- Feature 016 covers `arg()`, `select()`, `div()`, and `md()`.
- Feature 017 covers `form()` and `fields()`.
- This feature covers SDK/env prompt behavior, not every environment-variable use in script execution or menu syntax.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing, storage-cache-security
- `lat.md`: scripting, protocol, surfaces, verification, design, builtins, logging, workspace if relevant
- Source: SDK `env()`, prompt handler env route, execution helper API key prompt route, EnvPrompt entity/helpers/render/tests, secrets/keyring, app view state, render dispatch, footer routing, focus coordinator, element collection, simulateKey
- Tests/reports: SDK env tests, smoke env visual/keychain/existing/overflow/title, protocol submit, minimal chrome audits, tab AI input coverage, source audits

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 021 Env Prompt / env()

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
