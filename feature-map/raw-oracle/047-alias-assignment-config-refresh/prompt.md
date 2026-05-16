[alias-config-atlas]

Project briefing:

This is Script Kit GPUI, a Rust/GPUI desktop app with a Bun/TypeScript SDK and config tooling. The repository uses `lat.md/` as the architecture/test knowledge graph. Repo rules require `lat expand`, `lat search`, relevant skill context, `lat.md/` updates for behavior/docs changes, and `lat check`. This feature-map project is building an operator-grade atlas for humans and AI agents. Every Oracle answer is preserved under `feature-map/raw-oracle/<feature-id>/`, then locally distilled into `feature-map/features/<feature-id>.md`.

Feature id:

`047-alias-assignment-config-refresh`

Goal:

Produce a complete operator-grade feature atlas chapter for launcher alias assignment and removal: add/update/remove alias actions, AliasInput UI, alias persistence, command IDs, config/source-of-truth behavior, refresh semantics, duplicate/conflict behavior, supported/unsupported row types, action exposure, and verification. This is a focused follow-up to shortcut assignment because the handler says shortcut and alias configuration share action dispatch but have separate UI and persistence.

Current evidence:

- `lat expand "047 Launcher alias assignment config refresh: assign alias remove alias main menu actions config.ts command aliases alias recorder alias input duplicate conflict command IDs refresh app"` was run.
- `lat search "launcher alias assignment config.ts remove alias main menu actions command aliases alias input duplicate conflict refresh app"` returned:
  - `lat.md/shortcuts#Shortcuts#Removal Writes`
  - `lat.md/shortcuts#Shortcuts`
  - `lat.md/shortcuts#Shortcuts#Key Facts`
  - `lat.md/shortcuts#Shortcuts#Command IDs`
  - `lat.md/architecture#Architecture#Key Facts`
- Source search found `src/app_actions/handle_action/shortcuts.rs` handling `add_alias`, `update_alias`, and `remove_alias`; `src/app_impl/alias_input.rs`; `src/aliases/mod.rs`; `src/aliases/persistence.rs`; `src/components/alias_input/*`; source audits and smoke tests around alias conflict/action exposure.

Bundle map:

The attached bundle includes process docs, the goal file, owning skills, shortcut/command-id lat context, script validation context for duplicate metadata bindings, alias action handlers, AliasInput app integration and component code, alias persistence modules, command ID helpers, action helper messages, source-audit tests, smoke tests, and action-builder files for alias action exposure.

Please map what is present in the attached repo snapshot. Mark uncertain claims as inference. Do not invent behavior. If a user story or proof path is not implemented, call it a gap.

Required deliverable:

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

```markdown
## 047 Alias Assignment And Config Refresh

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

### Alias Persistence And Refresh Semantics

### Command IDs And Source Priority

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Conflict, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps
```

Specific questions to answer:

1. How does a user add or update an alias from main menu/actions surfaces?
2. How does a user remove an alias?
3. Which row types expose alias actions, and which row types should not?
4. How does AliasInput behave from open -> edit -> validate -> save/cancel/clear?
5. What exact alias validation rules exist?
6. Where is alias state persisted, and is it in `config.ts`, a separate alias store, script metadata, or another file?
7. How are alias overrides merged with script/scriptlet metadata aliases?
8. What duplicate/conflict behavior exists for aliases, both for metadata validation and user overrides?
9. What refresh/reload happens after save/remove?
10. What command IDs are used and how do aliases relate to command deeplinks?
11. What error states occur from unsupported rows, no selection, invalid aliases, persistence failures, duplicate aliases, or hidden commands?
12. What tests and runtime receipts prove this feature?
13. What unsafe claims should feature-map chapters avoid?

Repo-specific rules:

- Include the `lat.md/` update rule and required `lat check` in the verification plan.
- Treat screenshots as secondary; prefer state-first receipts where possible.
- Preserve raw Oracle output separately from the distilled feature chapter.
- Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
